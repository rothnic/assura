//! Installable local host-agent integration bundles.

#[path = "agent_integration_bundle.rs"]
mod bundle;
#[path = "agent_integration_host.rs"]
mod host;
#[path = "agent_integration_host_json.rs"]
mod host_json;
#[path = "agent_integration_templates.rs"]
mod templates;
#[path = "agent_integration_transaction.rs"]
mod transaction;

use super::{
    AgentIntegrationCommands, AgentIntegrationLifecycleArgs, AgentIntegrationStatusArgs,
    AgentIntegrationTarget, ExitCode, OutputFormat,
};
use crate::cli::ConfigDiscovery;
use bundle::{
    file_status, host_guidance, path_string, remove_empty_dir, set_executable_if_script,
    CheckStatus, DoctorCheck, FileAction, IntegrationBundle, IntegrationReport,
    RenderedIntegrationReport, OUTPUT_SCHEMA,
};
use std::fs;
use std::path::PathBuf;

/// Execute one agent integration lifecycle command.
pub async fn agent_integration_command(command: AgentIntegrationCommands) -> ExitCode {
    match run_agent_integration_command(command) {
        Ok(report) => {
            println!("{}", report.render());
            if report
                .report
                .checks
                .iter()
                .any(|check| check.status == CheckStatus::Fail)
            {
                ExitCode::ValidationFailed
            } else {
                ExitCode::Success
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::RuntimeError
        }
    }
}

pub(super) struct ManagedIntegrationState {
    pub(super) generated: bool,
    pub(super) activated: bool,
    pub(super) structural_verified: bool,
    pub(super) conflicted: bool,
    pub(super) post_activation_status: &'static str,
    pub(super) doctor_status: &'static str,
    pub(super) doctor_facts: Vec<ManagedIntegrationDoctorFact>,
    pub(super) host_approval_required: bool,
    pub(super) host_approval_status: &'static str,
    pub(super) host_approval_follow_up: &'static str,
}

pub(super) struct ManagedIntegrationDoctorFact {
    pub(super) name: &'static str,
    pub(super) status: &'static str,
    pub(super) detail: &'static str,
}

pub(super) fn configure_agent_integration_bundle(
    agent: AgentIntegrationTarget,
    project_root: PathBuf,
    activate: bool,
) -> Result<ManagedIntegrationState, String> {
    let report = lifecycle_command(
        if activate { "activate" } else { "install" },
        AgentIntegrationLifecycleArgs {
            agent,
            path: Some(project_root.clone()),
            dry_run: false,
            force: false,
            format: OutputFormat::Json,
        },
        activate,
        false,
    )?;
    let activation = report.report.activation;
    let post_activation = status_command(
        "status",
        AgentIntegrationStatusArgs {
            agent,
            path: Some(project_root.clone()),
            format: OutputFormat::Json,
        },
    )?
    .report
    .activation;
    let doctor = status_command(
        "doctor",
        AgentIntegrationStatusArgs {
            agent,
            path: Some(project_root),
            format: OutputFormat::Json,
        },
    )?
    .report;
    let doctor_status = if doctor
        .checks
        .iter()
        .any(|check| check.status == CheckStatus::Fail)
    {
        "fail"
    } else {
        "pass"
    };
    let doctor_facts = doctor
        .checks
        .into_iter()
        .map(|check| ManagedIntegrationDoctorFact {
            name: check.name,
            status: match check.status {
                CheckStatus::Pass => "pass",
                CheckStatus::Fail => "fail",
            },
            detail: check.message,
        })
        .collect();
    Ok(ManagedIntegrationState {
        generated: activation.generated,
        activated: activation.activated,
        structural_verified: activation.verified,
        conflicted: activation.conflicted,
        post_activation_status: activation_status(&post_activation),
        doctor_status,
        doctor_facts,
        host_approval_required: post_activation.host_approval_required,
        host_approval_status: if post_activation.host_approval_required {
            "unavailable"
        } else {
            "not_required"
        },
        host_approval_follow_up: host_approval_follow_up(agent),
    })
}

fn activation_status(activation: &bundle::ActivationReport) -> &'static str {
    if activation.conflicted {
        "conflicted"
    } else if activation.verified {
        "structurally_verified"
    } else if activation.activated {
        "activated"
    } else if activation.generated {
        "generated"
    } else {
        "not_generated"
    }
}

