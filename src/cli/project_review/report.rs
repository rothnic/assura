//! Project review report assembly and rendering.

use super::heatmap::ProjectReviewHeatmap;
use super::history::{apply_finding_history, FindingHistorySummary};
use super::text::render_project_review_text;
use crate::cli::args::CheckOutputFormat;
use crate::cli::content_query::AgentQueryGapsOutput;
use crate::cli::doctor::{DoctorItem, DoctorNextAction, DoctorViolation, ProjectDoctorReport};
use serde::Serialize;

const PROJECT_REVIEW_SCHEMA: &str = "assura.project-review.v2";
const PROJECT_REVIEW_AGENT_SCHEMA: &str = "assura.project-review.agent.v2";

pub(super) fn render_project_review(
    report: &ProjectReviewReport,
    format: CheckOutputFormat,
) -> String {
    match format {
        CheckOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        CheckOutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        CheckOutputFormat::Agent => {
            serde_json::to_string_pretty(&ProjectReviewAgentReport::from(report))
                .unwrap_or_default()
        }
        CheckOutputFormat::Text | CheckOutputFormat::Advice | CheckOutputFormat::Status => {
            render_project_review_text(report)
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectReviewReport {
    schema: &'static str,
    pub(super) status: &'static str,
    project_root: String,
    config_path: String,
    checked_path: String,
    pub(super) structure: ProjectReviewStructure,
    pub(super) summary: ProjectReviewSummary,
    pub(super) findings: Vec<ProjectReviewFinding>,
    pub(super) finding_history: FindingHistorySummary,
    pub(super) content_gaps: ProjectReviewContentGaps,
    pub(super) heatmap: ProjectReviewHeatmap,
    omitted_noise: Vec<ProjectReviewOmission>,
    pub(super) next_actions: Vec<ProjectReviewAction>,
    pub(super) lower_level_commands: Vec<&'static str>,
}

impl ProjectReviewReport {
    pub(super) fn from_parts(
        doctor: ProjectDoctorReport,
        content_gaps: AgentQueryGapsOutput,
        heatmap: ProjectReviewHeatmap,
        persist_history: bool,
    ) -> Self {
        let mut findings = Vec::new();
        findings.extend(blocking_findings(&doctor.blocking_violations));
        findings.extend(advisory_findings(&doctor.gaps));
        findings.extend(inactive_findings(&doctor.inactive, &doctor.binary_custody));
        findings.extend(content_findings(&content_gaps));
        findings.push(structure_fit_finding());

        let finding_history =
            apply_finding_history(&doctor.project_root, &mut findings, persist_history);
        let omitted_noise = omitted_noise_policy();
        let summary = ProjectReviewSummary::from_findings(&findings, omitted_noise.len());
        let status = if summary.blocking > 0 {
            "fail"
        } else if summary.advisory > 0 || summary.inactive > 0 {
            "needs-review"
        } else {
            "pass"
        };

        Self {
            schema: PROJECT_REVIEW_SCHEMA,
            status,
            project_root: doctor.project_root,
            config_path: doctor.config_path,
            checked_path: doctor.checked_path,
            structure: ProjectReviewStructure {
                status: doctor.check.status,
                files_checked: doctor.check.files_checked,
                dirs_checked: doctor.check.dirs_checked,
                violations: doctor.check.violations,
            },
            summary,
            findings,
            finding_history,
            content_gaps: ProjectReviewContentGaps::from(content_gaps),
            heatmap,
            omitted_noise,
            next_actions: review_next_actions(&doctor.next_actions),
            lower_level_commands: vec![
                "assura check --format json .",
                "assura doctor --format json .",
                "assura content agent-query gaps --format json .",
                "assura explain <path> --format json",
            ],
        }
    }

    pub(crate) fn onboarding_summary(&self) -> (&'static str, &'static str, usize, usize, usize) {
        (
            self.structure.status,
            self.status,
            self.summary.blocking,
            self.summary.advisory,
            self.summary.inactive,
        )
    }

    pub(super) fn findings_by_action(
        &self,
        action_kind: &'static str,
    ) -> Vec<&ProjectReviewFinding> {
        self.findings
            .iter()
            .filter(|finding| {
                finding.action_kind == action_kind
                    && !matches!(finding.state, "unchanged" | "resolved")
            })
            .take(4)
            .collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewStructure {
    pub(super) status: &'static str,
    pub(super) files_checked: usize,
    pub(super) dirs_checked: usize,
    pub(super) violations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewSummary {
    pub(super) blocking: usize,
    pub(super) advisory: usize,
    pub(super) inactive: usize,
    pub(super) informational: usize,
    pub(super) omitted_noise: usize,
}

impl ProjectReviewSummary {
    fn from_findings(findings: &[ProjectReviewFinding], omitted_noise: usize) -> Self {
        Self {
            blocking: count_severity(findings, "blocking"),
            advisory: count_severity(findings, "advisory"),
            inactive: count_severity(findings, "inactive"),
            informational: count_severity(findings, "informational"),
            omitted_noise,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewFinding {
    pub(super) id: String,
    pub(super) fingerprint: String,
    pub(super) state: &'static str,
    pub(super) category: &'static str,
    pub(super) severity: &'static str,
    pub(super) action_kind: &'static str,
    pub(super) title: String,
    pub(super) detail: String,
    pub(super) command: &'static str,
    pub(super) source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewContentGaps {
    pub(super) diagnostics: usize,
    pub(super) safe_fixes: usize,
    pub(super) missing_relations: usize,
    pub(super) unresolved_repository_references: usize,
    requirements_traceability: usize,
    computed_checks: usize,
}

impl From<AgentQueryGapsOutput> for ProjectReviewContentGaps {
    fn from(value: AgentQueryGapsOutput) -> Self {
        Self {
            diagnostics: value.diagnostics,
            safe_fixes: value.safe_fixes,
            missing_relations: value.missing_relations,
            unresolved_repository_references: value.unresolved_repository_references,
            requirements_traceability: value.requirements_traceability,
            computed_checks: value.computed_checks,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct ProjectReviewOmission {
    category: &'static str,
    reason: &'static str,
    command: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ProjectReviewAction {
    priority: u32,
    pub(super) action: String,
    pub(super) command: String,
}

#[derive(Debug, Clone, Serialize)]
struct ProjectReviewAgentReport {
    schema: &'static str,
    status: &'static str,
    project_root: String,
    summary: ProjectReviewSummary,
    findings: Vec<ProjectReviewFinding>,
    finding_history: FindingHistorySummary,
    content_gaps: ProjectReviewContentGaps,
    heatmap: ProjectReviewHeatmap,
    omitted_noise: Vec<ProjectReviewOmission>,
    next_actions: Vec<ProjectReviewAction>,
    lower_level_commands: Vec<&'static str>,
}

impl From<&ProjectReviewReport> for ProjectReviewAgentReport {
    fn from(report: &ProjectReviewReport) -> Self {
        Self {
            schema: PROJECT_REVIEW_AGENT_SCHEMA,
            status: report.status,
            project_root: report.project_root.clone(),
            summary: report.summary.clone(),
            findings: report
                .findings
                .iter()
                .filter(|finding| finding.state != "unchanged")
                .take(12)
                .cloned()
                .collect(),
            finding_history: report.finding_history.clone(),
            content_gaps: report.content_gaps.clone(),
            heatmap: report.heatmap.clone(),
            omitted_noise: report.omitted_noise.clone(),
            next_actions: report.next_actions.iter().take(6).cloned().collect(),
            lower_level_commands: report.lower_level_commands.clone(),
        }
    }
}

fn blocking_findings(violations: &[DoctorViolation]) -> Vec<ProjectReviewFinding> {
    violations
        .iter()
        .map(|violation| ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: format!("blocking:{}", violation.rule),
            category: violation_category(&violation.rule),
            severity: "blocking",
            action_kind: "fix-now",
            title: format!("Fix blocking `{}` violation", violation.rule),
            detail: format!("{}: {}", violation.path, violation.message),
            command: "assura check --format agent .",
            source: "doctor.blocking_violations",
        })
        .collect()
}

fn advisory_findings(items: &[DoctorItem]) -> Vec<ProjectReviewFinding> {
    items
        .iter()
        .map(|item| ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: format!("gap:{}", item.name),
            category: "configuration",
            severity: "advisory",
            action_kind: "configure-intentionally",
            title: format!("Review recommended gap `{}`", item.name),
            detail: item.detail.clone(),
            command: "assura doctor --format json .",
            source: "doctor.gaps",
        })
        .collect()
}

fn inactive_findings(
    items: &[DoctorItem],
    binary_custody: &DoctorItem,
) -> Vec<ProjectReviewFinding> {
    let mut findings = items
        .iter()
        .map(|item| ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: format!("inactive:{}", item.name),
            category: "configuration",
            severity: "inactive",
            action_kind: "configure-intentionally",
            title: format!("Capability `{}` is not active", item.name),
            detail: item.detail.clone(),
            command: "assura doctor --format json .",
            source: "doctor.inactive",
        })
        .collect::<Vec<_>>();

    if binary_custody.status == "inactive" {
        findings.push(ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: "inactive:binary_custody".to_string(),
            category: "configuration",
            severity: "inactive",
            action_kind: "configure-intentionally",
            title: "Binary custody pattern is not active".to_string(),
            detail: binary_custody.detail.clone(),
            command: "assura doctor --format json .",
            source: "doctor.binary_custody",
        });
    }
    findings
}

fn content_findings(gaps: &AgentQueryGapsOutput) -> Vec<ProjectReviewFinding> {
    let mut findings = Vec::new();
    if gaps.diagnostics > 0 || gaps.missing_relations > 0 {
        findings.push(ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: "content:diagnostics".to_string(),
            category: "content",
            severity: "advisory",
            action_kind: "fix-now",
            title: "Inspect actionable content diagnostics".to_string(),
            detail: format!(
                "{} diagnostic(s) and {} missing relation(s) were reported by content-query.",
                gaps.diagnostics, gaps.missing_relations
            ),
            command: "assura content agent-query diagnostics --format json .",
            source: "content.agent_query.gaps",
        });
    }
    if gaps.safe_fixes > 0 {
        findings.push(ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: "content:safe_fixes".to_string(),
            category: "content",
            severity: "advisory",
            action_kind: "fix-now",
            title: "Review available deterministic safe fixes".to_string(),
            detail: format!("{} safe fix proposal(s) are available.", gaps.safe_fixes),
            command: "assura content agent-query safe-fixes --format json .",
            source: "content.agent_query.gaps",
        });
    }
    if gaps.unresolved_repository_references > 0 {
        findings.push(ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: "content:unresolved_repository_references".to_string(),
            category: "content",
            severity: "informational",
            action_kind: "informational",
            title: "Classify unresolved repository-reference candidates".to_string(),
            detail: format!(
                "{} unresolved reference candidate(s) need filtering before becoming validation truth.",
                gaps.unresolved_repository_references
            ),
            command: "assura content agent-query unresolved-references --format json .",
            source: "content.agent_query.gaps",
        });
    }
    if gaps.requirements_traceability > 0 || gaps.computed_checks > 0 {
        findings.push(ProjectReviewFinding {
            fingerprint: String::new(),
            state: "new",
            id: "content:policy_diagnostics".to_string(),
            category: "content",
            severity: "advisory",
            action_kind: "fix-now",
            title: "Inspect configured policy diagnostics".to_string(),
            detail: format!(
                "{} requirements traceability and {} computed-check finding(s) were reported.",
                gaps.requirements_traceability, gaps.computed_checks
            ),
            command: "assura content agent-query diagnostics --format json .",
            source: "content.agent_query.gaps",
        });
    }
    findings
}

fn structure_fit_finding() -> ProjectReviewFinding {
    ProjectReviewFinding {
        fingerprint: String::new(),
        state: "new",
        id: "structure-fit:inspect-before-changing".to_string(),
        category: "structure-fit",
        severity: "informational",
        action_kind: "inspect-before-changing",
        title: "Inspect nearby project shape before adding paths".to_string(),
        detail: "Use path explanation and nearby directory shape before creating a new directory or changing .assura/config.yml."
            .to_string(),
        command: "assura explain <path> --format json",
        source: "review.structure_fit_guidance",
    }
}

fn review_next_actions(actions: &[DoctorNextAction]) -> Vec<ProjectReviewAction> {
    let mut next_actions = actions
        .iter()
        .map(|action| ProjectReviewAction {
            priority: action.priority,
            action: action.action.clone(),
            command: action.follow_up.clone(),
        })
        .collect::<Vec<_>>();
    next_actions.push(ProjectReviewAction {
        priority: (next_actions.len() + 1) as u32,
        action: "Before adding a new top-level path, inspect whether it fits the current repository contract."
            .to_string(),
        command: "assura explain <path> --format json".to_string(),
    });
    next_actions
}

fn omitted_noise_policy() -> Vec<ProjectReviewOmission> {
    vec![
        ProjectReviewOmission {
            category: "generated",
            reason: "Generated reference candidates are not promoted to compact-review blockers.",
            command: "assura content agent-query unresolved-references --format json .",
        },
        ProjectReviewOmission {
            category: "archive",
            reason: "Historical archive references require manual intent before validation policy.",
            command: "assura content agent-query unresolved-references --format json .",
        },
        ProjectReviewOmission {
            category: "log",
            reason: "Log-file references are noisy evidence and remain informational.",
            command: "assura content agent-query unresolved-references --format json .",
        },
        ProjectReviewOmission {
            category: "benchmark",
            reason: "Benchmark history references are not structure-health blockers.",
            command: "assura content agent-query unresolved-references --format json .",
        },
    ]
}

fn violation_category(rule: &str) -> &'static str {
    if rule.starts_with("content_runtime:") {
        "content"
    } else {
        "structure"
    }
}

fn count_severity(findings: &[ProjectReviewFinding], severity: &'static str) -> usize {
    findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .count()
}
