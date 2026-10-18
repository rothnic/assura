//! Compact project review command built from existing Assura truth surfaces.

mod heatmap;
mod history;
mod report;
mod text;

use self::heatmap::build_project_review_heatmap;
use self::report::{
    render_project_review, ProjectReviewAction, ProjectReviewFinding, ProjectReviewReport,
};
use super::args::CheckOutputFormat;
use super::check::CheckError;
use super::content_query::context::{ContentQueryError, QueryContext};
use super::content_query::{content_gap_summary, AgentQueryGapsOutput};
use super::doctor::{
    build_project_doctor_with_structure_report, exit_code_for_check_error, DoctorNextAction,
};
use super::ExitCode;
use std::path::PathBuf;

/// Run the top-level compact project review command.
pub async fn project_review_command(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    format: CheckOutputFormat,
    base: String,
    verbose: bool,
) -> ExitCode {
    match build_project_review(path, config, requested_base(&base), true) {
        Ok(report) => {
            println!("{}", render_project_review(&report, format, verbose));
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {error}");
            error.exit_code()
        }
    }
}

pub(crate) fn build_project_review(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
    base: Option<&str>,
    persist_history: bool,
) -> Result<ProjectReviewReport, ProjectReviewError> {
    let doctor_build = build_project_doctor_with_structure_report(path.clone(), config.clone())
        .map_err(ProjectReviewError::Check)?;
    let content_gaps =
        load_content_gap_summary(path, config).map_err(ProjectReviewError::Content)?;
    let heatmap = build_project_review_heatmap(&doctor_build.structure_report, &content_gaps, base)
        .map_err(ProjectReviewError::Git)?;
    Ok(ProjectReviewReport::from_parts(
        doctor_build.doctor,
        doctor_build.configured_nonblocking_violations,
        content_gaps,
        heatmap,
        persist_history,
    ))
}

fn review_next_actions(
    actions: &[DoctorNextAction],
    findings: &[ProjectReviewFinding],
    check_status: &str,
) -> Vec<ProjectReviewAction> {
    let insertion = usize::from(check_status == "fail" && !actions.is_empty());
    let configured = findings
        .iter()
        .find_map(|finding| finding.configured_fix_action((insertion + 1) as u32));
    let has_violation = configured.is_some();
    let mut next_actions = actions
        .iter()
        .enumerate()
        .map(|(index, action)| ProjectReviewAction {
            priority: (index + 1 + usize::from(has_violation && index >= insertion)) as u32,
            action: action.action.clone(),
            command: action.follow_up.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(action) = configured {
        next_actions.insert(insertion, action);
    }
    next_actions.push(ProjectReviewAction {
        priority: (next_actions.len() + 1) as u32,
        action: "Before adding a new top-level path, inspect whether it fits the current repository contract."
            .to_string(),
        command: "assura explain <path> --format json".to_string(),
    });
    next_actions
}

fn requested_base(base: &str) -> Option<&str> {
    (base != "auto").then_some(base)
}

fn load_content_gap_summary(
    path: Option<PathBuf>,
    config: Option<PathBuf>,
) -> Result<AgentQueryGapsOutput, ContentQueryError> {
    let path = match path {
        Some(path) => path,
        None => std::env::current_dir().map_err(|error| {
            ContentQueryError::runtime(format!("failed to read current directory: {error}"))
        })?,
    };
    let context = QueryContext::load_for_path(path, config, false, false, true)?;
    Ok(content_gap_summary(&context))
}

#[derive(Debug)]
pub(crate) enum ProjectReviewError {
    Check(CheckError),
    Content(ContentQueryError),
    Git(String),
}

impl ProjectReviewError {
    fn exit_code(&self) -> ExitCode {
        match self {
            Self::Check(error) => exit_code_for_check_error(error),
            Self::Content(error) => error.exit_code,
            Self::Git(_) => ExitCode::ConfigurationError,
        }
    }
}

impl std::fmt::Display for ProjectReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Check(error) => write!(f, "{error}"),
            Self::Content(error) => write!(f, "{error}"),
            Self::Git(error) => write!(f, "{error}"),
        }
    }
}
