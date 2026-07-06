//! Project doctor report model and assembly helpers.

use super::check::StructureCheckReport;
use super::doctor_content;
use crate::config::config::Config;
use serde::Serialize;
use std::fs;
use std::path::Path;

const DOCTOR_SCHEMA: &str = "assura.project-doctor.v1";

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectDoctorReport {
    schema: &'static str,
    pub(super) project_root: String,
    pub(super) config_path: String,
    pub(super) checked_path: String,
    pub(super) check: DoctorCheckSummary,
    pub(super) configured: Vec<DoctorItem>,
    pub(super) inactive: Vec<DoctorItem>,
    pub(super) gaps: Vec<DoctorItem>,
    pub(super) binary_custody: DoctorItem,
    pub(super) blocking_violations: Vec<DoctorViolation>,
    pub(super) next_actions: Vec<DoctorNextAction>,
}

pub(super) struct ProjectDoctorBuild {
    pub(super) doctor: ProjectDoctorReport,
    pub(super) structure_report: StructureCheckReport,
}

impl ProjectDoctorReport {
    pub(super) fn from_check(report: &StructureCheckReport, config: &Config) -> Self {
        let configured = configured_items(&report, &config);
        let inactive = inactive_items(&report.project_root, &config);
        let gaps = gap_items(&report.project_root, &config);
        let binary_custody = binary_custody_item(&report.project_root);
        let blocking_violations = blocking_violations(&report);
        let next_actions = next_actions(&report, &inactive, &gaps);
        Self {
            schema: DOCTOR_SCHEMA,
            project_root: display_path(&report.project_root),
            config_path: display_path(&report.config_path),
            checked_path: display_path(&report.checked_path),
            check: DoctorCheckSummary {
                status: if report.success { "pass" } else { "fail" },
                files_checked: report.files_checked,
                dirs_checked: report.dirs_checked,
                violations: report.violation_count(),
            },
            configured,
            inactive,
            gaps,
            binary_custody,
            blocking_violations,
            next_actions,
        }
    }