fn host_approval_follow_up(agent: AgentIntegrationTarget) -> &'static str {
    match agent {
        AgentIntegrationTarget::Codex => {
            "Review the generated project hooks, then trust the project and approve its hooks in Codex."
        }
        AgentIntegrationTarget::Claude => {
            "Review the generated project hooks, then trust the project hooks in Claude Code."
        }
        AgentIntegrationTarget::Pi => {
            "Review the generated project extension, then trust the project extension in Pi."
        }
        AgentIntegrationTarget::Opencode => {
            "Review and trust the generated project plugin in OpenCode before relying on delivery."
        }
    }
}

/// Resolve an installed harness name to its supported integration target.
pub(super) fn target_from_harness(agent_harness: &str) -> Option<AgentIntegrationTarget> {
    match agent_harness {
        "codex" => Some(AgentIntegrationTarget::Codex),
        "opencode" => Some(AgentIntegrationTarget::Opencode),
        "claude" => Some(AgentIntegrationTarget::Claude),
        "pi" => Some(AgentIntegrationTarget::Pi),
        _ => None,
    }
}

fn run_agent_integration_command(
    command: AgentIntegrationCommands,
) -> Result<RenderedIntegrationReport, String> {
    match command {
        AgentIntegrationCommands::Install(args) => lifecycle_command("install", args, false, false),
        AgentIntegrationCommands::Activate(args) => {
            lifecycle_command("activate", args, true, false)
        }
        AgentIntegrationCommands::Update(args) => lifecycle_command("update", args, false, true),
        AgentIntegrationCommands::Deactivate(args) => deactivate_command(args),
        AgentIntegrationCommands::Remove(args) => remove_command(args),
        AgentIntegrationCommands::Status(args) => status_command("status", args),
        AgentIntegrationCommands::Doctor(args) => status_command("doctor", args),
    }
}

fn lifecycle_command(
    action: &'static str,
    args: AgentIntegrationLifecycleArgs,
    activate: bool,
    update: bool,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    let initial_activation = host::status(&bundle)?;
    let manifest = bundle.manifest();
    let files = bundle.expected_files(&manifest);
    let transaction_paths = files
        .iter()
        .map(|file| file.path.clone())
        .chain(host::managed_paths(&bundle))
        .collect::<Vec<_>>();

    transaction::run(
        transaction_paths,
        &bundle.project_root,
        args.dry_run,
        || {
            let mut actions = Vec::new();
            let mut changed = false;

            for file in &files {
                let status = file_status(&file.path);
                let content_changed = status.as_ref().is_none_or(|status| {
                    !status.managed || status.content.as_deref() != Some(&file.content)
                });
                let write = args.force || content_changed;
                if write
                    && !args.force
                    && !args.dry_run
                    && status.as_ref().is_some_and(|status| !status.managed)
                {
                    return Err(format!(
                        "refusing to overwrite non-Assura-managed file: {}; rerun with --force to replace it",
                        path_string(&file.path)
                    ));
                }
            }

            for file in files {
                let status = file_status(&file.path);
                let content_changed = status.as_ref().is_none_or(|status| {
                    !status.managed || status.content.as_deref() != Some(&file.content)
                });
                let write = args.force || content_changed;
                let action_name = if write { "write" } else { "unchanged" };
                changed |= write;
                actions.push(FileAction {
                    path: path_string(&file.path),
                    kind: file.kind,
                    action: if args.dry_run && write {
                        "would_write"
                    } else if args.dry_run {
                        "unchanged"
                    } else {
                        action_name
                    },
                    existed: status.is_some(),
                    managed: status.as_ref().is_some_and(|status| status.managed),
                });
                if write && !args.dry_run {
                    if let Some(parent) = file.path.parent() {
                        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                    }
                    fs::write(&file.path, file.content).map_err(|error| error.to_string())?;
                    set_executable_if_script(&file.path, file.kind)?;
                }
            }

            let should_refresh_activation = activate || (update && initial_activation.activated);
            let activation = if should_refresh_activation {
                let mutation = host::activate(&bundle, args.dry_run)?;
                changed |= mutation.changed;
                actions.extend(mutation.files);
                mutation.state
            } else if args.dry_run {
                initial_activation
            } else {
                host::status(&bundle)?
            };

            Ok(RenderedIntegrationReport {
                report: IntegrationReport {
                    schema: OUTPUT_SCHEMA,
                    action,
                    agent: args.agent.as_str(),
                    dry_run: args.dry_run,
                    changed,
                    installed: bundle.is_installed(),
                    project_root: path_string(&bundle.project_root),
                    integration_dir: path_string(&bundle.integration_dir),
                    manifest: Some(manifest),
                    files: actions,
                    checks: Vec::new(),
                    activation,
                    host: host_guidance(args.agent),
                },
                format: args.format,
            })
        },
    )
}

