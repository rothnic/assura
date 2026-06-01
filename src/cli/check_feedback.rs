//! Guided check output rendered from structure check reports.

use super::StructureCheckReport;
use serde::Serialize;
use std::path::Path;

/// Guided check output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckFeedbackFormat {
    /// Human-readable guidance.
    Advice,
    /// One-line status for compact tool output.
    Status,
}

/// Options controlling which feedback messages are rendered.
#[derive(Debug, Clone, Default)]
pub struct FeedbackOptions {
    /// Minimum severity required before a violation becomes a rendered message.
    pub minimum_severity: Option<String>,
    /// Maximum number of feedback messages to render.
    pub max_issues: Option<usize>,
    /// Whether validation failures are reported without a failing exit code.
    pub warn: bool,
}

/// Check-facing feedback derived from a structure check report.
#[derive(Debug, Serialize)]
pub struct CheckFeedback {
    status: &'static str,
    summary: String,
    violation_count: usize,
    suppressed_violation_count: usize,
    minimum_severity: Option<String>,
    affected_rules: Vec<String>,
    messages: Vec<FeedbackMessage>,
    metrics: FeedbackMetrics,
}

/// One actionable feedback item.
#[derive(Debug, Serialize)]
struct FeedbackMessage {
    path: String,
    rule: String,
    severity: String,
    problem: String,
    corrective_context: String,
    guidance: Vec<&'static str>,
    references: Vec<&'static str>,
}

/// Simple local metrics for feedback usefulness analysis.
#[derive(Debug, Serialize)]
struct FeedbackMetrics {
    structural_violations: usize,
    affected_rules: Vec<String>,
    affected_paths: Vec<String>,
    feedback_count: usize,
}

#[derive(Debug, Serialize)]
struct AgentFeedbackOutput {
    schema: &'static str,
    source: AgentFeedbackSource,
    filter: AgentFeedbackFilter,
    blocking: bool,
    feedback: Vec<CheckFeedback>,
}

#[derive(Debug, Serialize)]
struct AgentFeedbackSource {
    command: &'static str,
}

#[derive(Debug, Serialize)]
struct AgentFeedbackFilter {
    #[serde(rename = "minimumSeverity")]
    minimum_severity: Option<String>,
    #[serde(rename = "maxIssues")]
    max_issues: Option<usize>,
}

#[derive(Debug, Serialize)]
struct CodexHookOutput {
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: CodexHookSpecificOutput,
}

#[derive(Debug, Serialize)]
struct CodexHookSpecificOutput {
    #[serde(rename = "hookEventName")]
    hook_event_name: &'static str,
    #[serde(rename = "additionalContext")]
    additional_context: String,
}

const DEFAULT_REFERENCES: &[&str] = &["AGENTS.md", ".agents/skills/", ".assura/config.yml"];

