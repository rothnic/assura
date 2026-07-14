//! Agent-oriented project doctor and path explanation renderers.

use super::doctor::{DoctorItem, DoctorNextAction, ProjectDoctorReport, DOCTOR_AGENT_SCHEMA};
use crate::cli::check::{
    PathExplainNextAction, PathExplainReport, PathExplainRules, PathExplainScope, PathExplainSkip,
};
use serde::Serialize;

const EXPLAIN_AGENT_SCHEMA: &str = "assura.path-explain.agent.v1";

#[derive(Debug, Clone, Serialize)]
struct AgentDoctorReport {
    schema: &'static str,
    project_root: String,
    check_status: &'static str,
    inactive: Vec<DoctorItem>,
    gaps: Vec<DoctorItem>,
    blocking_violations: Vec<super::doctor::DoctorViolation>,
    next_actions: Vec<DoctorNextAction>,
    follow_up_surfaces: Vec<&'static str>,
}

impl From<&ProjectDoctorReport> for AgentDoctorReport {
    fn from(report: &ProjectDoctorReport) -> Self {
        Self {
            schema: DOCTOR_AGENT_SCHEMA,
            project_root: report.project_root.clone(),
            check_status: report.check.status,
            inactive: report.inactive.clone(),
            gaps: report.gaps.clone(),
            blocking_violations: report.blocking_violations.clone(),
            next_actions: report.next_actions.clone(),
            follow_up_surfaces: vec![
                "assura check --format json .",
                "assura explain <path> --format json",
                ".assura/onboarding/agent-next.md",
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct AgentExplainReport {
    schema: &'static str,
    relative_path: String,
    kind: &'static str,
    excluded: bool,
    applied_scopes: Vec<PathExplainScope>,
    effective_rules: PathExplainRules,
    matched_file_patterns: Vec<crate::cli::check::PathExplainFilePatternRule>,
    skipped_checks: Vec<PathExplainSkip>,
    next_actions: Vec<PathExplainNextAction>,
}

impl From<&PathExplainReport> for AgentExplainReport {
    fn from(report: &PathExplainReport) -> Self {
        Self {
            schema: EXPLAIN_AGENT_SCHEMA,
            relative_path: report.relative_path.clone(),
            kind: report.kind,
            excluded: report.excluded,
            applied_scopes: report.applied_scopes.clone(),
            effective_rules: report.effective_rules.clone(),
            matched_file_patterns: report.matched_file_patterns.clone(),
            skipped_checks: report.skipped_checks.clone(),
            next_actions: report.next_actions.clone(),
        }
    }
}

pub(super) fn render_agent_doctor(report: &ProjectDoctorReport) -> String {
    serde_json::to_string_pretty(&AgentDoctorReport::from(report)).unwrap_or_default()
}

pub(super) fn render_agent_explain(report: &PathExplainReport) -> String {
    serde_json::to_string_pretty(&AgentExplainReport::from(report)).unwrap_or_default()
}