    pub(super) fn render_text(&self) -> String {
        let mut lines = vec![
            "Assura project doctor".to_string(),
            format!(
                "check={} files={} dirs={} violations={}",
                self.check.status,
                self.check.files_checked,
                self.check.dirs_checked,
                self.check.violations
            ),
            "checked means configured checks ran; inactive means not configured yet.".to_string(),
            format!(
                "configured: {}",
                self.configured
                    .iter()
                    .map(|item| format!("{}={}", item.name, item.status))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "inactive: {}",
                self.inactive
                    .iter()
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ];
        if !self.gaps.is_empty() {
            lines.push(format!(
                "gaps: {}",
                self.gaps
                    .iter()
                    .map(|item| format!("{}={}", item.name, item.status))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(violation) = self.blocking_violations.first() {
            lines.push(format!(
                "top-violation: {} {} {}",
                violation.path, violation.rule, violation.severity
            ));
        }
        if let Some(action) = self.next_actions.first() {
            lines.push(format!("next: {}", action.action));
            lines.push(format!("follow-up: {}", action.follow_up));
        }
        lines.join("\n")
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DoctorCheckSummary {
    pub(super) status: &'static str,
    pub(super) files_checked: usize,
    pub(super) dirs_checked: usize,
    pub(super) violations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DoctorItem {
    pub(super) name: String,
    pub(super) status: &'static str,
    pub(super) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DoctorNextAction {
    pub(super) priority: u32,
    pub(super) action: String,
    pub(super) follow_up: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DoctorViolation {
    pub(super) path: String,
    pub(super) rule: String,
    pub(super) severity: String,
    pub(super) blocking: bool,
    pub(super) message: String,
}

fn blocking_violations(report: &StructureCheckReport) -> Vec<DoctorViolation> {
    report
        .violations
        .iter()
        .filter(|violation| violation.blocking)
        .take(5)
        .map(|violation| DoctorViolation {
            path: display_path(&violation.path),
            rule: violation.rule.clone(),
            severity: violation.severity.clone(),
            blocking: violation.blocking,
            message: violation.message.clone(),
        })
        .collect()
}

fn configured_items(report: &StructureCheckReport, config: &Config) -> Vec<DoctorItem> {
    let extensions = config.extensions.as_ref();
    let repository_references = extensions
        .map(|extensions| extensions.repository_references.len())
        .unwrap_or_default();
    let frontmatter_reference_fields = extensions
        .map(|extensions| {
            extensions
                .repository_references
                .iter()
                .map(|policy| policy.frontmatter_fields.len())
                .sum::<usize>()
        })
        .unwrap_or_default();
    let unresolved_repository_references = report
        .violations
        .iter()
        .filter(|violation| violation.rule == "repository_reference_target")
        .count();
    let relationships = extensions
        .map(|extensions| extensions.relationships.len())
        .unwrap_or_default();
    vec![
        DoctorItem {
            name: "structure_config".to_string(),
            status: "active",
            detail: format!(
                "{} structure root(s); last check status {}",
                config.structure.len(),
                if report.success { "pass" } else { "fail" }
            ),
        },
        DoctorItem {
            name: "content_models".to_string(),
            status: if config.models.is_some() {
                "active"
            } else {
                "inactive"
            },
            detail: config
                .models
                .as_ref()
                .map(|models| format!("validation artifact {}", models.validation_artifact))
                .unwrap_or_else(|| "models.validation_artifact is not configured".to_string()),
        },
        DoctorItem {
            name: "collections".to_string(),
            status: if config.collections.is_empty() {
                "inactive"
            } else {
                "active"
            },
            detail: format!("{} configured collection(s)", config.collections.len()),
        },
        DoctorItem {
            name: "repository_references".to_string(),
            status: if repository_references == 0 {
                "inactive"
            } else {
                "active"
            },
            detail: format!(
                "{repository_references} configured repository-reference policy item(s); {frontmatter_reference_fields} configured frontmatter field(s); {unresolved_repository_references} unresolved reference target violation(s)"
            ),
        },
        DoctorItem {
            name: "structure_relationships".to_string(),
            status: if relationships == 0 {
                "inactive"
            } else {
                "active"
            },
            detail: format!("{relationships} configured relationship constraint(s)"),
        },
    ]
}

fn inactive_items(project_root: &Path, config: &Config) -> Vec<DoctorItem> {
    let mut items = Vec::new();
    if config.models.is_none() {
        items.push(DoctorItem {
            name: "content_models".to_string(),
            status: "inactive",
            detail: "No models.validation_artifact is configured.".to_string(),
        });
    }
    if config.collections.is_empty() {
        items.push(DoctorItem {
            name: "collections".to_string(),
            status: "inactive",
            detail: "No content collections are configured.".to_string(),
        });
    }
    if config.models.is_none() || config.collections.is_empty() {
        items.push(DoctorItem {
            name: "search_chunks".to_string(),
            status: "unchecked",
            detail:
                "No active content runtime is configured for project-doctor search chunk checks."
                    .to_string(),
        });
    }
    if config
        .extensions
        .as_ref()
        .map(|extensions| extensions.repository_references.is_empty())
        .unwrap_or(true)
    {
        items.push(DoctorItem {
            name: "unresolved_references".to_string(),
            status: "unchecked",
            detail: "No repository-reference diagnostics policy is configured.".to_string(),
        });
    }
    if !project_root
        .join(".assura/onboarding/agent-next.md")
        .is_file()
    {
        items.push(DoctorItem {
            name: "onboarding_packet".to_string(),
            status: "inactive",
            detail: ".assura/onboarding/agent-next.md was not found.".to_string(),
        });
    }
    items
}

fn gap_items(project_root: &Path, config: &Config) -> Vec<DoctorItem> {
    let mut gaps = Vec::new();
    if has_draft_model_files(project_root) && config.models.is_none() {
        gaps.push(DoctorItem {
            name: "draft_models_unwired".to_string(),
            status: "gap",
            detail: "Model files exist under .assura/models, but models.validation_artifact is not configured.".to_string(),
        });
    }
    for (name, path) in [
        ("agents_guidance", "AGENTS.md"),
        ("skill_directory", ".agents/skills"),
        ("process_docs", "docs/process"),
        ("learnings_docs", "docs/learnings"),
    ] {
        if !project_root.join(path).exists() {
            gaps.push(DoctorItem {
                name: name.to_string(),
                status: "recommended_missing",
                detail: format!("Recommended agent-project baseline path `{path}` is missing."),
            });
        }
    }
    gaps.extend(doctor_content::content_runtime_gaps(project_root, config));
    gaps
}

fn binary_custody_item(project_root: &Path) -> DoctorItem {
    let manifest = project_root.join("source-documents/manifest.md");
    let files = project_root.join("source-documents/files");
    if manifest.is_file() && files.is_dir() {
        DoctorItem {
            name: "binary_custody".to_string(),
            status: "active",
            detail: "source-documents manifest and files directory are present.".to_string(),
        }
    } else {
        DoctorItem {
            name: "binary_custody".to_string(),
            status: "inactive",
            detail: "No source-documents/manifest.md custody pattern is active.".to_string(),
        }
    }
}

fn next_actions(
    report: &StructureCheckReport,
    inactive: &[DoctorItem],
    gaps: &[DoctorItem],
) -> Vec<DoctorNextAction> {
    let mut actions = Vec::new();
    if !report.success {
        actions.push(DoctorNextAction {
            priority: 1,
            action: "Fix blocking configured check violations first.".to_string(),
            follow_up: "assura check --format agent .".to_string(),
        });
    }
    if let Some(gap) = gaps.first() {
        actions.push(DoctorNextAction {
            priority: (actions.len() + 1) as u32,
            action: format!("Address recommended preset gap `{}`.", gap.name),
            follow_up: "assura agent onboard --format json .".to_string(),
        });
    }
    if inactive.iter().any(|item| item.name == "content_models") {
        actions.push(DoctorNextAction {
            priority: (actions.len() + 1) as u32,
            action: "Decide whether this project needs content models before assuming project facts are modeled.".to_string(),
            follow_up: ".assura/onboarding/agent-next.md".to_string(),
        });
    }
    actions.push(DoctorNextAction {
        priority: (actions.len() + 1) as u32,
        action: "Use path explanation for the next confusing file or directory.".to_string(),
        follow_up: "assura explain <path> --format json".to_string(),
    });
    actions
}

fn has_draft_model_files(project_root: &Path) -> bool {
    let model_dir = project_root.join(".assura/models");
    let Ok(entries) = fs::read_dir(model_dir) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        path.is_file() || (path.is_dir() && directory_has_file(&path))
    })
}

fn directory_has_file(path: &Path) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let path = entry.path();
        path.is_file() || (path.is_dir() && directory_has_file(&path))
    })
}

fn display_path(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}