/// Create check-facing feedback from a raw structure report.
pub fn create_check_feedback(
    report: &StructureCheckReport,
    options: &FeedbackOptions,
) -> CheckFeedback {
    let filtered = report
        .violations
        .iter()
        .filter(|violation| {
            meets_minimum_severity(&violation.severity, options.minimum_severity.as_deref())
        })
        .collect::<Vec<_>>();
    let shown = if let Some(max_issues) = options.max_issues {
        filtered.into_iter().take(max_issues).collect::<Vec<_>>()
    } else {
        filtered
    };
    let affected_rules = unique(
        shown
            .iter()
            .map(|violation| violation.rule.clone())
            .collect(),
    );
    let affected_paths = unique(
        shown
            .iter()
            .map(|violation| path_to_string(&violation.path))
            .collect(),
    );
    let suppressed_violation_count = report.violations.len().saturating_sub(shown.len());

    if report.success {
        return CheckFeedback {
            status: "pass",
            summary: format!(
                "Assura passed for {}; no guided output is needed.",
                report.checked_path.display()
            ),
            violation_count: 0,
            suppressed_violation_count: 0,
            minimum_severity: options.minimum_severity.clone(),
            affected_rules: Vec::new(),
            messages: Vec::new(),
            metrics: FeedbackMetrics {
                structural_violations: 0,
                affected_rules: Vec::new(),
                affected_paths: Vec::new(),
                feedback_count: 0,
            },
        };
    }

    let messages = shown
        .iter()
        .map(|violation| FeedbackMessage {
            path: path_to_string(&violation.path),
            rule: violation.rule.clone(),
            severity: violation.severity.clone(),
            problem: violation.message.clone(),
            corrective_context: violation.corrective_context.clone(),
            guidance: guidance_for_rule(&violation.rule),
            references: DEFAULT_REFERENCES.to_vec(),
        })
        .collect::<Vec<_>>();

    CheckFeedback {
        status: "fail",
        summary: summarize_check_feedback(
            report,
            messages.len(),
            affected_rules.len(),
            suppressed_violation_count,
            options,
        ),
        violation_count: report.violations.len(),
        suppressed_violation_count,
        minimum_severity: options.minimum_severity.clone(),
        affected_rules: affected_rules.clone(),
        messages,
        metrics: FeedbackMetrics {
            structural_violations: report.violations.len(),
            affected_rules,
            affected_paths,
            feedback_count: shown.len(),
        },
    }
}

/// Render check-facing feedback.
pub fn render_check_feedback(
    report: &StructureCheckReport,
    format: CheckFeedbackFormat,
    options: &FeedbackOptions,
) -> String {
    let feedback = create_check_feedback(report, options);
    match format {
        CheckFeedbackFormat::Status => render_status_line(&feedback),
        CheckFeedbackFormat::Advice => render_text(&feedback),
    }
}

/// Render one or more check reports as stable agent feedback JSON.
pub fn render_agent_feedback<'a>(
    reports: impl IntoIterator<Item = &'a StructureCheckReport>,
    options: &FeedbackOptions,
) -> String {
    let feedback = reports
        .into_iter()
        .map(|report| create_check_feedback(report, options))
        .collect::<Vec<_>>();

    serde_json::to_string(&AgentFeedbackOutput {
        schema: "assura.agent-feedback.v1",
        source: AgentFeedbackSource {
            command: "assura check --format agent",
        },
        filter: AgentFeedbackFilter {
            minimum_severity: options.minimum_severity.clone(),
            max_issues: options.max_issues,
        },
        blocking: !options.warn,
        feedback,
    })
    .unwrap_or_default()
}

/// Render stable agent feedback in Codex `UserPromptSubmit` hook JSON.
pub fn render_codex_agent_feedback<'a>(
    reports: impl IntoIterator<Item = &'a StructureCheckReport>,
    options: &FeedbackOptions,
) -> String {
    let feedback = reports
        .into_iter()
        .map(|report| create_check_feedback(report, options))
        .collect::<Vec<_>>();
    let mut lines = vec![
        "<assura-feedback>".to_string(),
        "Hook event: UserPromptSubmit".to_string(),
        "Check state: ran assura check --format agent --agent codex".to_string(),
        format!(
            "Filter: severity >= {}; max issues {}",
            options.minimum_severity.as_deref().unwrap_or("all"),
            options
                .max_issues
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unbounded".to_string())
        ),
        format!(
            "Blocking: {}",
            if options.warn {
                "no (--warn)"
            } else {
                "yes (validation failures exit 1)"
            }
        ),
        String::new(),
    ];

    for (index, feedback) in feedback.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(render_text(feedback));
    }

    lines.push("</assura-feedback>".to_string());

    serde_json::to_string(&CodexHookOutput {
        hook_specific_output: CodexHookSpecificOutput {
            hook_event_name: "UserPromptSubmit",
            additional_context: lines.join("\n"),
        },
    })
    .unwrap_or_default()
}

