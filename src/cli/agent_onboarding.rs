//! First-run local onboarding for agent-ready repositories.

use super::agent_integration::install_agent_integration_bundle;
use super::agent_lifecycle::{lifecycle_profiles, ranked_next_actions};
use super::agent_onboarding_report::{
    render_report, CheckItem, ContentSection, FileAction, InstalledSection, IntegrationSection,
    OnboardingReport, RenderedOnboardingReport,
};
use super::agent_onboarding_rules::{normalize_existing_root, recommended_rules};
use super::agent_onboarding_templates::{baseline_files, rule_recommendations_file, GeneratedFile};
use super::doctor::project_doctor_packet_json;
use super::project_review::build_project_review;
use super::{
    AgentContentTemplate, AgentIntegrationTarget, AgentOnboardingTarget, ExitCode, OutputFormat,
};
use serde::Serialize;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

const OUTPUT_SCHEMA: &str = "assura.agent-onboarding.v1";
/// Options for `assura agent onboard`.
pub struct AgentOnboardingOptions {
    /// Project root directory.
    pub path: Option<PathBuf>,
    /// Requested host-agent profile.
    pub agent: AgentOnboardingTarget,
    /// Optional content runtime activation template.
    pub content_template: AgentContentTemplate,
    /// Output format.
    pub format: OutputFormat,
}

/// Run the first-run agent onboarding command.
pub async fn agent_onboarding_command(
    options: AgentOnboardingOptions,
    config: Option<PathBuf>,
) -> ExitCode {
    match run_agent_onboarding(options, config) {
        Ok(report) => {
            println!("{}", render_report(&report));
            if report
                .report
                .verified
                .iter()
                .all(|item| item.status == "pass")
            {
                ExitCode::Success
            } else {
                ExitCode::ValidationFailed
            }
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::RuntimeError
        }
    }
}

fn run_agent_onboarding(
    options: AgentOnboardingOptions,
    config: Option<PathBuf>,
) -> Result<RenderedOnboardingReport, String> {
    let project_root = resolve_project_root(options.path)?;
    fs::create_dir_all(&project_root).map_err(|error| error.to_string())?;

    let detected = detect_project(&project_root, options.agent);
    let config_path = config.unwrap_or_else(|| project_root.join(".assura/config.yml"));
    let mut files = Vec::new();
    for file in baseline_files(&detected, options.content_template) {
        files.push(materialize_baseline_file(&project_root, file)?);
    }
    let rule_recommendations = recommended_rules(&detected, &config_path)?;
    files.push(materialize_managed_file(
        &project_root,
        rule_recommendations_file(&detected, rule_recommendations[0].status),
    )?);

    let integration_target = integration_target(&detected);
    let integration = install_integration(&project_root, &detected, integration_target)?;
    let (verified, review) = verify_project(&project_root, Some(config_path.clone()))?;
    let content = content_section(options.content_template);
    let inactive = inactive_capabilities(options.content_template);
    let lifecycle_profiles = lifecycle_profiles(&project_root, integration_target);
    let next_actions = ranked_next_actions(integration_target, options.content_template);
    let doctor_json = project_doctor_packet_json(&project_root, Some(config_path))?;
    files.push(materialize_file(
        &project_root,
        GeneratedFile {
            path: ".assura/onboarding/doctor.json",
            contents: doctor_json,
            required: true,
            executable: false,
        },
    )?);

    Ok(RenderedOnboardingReport {
        report: OnboardingReport {
            schema: OUTPUT_SCHEMA,
            project_root: project_root.display().to_string(),
            installed: InstalledSection {
                assura_version: env!("CARGO_PKG_VERSION"),
                config: ".assura/config.yml",
                onboarding_packet: ".assura/onboarding/",
            },
            detected,
            rule_recommendations,
            integration,
            content,
            lifecycle_profiles,
            files,
            verified,
            review,
            inactive,
            next_actions,
        },
        format: options.format,
    })
}

fn resolve_project_root(path: Option<PathBuf>) -> Result<PathBuf, String> {
    let path = match path {
        Some(path) => path,
        None => std::env::current_dir().map_err(|error| error.to_string())?,
    };
    if path.exists() {
        path.canonicalize().map_err(|error| error.to_string())
    } else {
        Ok(path)
    }
}

