//! Structure check report formatting for CLI output.

use crate::cli::args::OutputFormat;
use crate::cli::check::StructureCheckReport;
use crate::cli::check_feedback::{render_check_feedback, CheckFeedbackFormat, FeedbackOptions};
use crate::cli::CheckCommandOptions;

/// Format a structure check report for the requested CLI output format.
pub fn format_structure_report(
    report: &StructureCheckReport,
    format: OutputFormat,
    options: &CheckCommandOptions,
) -> String {
    match format {
        OutputFormat::Text => format_structure_report_text(report),
        OutputFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
        OutputFormat::Yaml => serde_yaml::to_string(report).unwrap_or_default(),
        OutputFormat::Advice => format_feedback(report, options, CheckFeedbackFormat::Advice),
        OutputFormat::Status => format_feedback(report, options, CheckFeedbackFormat::Status),
    }
}

fn format_feedback(
    report: &StructureCheckReport,
    options: &CheckCommandOptions,
    format: CheckFeedbackFormat,
) -> String {
    render_check_feedback(
        report,
        format,
        &FeedbackOptions {
            minimum_severity: options.min_severity.clone(),
            max_issues: options.max_issues,
            warn: options.warn,
        },
    )
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

    output.push_str("\nViolations\n");
    output.push_str("----------\n");
    for violation in &report.violations {
        output.push_str(&format!(
            "{} [{}:{}] {}\n",
            violation.path.display(),
            violation.severity,
            violation.rule,
            violation.message
        ));
    }

    output
}