fn deactivate_command(
    args: AgentIntegrationLifecycleArgs,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    transaction::run(
        host::managed_paths(&bundle),
        &bundle.project_root,
        args.dry_run,
        || {
            let mutation = host::deactivate(&bundle, args.dry_run)?;
            Ok(RenderedIntegrationReport {
                report: IntegrationReport {
                    schema: OUTPUT_SCHEMA,
                    action: "deactivate",
                    agent: args.agent.as_str(),
                    dry_run: args.dry_run,
                    changed: mutation.changed,
                    installed: bundle.is_installed(),
                    project_root: path_string(&bundle.project_root),
                    integration_dir: path_string(&bundle.integration_dir),
                    manifest: Some(bundle.manifest()),
                    files: mutation.files,
                    checks: Vec::new(),
                    activation: mutation.state,
                    host: host_guidance(args.agent),
                },
                format: args.format,
            })
        },
    )
}

fn remove_command(
    args: AgentIntegrationLifecycleArgs,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    let manifest = bundle.manifest();
    let files = bundle.expected_files(&manifest);
    let transaction_paths = files
        .iter()
        .map(|file| file.path.clone())
        .chain(host::managed_paths(&bundle))
        .collect::<Vec<_>>();

    transaction::run(
        transaction_paths,
        &bundle.project_root,
        args.dry_run,
        || {
            for file in &files {
                if let Some(status) = file_status(&file.path) {
                    if status.content.as_deref() != Some(file.content.as_str()) {
                        return Err(format!(
                            "refusing to remove drifted or non-Assura-managed bundle file: {}",
                            path_string(&file.path)
                        ));
                    }
                }
            }

            let activation_mutation = host::deactivate(&bundle, args.dry_run)?;
            let mut actions = activation_mutation.files;
            let mut changed = activation_mutation.changed;

            for file in files {
                let status = file_status(&file.path);
                let remove = status.is_some();
                changed |= remove;
                actions.push(FileAction {
                    path: path_string(&file.path),
                    kind: file.kind,
                    action: if args.dry_run {
                        if remove {
                            "would_remove"
                        } else {
                            "unchanged"
                        }
                    } else if remove {
                        "remove"
                    } else {
                        "unchanged"
                    },
                    existed: status.is_some(),
                    managed: remove,
                });
                if remove && !args.dry_run {
                    fs::remove_file(&file.path).map_err(|error| error.to_string())?;
                }
            }
            if !args.dry_run {
                remove_empty_dir(&bundle.integration_dir)?;
            }
            let activation = if args.dry_run {
                activation_mutation.state
            } else {
                host::status(&bundle)?
            };

            Ok(RenderedIntegrationReport {
                report: IntegrationReport {
                    schema: OUTPUT_SCHEMA,
                    action: "remove",
                    agent: args.agent.as_str(),
                    dry_run: args.dry_run,
                    changed,
                    installed: bundle.is_installed(),
                    project_root: path_string(&bundle.project_root),
                    integration_dir: path_string(&bundle.integration_dir),
                    manifest: None,
                    files: actions,
                    checks: Vec::new(),
                    activation,
                    host: host_guidance(args.agent),
                },
                format: args.format,
            })
        },
    )
}