fn detect_project(project_root: &Path, requested_agent: AgentOnboardingTarget) -> DetectedSection {
    let git_repository = project_root.join(".git").exists();
    let has_cargo = project_root.join("Cargo.toml").is_file();
    let has_package_json = project_root.join("package.json").is_file();
    let has_pyproject = project_root.join("pyproject.toml").is_file();
    let has_docs = project_root.join("docs").is_dir();
    let has_src = project_root.join("src").is_dir();
    let has_packages = project_root.join("packages").is_dir();
    let existing_source_files = has_src || has_cargo || has_package_json || has_pyproject;
    let project_type = if is_empty_project(project_root) {
        "empty"
    } else if has_packages {
        "monorepo"
    } else if has_cargo {
        "rust"
    } else if has_package_json {
        "node"
    } else if has_pyproject {
        "python"
    } else if has_docs {
        "docs-heavy"
    } else {
        "unknown"
    };
    let project_confidence = if matches!(project_type, "empty" | "unknown") {
        "low"
    } else {
        "high"
    };
    let agent = detect_agent(project_root, requested_agent);

    DetectedSection {
        project_type,
        project_confidence,
        requested_agent: requested_agent.as_str(),
        agent_harness: agent.target.as_str(),
        agent_confidence: agent.confidence,
        git_repository,
        existing_source_files,
    }
}

struct DetectedAgent {
    target: AgentOnboardingTarget,
    confidence: &'static str,
}

fn detect_agent(project_root: &Path, requested_agent: AgentOnboardingTarget) -> DetectedAgent {
    if requested_agent != AgentOnboardingTarget::Auto {
        return DetectedAgent {
            target: requested_agent,
            confidence: "explicit",
        };
    }
    for (path, target) in [
        (".codex", AgentOnboardingTarget::Codex),
        (".opencode", AgentOnboardingTarget::Opencode),
        (".claude", AgentOnboardingTarget::Claude),
        (".pi", AgentOnboardingTarget::Pi),
    ] {
        if project_root.join(path).exists() {
            return DetectedAgent {
                target,
                confidence: "high",
            };
        }
    }
    DetectedAgent {
        target: AgentOnboardingTarget::Generic,
        confidence: "low",
    }
}

fn is_empty_project(project_root: &Path) -> bool {
    let Ok(entries) = fs::read_dir(project_root) else {
        return false;
    };
    entries
        .filter_map(Result::ok)
        .all(|entry| entry.file_name() == ".git")
}

fn materialize_baseline_file(
    project_root: &Path,
    file: GeneratedFile,
) -> Result<FileAction, String> {
    if file.path == ".assura/config.yml" {
        return materialize_config(project_root, file);
    }
    materialize_file(project_root, file)
}

fn materialize_file(project_root: &Path, file: GeneratedFile) -> Result<FileAction, String> {
    let path = project_root.join(file.path);
    let existed = path.exists();
    let action = if existed { "existing" } else { "write" };
    if !existed {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&path, &file.contents).map_err(|error| error.to_string())?;
        set_executable_if_needed(&path, file.executable)?;
    }
    Ok(FileAction {
        path: file.path,
        action,
        existed,
        required: file.required,
    })
}

fn materialize_managed_file(
    project_root: &Path,
    file: GeneratedFile,
) -> Result<FileAction, String> {
    let path = project_root.join(file.path);
    let existed = path.exists();
    let current = if existed {
        Some(fs::read_to_string(&path).map_err(|error| error.to_string())?)
    } else {
        None
    };
    let changed = current.as_deref() != Some(file.contents.as_str());
    if changed {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&path, &file.contents).map_err(|error| error.to_string())?;
    }
    Ok(FileAction {
        path: file.path,
        action: if !existed {
            "write"
        } else if changed {
            "update"
        } else {
            "existing"
        },
        existed,
        required: file.required,
    })
}