fn render_status_line(feedback: &CheckFeedback) -> String {
    if feedback.status == "pass" {
        return "Assura: pass; no structural feedback.".to_string();
    }

    let threshold = feedback
        .minimum_severity
        .as_ref()
        .map(|severity| format!(" at {severity}+ severity"))
        .unwrap_or_default();
    let suppressed = if feedback.suppressed_violation_count > 0 {
        format!(
            "; {} lower-priority or overflow violation(s) suppressed",
            feedback.suppressed_violation_count
        )
    } else {
        String::new()
    };

    format!(
        "Assura: {} violation(s); showing {} guided item(s){threshold}{suppressed}.",
        feedback.violation_count,
        feedback.messages.len()
    )
}

fn render_text(feedback: &CheckFeedback) -> String {
    let mut output = String::new();
    output.push_str(&render_status_line(feedback));
    output.push('\n');
    output.push_str(&feedback.summary);
    output.push('\n');

    if feedback.messages.is_empty() {
        return output;
    }

    output.push('\n');
    use std::fmt::Write as _;
    for message in &feedback.messages {
        let _ = writeln!(
            output,
            "- {} [{}:{}]",
            message.path, message.rule, message.severity
        );
        output.push_str(&format!("  Problem: {}\n", message.problem));
        let _ = writeln!(output, "  Fix: {}", message.corrective_context);
        for guidance in &message.guidance {
            output.push_str(&format!("  Next: {guidance}\n"));
        }
        output.push_str(&format!(
            "  References: {}\n",
            message.references.join(", ")
        ));
    }
    output
}

fn summarize_check_feedback(
    report: &StructureCheckReport,
    shown_message_count: usize,
    shown_rule_count: usize,
    suppressed_violation_count: usize,
    options: &FeedbackOptions,
) -> String {
    let threshold = options
        .minimum_severity
        .as_ref()
        .map(|severity| format!(" at {severity}+ severity"))
        .unwrap_or_default();
    let suppressed = if suppressed_violation_count > 0 {
        format!(" {suppressed_violation_count} violation(s) were hidden by display filters.")
    } else {
        String::new()
    };
    let exit_behavior = if options.warn {
        "This command is running with --warn, so validation failures are reported without a failing exit code."
    } else {
        "This command exits 1 for validation failures; use --warn for advisory reporting."
    };

    format!(
        "Assura found {} structural violation(s); showing {shown_message_count} guided item(s){threshold} across {shown_rule_count} rule(s).{suppressed} {exit_behavior}",
        report.violations.len(),
    )
}

fn guidance_for_rule(rule: &str) -> Vec<&'static str> {
    match rule {
        "file_naming" => vec![
            "Rename the file to match the configured file naming convention for its directory.",
            "Check `.assura/config.yml` for the effective file naming rule before editing nearby files.",
        ],
        "directory_naming" => vec![
            "Rename the directory to match the configured directory naming convention.",
            "Update imports, references, and documentation that mention the old path.",
        ],
        "unexpected_file" | "unexpected_directory" => vec![
            "Move or remove the unexpected path, or update the direct-content policy if it is intentional.",
            "Read nearby `AGENTS.md` guidance before deciding whether the path belongs in this scope.",
        ],
        "exists_count" => vec![
            "Create, remove, or move entries so the direct-count requirement is satisfied.",
            "Exact count rules are Assura policy behavior; check `.assura/config.yml` before editing.",
        ],
        _ => vec![
            "Inspect the reported path and rule, then update the file tree or policy so they agree.",
            "Prefer changing project structure before weakening policy unless the policy is stale.",
        ],
    }
}

fn meets_minimum_severity(severity: &str, minimum_severity: Option<&str>) -> bool {
    let Some(minimum_severity) = minimum_severity else {
        return true;
    };
    match (severity_rank(severity), severity_rank(minimum_severity)) {
        (Some(severity_rank), Some(minimum_rank)) => severity_rank >= minimum_rank,
        _ => severity == minimum_severity,
    }
}

fn severity_rank(severity: &str) -> Option<u8> {
    match severity.to_ascii_lowercase().as_str() {
        "low" => Some(1),
        "medium" => Some(2),
        "high" => Some(3),
        "critical" => Some(4),
        _ => None,
    }
}

fn unique(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
