//! Structure check report formatting for CLI output.

use crate::cli::args::{AgentTarget, CheckOutputFormat};
use crate::cli::check::StructureCheckReport;
use crate::cli::check_feedback::{
    render_agent_feedback, render_check_feedback, render_codex_agent_feedback, CheckFeedbackFormat,
    FeedbackOptions,
};
use crate::cli::CheckCommandOptions;

/// Format a structure check report for the requested CLI output format.
pub fn format_structure_report(
    report: &StructureCheckReport,
    format: CheckOutputFormat,
    options: &CheckCommandOptions,
) -> String {
    match format {
        CheckOutputFormat::Text => format_structure_report_text(report),
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

fn format_structure_report_text(report: &StructureCheckReport) -> String {
    let mut output = String::new();
    output.push_str("Assura structure check\n");
    output.push_str("======================\n");
    output.push_str(&format!(
        "Project root: {}\n",
        report.project_root.display()
    ));
    output.push_str(&format!("Config: {}\n", report.config_path.display()));
    output.push_str(&format!(
        "Checked path: {}\n",
        report.checked_path.display()
    ));
    output.push_str(&format!("Files checked: {}\n", report.files_checked));
    output.push_str(&format!("Directories checked: {}\n", report.dirs_checked));
    output.push_str(&format!("Violations: {}\n", report.violation_count()));

    if report.violations.is_empty() {
        output.push_str("\nAll configured structure checks passed.\n");
        return output;
    }

    use std::fmt::Write as _;
    output.push_str("\nViolations\n----------\n");
    for violation in &report.violations {
        output.push_str(&format!(
            "{} [{}:{}] {}\n",
            violation.path.display(),
            violation.severity,
            violation.rule,
            violation.message
        ));
        let _ = writeln!(output, "  Fix: {}", violation.corrective_context);
    }

    output
}