#[cfg(unix)]
fn set_executable_if_needed(path: &Path, executable: bool) -> Result<(), String> {
    if !executable {
        return Ok(());
    }
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)
        .map_err(|error| error.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn set_executable_if_needed(_path: &Path, _executable: bool) -> Result<(), String> {
    Ok(())
}

fn materialize_config(project_root: &Path, file: GeneratedFile) -> Result<FileAction, String> {
    let path = project_root.join(file.path);
    let existed = path.exists();
    if !existed {
        return materialize_file(project_root, file);
    }

    let existing_contents = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    let mut existing: Value =
        serde_yaml::from_str(&existing_contents).map_err(|error| error.to_string())?;
    normalize_existing_root(&mut existing);
    let defaults: Value =
        serde_yaml::from_str(&file.contents).map_err(|error| error.to_string())?;
    let changed = merge_missing_values(&mut existing, &defaults);
    if changed {
        let merged = serde_yaml::to_string(&existing).map_err(|error| error.to_string())?;
        fs::write(&path, merged).map_err(|error| error.to_string())?;
    }

    Ok(FileAction {
        path: file.path,
        action: if changed { "merge" } else { "existing" },
        existed,
        required: file.required,
    })
}

fn merge_missing_values(existing: &mut Value, defaults: &Value) -> bool {
    match (existing, defaults) {
        (Value::Mapping(existing_map), Value::Mapping(default_map)) => {
            let mut changed = false;
            for (key, default_value) in default_map {
                if let Some(existing_value) = existing_map.get_mut(key) {
                    changed |= merge_missing_values(existing_value, default_value);
                } else {
                    existing_map.insert(key.clone(), default_value.clone());
                    changed = true;
                }
            }
            changed
        }
        (Value::Sequence(existing_items), Value::Sequence(default_items)) => {
            let mut changed = false;
            for default_item in default_items {
                if !existing_items.contains(default_item) {
                    existing_items.push(default_item.clone());
                    changed = true;
                }
            }
            changed
        }
        _ => false,
    }
}

fn install_integration(
    project_root: &Path,
    detected: &DetectedSection,
    target: Option<AgentIntegrationTarget>,
) -> Result<IntegrationSection, String> {
    if let Some(agent) = target {
        let installed = install_agent_integration_bundle(agent, project_root.to_path_buf())?;
        Ok(IntegrationSection {
            status: if installed {
                "installed"
            } else {
                "not-installed"
            },
            agent: detected.agent_harness,
            mode: "reviewable-local-bundle",
            detail:
                ".assura/integrations/<agent>/ generated; host-agent wiring remains manual opt-in",
        })
    } else {
        Ok(IntegrationSection {
            status: "generic-guidance",
            agent: "generic",
            mode: "manual-shell",
            detail: "no supported host-agent harness detected; use AGENTS.md and assura check --format agent --warn",
        })
    }
}

fn integration_target(detected: &DetectedSection) -> Option<AgentIntegrationTarget> {
    match detected.agent_harness {
        "codex" => Some(AgentIntegrationTarget::Codex),
        "opencode" => Some(AgentIntegrationTarget::Opencode),
        "claude" => Some(AgentIntegrationTarget::Claude),
        "pi" => Some(AgentIntegrationTarget::Pi),
        _ => None,
    }
}

fn verify_project(
    project_root: &Path,
    config: Option<PathBuf>,
) -> Result<(Vec<CheckItem>, OnboardingReview), String> {
    let config_path = config.unwrap_or_else(|| project_root.join(".assura/config.yml"));
    let report = build_project_review(
        Some(project_root.to_path_buf()),
        Some(config_path),
        None,
        false,
    )
    .map_err(|error| error.to_string())?;
    let (structure_status, review_status, blocking, advisory, inactive) =
        report.onboarding_summary();
    let verified = vec![
        CheckItem {
            name: "structure_config",
            status: if structure_status == "pass" {
                "pass"
            } else {
                "fail"
            },
            detail: ".assura/config.yml loaded and checked",
        },
        CheckItem {
            name: "onboarding_packet",
            status: if project_root
                .join(".assura/onboarding/agent-next.md")
                .is_file()
            {
                "pass"
            } else {
                "fail"
            },
            detail: ".assura/onboarding/agent-next.md exists",
        },
    ];
    let review = OnboardingReview {
        status: review_status,
        blocking,
        advisory,
        inactive,
        next_command: "assura review --format agent .",
    };
    Ok((verified, review))
}

fn content_section(template: AgentContentTemplate) -> ContentSection {
    if template.activates_content() {
        ContentSection {
            template: template.as_str(),
            status: "active",
            detail: "baseline repo-native content models are configured",
        }
    } else {
        ContentSection {
            template: "none",
            status: "inactive",
            detail: "content runtime activation was not requested",
        }
    }
}

fn inactive_capabilities(template: AgentContentTemplate) -> Vec<CheckItem> {
    let mut items = vec![CheckItem {
        name: "project_specialization",
        status: "inactive",
        detail: "waiting for user answers in .assura/onboarding/questions.md",
    }];
    if !template.activates_content() {
        items.push(CheckItem {
            name: "content_models",
            status: "inactive",
            detail: "deferred until --content-template is selected",
        });
    }
    items
}

#[derive(Clone, Serialize)]
pub(super) struct DetectedSection {
    pub(super) project_type: &'static str,
    pub(super) project_confidence: &'static str,
    pub(super) requested_agent: &'static str,
    pub(super) agent_harness: &'static str,
    pub(super) agent_confidence: &'static str,
    pub(super) git_repository: bool,
    pub(super) existing_source_files: bool,
}

#[derive(Serialize)]
pub(super) struct OnboardingReview {
    pub(super) status: &'static str,
    pub(super) blocking: usize,
    pub(super) advisory: usize,
    pub(super) inactive: usize,
    pub(super) next_command: &'static str,
}
