//! Installable local host-agent integration bundles.

#[path = "agent_integration_bundle.rs"]
mod bundle;

use super::{
    AgentIntegrationCommands, AgentIntegrationLifecycleArgs, AgentIntegrationStatusArgs, ExitCode,
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

fn run_agent_integration_command(
    command: AgentIntegrationCommands,
) -> Result<RenderedIntegrationReport, String> {
    match command {
        AgentIntegrationCommands::Install(args) => lifecycle_command("install", args, false),
        AgentIntegrationCommands::Update(args) => lifecycle_command("update", args, true),
        AgentIntegrationCommands::Remove(args) => remove_command(args),
        AgentIntegrationCommands::Status(args) => status_command("status", args),
        AgentIntegrationCommands::Doctor(args) => status_command("doctor", args),
    }
}

fn lifecycle_command(
    action: &'static str,
    args: AgentIntegrationLifecycleArgs,
    update: bool,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    let manifest = bundle.manifest();
    let files = bundle.expected_files(&manifest);
    let mut actions = Vec::new();
    let mut changed = false;

    for file in &files {
        let status = file_status(&file.path);
        let content_changed = status.as_ref().map_or(true, |status| {
            !status.managed || status.content.as_deref() != Some(&file.content)
        });
        let write = args.force || update || content_changed;
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
        let content_changed = status.as_ref().map_or(true, |status| {
            !status.managed || status.content.as_deref() != Some(&file.content)
        });
        let write = args.force || update || content_changed;
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
            host: host_guidance(args.agent),
        },
        format: args.format,
    })
}

fn remove_command(
    args: AgentIntegrationLifecycleArgs,
) -> Result<RenderedIntegrationReport, String> {
    let project_root = resolve_project_root(args.path)?;
    let bundle = IntegrationBundle::new(args.agent, project_root);
    let manifest = bundle.manifest();
    let files = bundle.expected_files(&manifest);
    let mut actions = Vec::new();
    let mut changed = false;

    for file in files {
        let status = file_status(&file.path);
        let remove = status.as_ref().is_some_and(|status| status.managed);
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
            managed: status.as_ref().is_some_and(|status| status.managed),
        });
        if remove && !args.dry_run {
            fs::remove_file(&file.path).map_err(|error| error.to_string())?;
        }
    }
    if !args.dry_run {
        remove_empty_dir(&bundle.integration_dir)?;
    }

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
            host: host_guidance(args.agent),
        },
        format: args.format,
    })
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
    let checks = if action == "doctor" {
        doctor_checks(&bundle, &files)
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
            host: host_guidance(args.agent),
        },
        format: args.format,
    })
}

fn doctor_checks(bundle: &IntegrationBundle, files: &[FileAction]) -> Vec<DoctorCheck> {
    let all_expected_files_present = files.iter().all(|file| file.existed);
    let all_existing_files_managed = files.iter().all(|file| !file.existed || file.managed);
    let wrapper_path = bundle.integration_dir.join("assura-agent.sh");

    vec![
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
            status: check_status(wrapper_contains(&wrapper_path, "assura agent nudge")),
            message: "wrapper delegates to assura agent nudge",
        },
        DoctorCheck {
            name: "shared_check_contract",
            status: check_status(
                wrapper_contains(&wrapper_path, "assura check")
                    && wrapper_contains(&wrapper_path, "--format agent"),
            ),
            message: "wrapper delegates to assura check --format agent",
        },
        DoctorCheck {
            name: "shared_daemon_contract",
            status: check_status(
                wrapper_contains(&wrapper_path, "assura daemon status")
                    && wrapper_contains(&wrapper_path, "assura daemon doctor"),
            ),
            message: "wrapper delegates to assura daemon status/doctor",
        },
    ]
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