fn status_command(
    action: &'static str,
    args: AgentIntegrationStatusArgs,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    let manifest = bundle.manifest();
    let files = bundle
        .expected_files(&manifest)
        .into_iter()
        .map(|file| {
            let status = file_status(&file.path);
            FileAction {
                path: path_string(&file.path),
                kind: file.kind,
                action: if status.is_some() {
                    "present"
                } else {
                    "missing"
                },
                existed: status.is_some(),
                managed: status.as_ref().is_some_and(|status| status.managed),
            }
        })
        .collect::<Vec<_>>();
    let installed = bundle.is_installed();
    let activation = host::status(&bundle)?;
    let checks = if action == "doctor" {
        doctor_checks(&bundle, &files)?
    } else {
        Vec::new()
    };

    Ok(RenderedIntegrationReport {
        report: IntegrationReport {
            schema: OUTPUT_SCHEMA,
            action,
            agent: args.agent.as_str(),
            dry_run: false,
            changed: false,
            installed,
            project_root: path_string(&bundle.project_root),
            integration_dir: path_string(&bundle.integration_dir),
            manifest: Some(manifest),
            files,
            checks,
            activation,
            host: host_guidance(args.agent),
        },
        format: args.format,
    })
}

fn doctor_checks(
    bundle: &IntegrationBundle,
    files: &[FileAction],
) -> Result<Vec<DoctorCheck>, String> {
    let all_expected_files_present = files.iter().all(|file| file.existed);
    let all_existing_files_managed = files.iter().all(|file| !file.existed || file.managed);
    let wrapper_path = bundle.integration_dir.join("assura-agent.sh");
    let activation = host::status(bundle)?;

    Ok(vec![
        DoctorCheck {
            name: "config",
            status: if bundle.project_root.join(".assura/config.yml").is_file() {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            message: "project has .assura/config.yml",
        },
        DoctorCheck {
            name: "bundle_installed",
            status: if bundle.is_installed() {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            message: "managed integration manifest exists",
        },
        DoctorCheck {
            name: "expected_files",
            status: check_status(all_expected_files_present),
            message: "manifest, wrapper, and README files are present",
        },
        DoctorCheck {
            name: "managed_files",
            status: check_status(all_existing_files_managed),
            message: "expected files are Assura-managed",
        },
        DoctorCheck {
            name: "shared_nudge_contract",
            status: check_status(wrapper_contains(
                &wrapper_path,
                "\"$ASSURA_BIN\" agent nudge",
            )),
            message: "wrapper delegates to assura agent nudge",
        },
        DoctorCheck {
            name: "nudge_logging_contract",
            status: check_status(
                wrapper_contains(&wrapper_path, "ASSURA_AGENT_LOG")
                    && wrapper_contains(&wrapper_path, ".assura/agent-sessions"),
            ),
            message: "wrapper records nudge payloads for session review",
        },
        DoctorCheck {
            name: "shared_check_contract",
            status: check_status(
                wrapper_contains(&wrapper_path, "\"$ASSURA_BIN\" check")
                    && wrapper_contains(&wrapper_path, "--format agent"),
            ),
            message: "wrapper delegates to assura check --format agent",
        },
        DoctorCheck {
            name: "shared_daemon_contract",
            status: check_status(
                wrapper_contains(&wrapper_path, "\"$ASSURA_BIN\" daemon status")
                    && wrapper_contains(&wrapper_path, "\"$ASSURA_BIN\" daemon doctor"),
            ),
            message: "wrapper delegates to assura daemon status/doctor",
        },
        DoctorCheck {
            name: "host_configuration",
            status: check_status(
                activation.activated && activation.verified && !activation.conflicted,
            ),
            message: "Assura-managed project host configuration is structurally verified",
        },
    ])
}

fn check_status(pass: bool) -> CheckStatus {
    if pass {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    }
}

fn wrapper_contains(path: &std::path::Path, needle: &str) -> bool {
    match fs::read_to_string(path) {
        Ok(text) => text.contains(needle),
        Err(_) => false,
    }
}

fn resolve_project_root(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let path = match path {
        Some(path) => path,
        None => std::env::current_dir().map_err(|error| error.to_string())?,
    };
    if path.join(".assura/config.yml").is_file() {
        return Ok(path);
    }
    Ok(ConfigDiscovery::find_project_root(&path).unwrap_or(path))
}
