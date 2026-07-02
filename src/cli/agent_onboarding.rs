//! First-run local onboarding for agent-ready repositories.

use super::agent_integration::install_agent_integration_bundle;
use super::agent_onboarding_templates::{baseline_files, GeneratedFile};
use super::{AgentIntegrationTarget, AgentOnboardingTarget, ExitCode, OutputFormat};
use crate::cli::check::{run_structure_check_with_target_mode, CheckTargetMode};
use serde::Serialize;
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

const OUTPUT_SCHEMA: &str = "assura.agent-onboarding.v1";
const DOCTOR_SCHEMA: &str = "assura.agent-onboarding.doctor.v1";

/// Options for `assura agent onboard`.
pub struct AgentOnboardingOptions {
    /// Project root directory.
    pub path: Option<PathBuf>,
    /// Requested host-agent profile.
    pub agent: AgentOnboardingTarget,
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
    let mut files = Vec::new();
    for file in baseline_files(&detected) {
        files.push(materialize_baseline_file(&project_root, file)?);
    }

    let integration = install_integration(&project_root, &detected)?;
    let verified = verify_project(&project_root, config)?;
    let inactive = inactive_capabilities();
    let next_actions = next_actions(&detected);
    let doctor = OnboardingDoctor {
        schema: DOCTOR_SCHEMA,
        project_root: path_string(&project_root),
        checked: verified.clone(),
        inactive: inactive.clone(),
        next_actions: next_actions.clone(),
    };
    let doctor_json = serde_json::to_string_pretty(&doctor).map_err(|error| error.to_string())?;
    files.push(materialize_file(
        &project_root,
        GeneratedFile {
            path: ".assura/onboarding/doctor.json",
            contents: doctor_json,
            required: true,
        },
    )?);

    Ok(RenderedOnboardingReport {
        report: OnboardingReport {
            schema: OUTPUT_SCHEMA,
            project_root: path_string(&project_root),
            installed: InstalledSection {
                assura_version: env!("CARGO_PKG_VERSION"),
                config: ".assura/config.yml",
                onboarding_packet: ".assura/onboarding/",
            },
            detected,
            integration,
            files,
            verified,
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
    }
    Ok(FileAction {
        path: file.path,
        action,
        existed,
        required: file.required,
    })
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
) -> Result<IntegrationSection, String> {
    let target = match detected.agent_harness {
        "codex" => Some(AgentIntegrationTarget::Codex),
        "opencode" => Some(AgentIntegrationTarget::Opencode),
        "claude" => Some(AgentIntegrationTarget::Claude),
        "pi" => Some(AgentIntegrationTarget::Pi),
        _ => None,
    };
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

fn verify_project(project_root: &Path, config: Option<PathBuf>) -> Result<Vec<CheckItem>, String> {
    let config_path = config.unwrap_or_else(|| project_root.join(".assura/config.yml"));
    let report = run_structure_check_with_target_mode(
        Some(project_root.to_path_buf()),
        Some(config_path),
        false,
        CheckTargetMode::Recursive,
    )
    .map_err(|error| error.to_string())?;
    Ok(vec![
        CheckItem {
            name: "structure_config",
            status: if report.success { "pass" } else { "fail" },
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
    ])
}

fn inactive_capabilities() -> Vec<CheckItem> {
    vec![
        CheckItem {
            name: "project_specialization",
            status: "inactive",
            detail: "waiting for user answers in .assura/onboarding/questions.md",
        },
        CheckItem {
            name: "content_models",
            status: "inactive",
            detail: "deferred to the content activation child goal",
        },
        CheckItem {
            name: "domain_pack",
            status: "inactive",
            detail: "proposal/SBIR and other domain packs are optional future packs",
        },
    ]
}

fn next_actions(detected: &DetectedSection) -> Vec<&'static str> {
    let mut actions = vec![
        "Read .assura/onboarding/agent-next.md",
        "Ask the user the remaining specialization questions before adding project-specific rules",
        "Do not invent language, layout, naming, or domain conventions",
    ];
    if detected.agent_harness == "generic" {
        actions.push("Use generic shell guidance until a supported host-agent adapter is selected");
    }
    actions
}

fn render_report(report: &RenderedOnboardingReport) -> String {
    match report.format {
        OutputFormat::Json => serde_json::to_string_pretty(&report.report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(&report.report).unwrap_or_default(),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => report.render_text(),
    }
}

fn path_string(path: &Path) -> String {
    path.display().to_string()
}

#[derive(Serialize)]
struct RenderedOnboardingReport {
    #[serde(flatten)]
    report: OnboardingReport,
    #[serde(skip)]
    format: OutputFormat,
}

impl RenderedOnboardingReport {
    fn render_text(&self) -> String {
        let report = &self.report;
        format!(
            "Assura agent onboarding\ninstalled: assura {}\ndetected: project={} agent={} confidence={}\nverified: {}\ninactive: {}\nnext: {}\npacket: .assura/onboarding/agent-next.md",
            report.installed.assura_version,
            report.detected.project_type,
            report.detected.agent_harness,
            report.detected.agent_confidence,
            report
                .verified
                .iter()
                .map(|item| format!("{}={}", item.name, item.status))
                .collect::<Vec<_>>()
                .join(", "),
            report
                .inactive
                .iter()
                .map(|item| item.name)
                .collect::<Vec<_>>()
                .join(", "),
            report.next_actions.first().copied().unwrap_or("read agent-next.md")
        )
    }
}

#[derive(Serialize)]
struct OnboardingReport {
    schema: &'static str,
    project_root: String,
    installed: InstalledSection,
    detected: DetectedSection,
    integration: IntegrationSection,
    files: Vec<FileAction>,
    verified: Vec<CheckItem>,
    inactive: Vec<CheckItem>,
    next_actions: Vec<&'static str>,
}

#[derive(Serialize)]
struct InstalledSection {
    assura_version: &'static str,
    config: &'static str,
    onboarding_packet: &'static str,
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
struct IntegrationSection {
    status: &'static str,
    agent: &'static str,
    mode: &'static str,
    detail: &'static str,
}

#[derive(Serialize)]
struct FileAction {
    path: &'static str,
    action: &'static str,
    existed: bool,
    required: bool,
}

#[derive(Clone, Serialize)]
struct CheckItem {
    name: &'static str,
    status: &'static str,
    detail: &'static str,
}

#[derive(Serialize)]
struct OnboardingDoctor {
    schema: &'static str,
    project_root: String,
    checked: Vec<CheckItem>,
    inactive: Vec<CheckItem>,
    next_actions: Vec<&'static str>,
}
