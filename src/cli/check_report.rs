//! Structure check report formatting for CLI output.

use crate::cli::args::{AgentTarget, CheckOutputFormat};
use crate::cli::check::StructureCheckReport;
use crate::cli::check_feedback::{
    format_text_report, render_agent_feedback, render_check_feedback, render_codex_agent_feedback,
    CheckFeedbackFormat, FeedbackOptions,
};
use crate::cli::CheckCommandOptions;

/// Format a structure check report for the requested CLI output format.
pub fn format_structure_report(
    report: &StructureCheckReport,
    format: CheckOutputFormat,
    options: &CheckCommandOptions,
) -> String {
    match format {
        CheckOutputFormat::Text => format_text_report(report),
        CheckOutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        CheckOutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        CheckOutputFormat::Advice => format_feedback(report, options, CheckFeedbackFormat::Advice),
        CheckOutputFormat::Status => format_feedback(report, options, CheckFeedbackFormat::Status),
        CheckOutputFormat::Agent if options.agent == AgentTarget::Codex => {
            render_codex_agent_feedback([report], &feedback_options(options))
        }
        CheckOutputFormat::Agent => render_agent_feedback([report], &feedback_options(options)),
    }
}

fn format_feedback(
    report: &StructureCheckReport,
    options: &CheckCommandOptions,
    format: CheckFeedbackFormat,
) -> String {
    render_check_feedback(report, format, &feedback_options(options))
}

fn feedback_options(options: &CheckCommandOptions) -> FeedbackOptions {
    FeedbackOptions {
        minimum_severity: options.min_severity.clone(),
        max_issues: options.max_issues,
        warn: options.warn,
    }
}
