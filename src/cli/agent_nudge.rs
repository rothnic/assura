//! Bounded event-aware nudges for local coding agents.

#[path = "agent_nudge_helpers.rs"]
mod helpers;
#[path = "agent_nudge_log.rs"]
mod log;

use super::{AgentNudgeEvent, AgentNudgeTarget, ExitCode, OutputFormat};
use crate::cli::check::{StructureCheckReport, StructureViolation};
use crate::daemon::{DaemonAffectedReferences, DaemonHealth, LocalDaemonCore};
use helpers::{
    agent_name, category_for_rule, event_name, event_policy, health_state_name,
    meets_minimum_severity, path_string, performance_sensitive_path, quote_path, severity_static,
    suggested_check_command, suggested_command, unique,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Options for the shared agent nudge surface.
pub struct AgentNudgeOptions {
    /// Project root or path to inspect.
    pub path: Option<PathBuf>,
    /// Event that may receive a nudge.
    pub event: AgentNudgeEvent,
    /// Changed paths relevant to the event.
    pub changed_paths: Vec<PathBuf>,
    /// Target agent host label.
    pub agent: AgentNudgeTarget,
    /// Minimum finding severity to include.
    pub min_severity: String,
    /// Maximum finding nudges to include.
    pub max_issues: usize,
    /// Maximum repository-reference edges to inspect per changed path.
    pub reference_limit: usize,
    /// Output format.
    pub format: OutputFormat,
}

/// Run the shared agent nudge command.
pub async fn agent_nudge_command(options: AgentNudgeOptions, config: Option<PathBuf>) -> ExitCode {
    match build_agent_nudge(options, config) {
        Ok(output) => {
            if let Err(error) = log::maybe_write(&output.project_root, &output.output) {
                eprintln!("Warning: failed to write Assura nudge log: {error}");
            }
            println!("{}", output.render());
            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::RuntimeError
        }
    }
}

fn build_agent_nudge(
    options: AgentNudgeOptions,
    config: Option<PathBuf>,
) -> Result<RenderedNudge, String> {
    let project_path = match options.path.clone() {
        Some(path) => path,
        None => std::env::current_dir().map_err(|error| error.to_string())?,
    };
    let agent_fallback_command = suggested_command(&project_path, options.agent);
    let mut nudges = Vec::new();
    let mut changed_path_checks = Vec::new();
    let mut reference_contexts = Vec::new();
    let mut omitted = 0usize;

    let mut core = match LocalDaemonCore::load(project_path.clone(), config.clone()) {
        Ok(core) => Some(core),
        Err(error) => {
            let config_path = config.unwrap_or_else(|| project_path.join(".assura/config.yml"));
            let health =
                DaemonHealth::unavailable(project_path.clone(), config_path, error.to_string());
            nudges.push(NudgeItem::daemon_unavailable(
                &health,
                &agent_fallback_command,
            ));
            None
        }
    };

    let health = if let Some(core) = core.as_ref() {
        core.health()
    } else {
        nudges
            .iter()
            .find_map(|nudge| nudge.daemon_health.clone())
            .unwrap_or_else(|| {
                DaemonHealth::unavailable(
                    project_path.clone(),
                    project_path.join(".assura/config.yml"),
                    "daemon state unavailable",
                )
            })
    };

    if let Some(core) = core.as_mut() {
        if options.event != AgentNudgeEvent::SessionStart {
            let changed_path_limit = options.max_issues.max(1);
            omitted += options
                .changed_paths
                .len()
                .saturating_sub(changed_path_limit);
            for changed_path in options.changed_paths.iter().take(changed_path_limit) {
                match core.check_changed_path(changed_path.clone()) {
                    Ok(report) => {
                        changed_path_checks.push(ChangedPathCheck::from_report(&report));
                        let mut findings = finding_nudges(
                            &report,
                            &options.min_severity,
                            options.max_issues.saturating_sub(nudges.len()),
                            &project_path,
                            options.agent,
                        );
                        omitted += report.violations.len().saturating_sub(findings.shown_count);
                        nudges.append(&mut findings.nudges);
                    }
                    Err(error) => nudges.push(NudgeItem::daemon_error(
                        "daemon_changed_path",
                        changed_path,
                        &error.to_string(),
                        &core.health(),
                    )),
                }

                if options.reference_limit > 0 {
                    if let Ok(context) = core
                        .changed_source_references(changed_path.clone(), options.reference_limit)
                    {
                        push_reference_context(
                            &mut nudges,
                            &mut reference_contexts,
                            context,
                            options.max_issues,
                        );
                    }
                    if let Ok(context) = core
                        .changed_target_references(changed_path.clone(), options.reference_limit)
                    {
                        push_reference_context(
                            &mut nudges,
                            &mut reference_contexts,
                            context,
                            options.max_issues,
                        );
                    }
                }
            }
        }
    }

    for path in &options.changed_paths {
        if performance_sensitive_path(path) && nudges.len() < options.max_issues {
            nudges.push(NudgeItem::performance_gate(path));
        }
    }

    let affected_paths = unique(
        nudges
            .iter()
            .filter_map(|nudge| nudge.path.clone())
            .collect::<Vec<_>>(),
    );
    let affected_rules = unique(
        nudges
            .iter()
            .filter_map(|nudge| nudge.rule.clone())
            .collect::<Vec<_>>(),
    );
    let should_inject = nudges.iter().any(|nudge| nudge.inject);
    let event = event_name(options.event);
    let project_root_for_log = health.project_root.clone();
    let output = AgentNudgeOutput {
        schema: "assura.agent-nudge.v1",
        target_agent: agent_name(options.agent),
        event,
        event_policy: event_policy(options.event, !options.changed_paths.is_empty()),
        cache_policy: CachePolicy {
            stable_by_default: true,
            volatile_fields: Vec::new(),
            default_detail: "bounded summary; use suggested commands for full diagnostics",
        },
        daemon: DaemonNudgeHealth {
            state: health_state_name(health.state),
            reason: health.reason.clone(),
            fallback_command: agent_fallback_command.clone(),
            status_command: format!(
                "assura daemon status --format json {}",
                quote_path(&health.project_root)
            ),
            doctor_command: format!(
                "assura daemon doctor --format json {}",
                quote_path(&health.project_root)
            ),
        },
        summary: NudgeSummary {
            should_inject,
            nudge_count: nudges.len(),
            omitted_count: omitted,
            changed_path_count: options.changed_paths.len(),
            affected_paths,
            affected_rules,
            suggested_command: agent_fallback_command,
        },
        changed_path_checks,
        reference_contexts,
        nudges,
    };
    Ok(RenderedNudge {
        output,
        format: options.format,
        project_root: project_root_for_log,
    })
}

fn finding_nudges(
    report: &StructureCheckReport,
    min_severity: &str,
    remaining: usize,
    project_path: &Path,
    agent: AgentNudgeTarget,
) -> Findings {
    let mut violations = report
        .violations
        .iter()
        .filter(|violation| meets_minimum_severity(&violation.severity, min_severity))
        .collect::<Vec<_>>();
    violations.sort_by(|left, right| {
        right
            .severity_rank()
            .cmp(&left.severity_rank())
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.rule.cmp(&right.rule))
    });
    let shown_count = violations.len().min(remaining);
    let nudges = violations
        .into_iter()
        .take(remaining)
        .map(|violation| {
            let command = suggested_check_command(project_path, agent, &violation.severity, 5);
            NudgeItem::from_violation(violation, command)
        })
        .collect();
    Findings {
        nudges,
        shown_count,
    }
}

fn push_reference_context(
    nudges: &mut Vec<NudgeItem>,
    contexts: &mut Vec<ReferenceContext>,
    context: DaemonAffectedReferences,
    max_items: usize,
) {
    if nudges.len() + contexts.len() >= max_items {
        return;
    }
    if context.bounds.returned > 0 && nudges.len() < max_items {
        nudges.push(NudgeItem::reference_context(&context));
    }
    if nudges.len() + contexts.len() < max_items {
        contexts.push(ReferenceContext {
            mode: context.mode,
            path: path_string(&context.path),
            returned: context.bounds.returned,
            truncated: context.bounds.truncated,
            suggested_command: format!(
                "assura daemon references --{} {} --format json",
                context.mode,
                quote_path(&context.path)
            ),
        });
    }
}

#[derive(Debug, Serialize)]
struct AgentNudgeOutput {
    schema: &'static str,
    target_agent: &'static str,
    event: &'static str,
    event_policy: EventPolicy,
    cache_policy: CachePolicy,
    daemon: DaemonNudgeHealth,
    summary: NudgeSummary,
    changed_path_checks: Vec<ChangedPathCheck>,
    reference_contexts: Vec<ReferenceContext>,
    nudges: Vec<NudgeItem>,
}

#[derive(Debug, Serialize)]
struct EventPolicy {
    timing: &'static str,
    inject_when: &'static str,
    changed_paths_required: bool,
}

#[derive(Debug, Serialize)]
struct CachePolicy {
    stable_by_default: bool,
    volatile_fields: Vec<&'static str>,
    default_detail: &'static str,
}

#[derive(Debug, Serialize)]
struct DaemonNudgeHealth {
    state: &'static str,
    reason: String,
    fallback_command: String,
    status_command: String,
    doctor_command: String,
}

#[derive(Debug, Serialize)]
struct NudgeSummary {
    should_inject: bool,
    nudge_count: usize,
    omitted_count: usize,
    changed_path_count: usize,
    affected_paths: Vec<String>,
    affected_rules: Vec<String>,
    suggested_command: String,
}

#[derive(Debug, Serialize)]
struct ChangedPathCheck {
    path: String,
    success: bool,
    violation_count: usize,
}

impl ChangedPathCheck {
    fn from_report(report: &StructureCheckReport) -> Self {
        Self {
            path: relative_path_string(&report.checked_path, &report.project_root),
            success: report.success,
            violation_count: report.violations.len(),
        }
    }
}

#[derive(Debug, Serialize)]
struct ReferenceContext {
    mode: &'static str,
    path: String,
    returned: usize,
    truncated: bool,
    suggested_command: String,
}

#[derive(Debug, Serialize)]
struct NudgeItem {
    category: &'static str,
    path: Option<String>,
    rule: Option<String>,
    severity: &'static str,
    message: String,
    suggested_command: String,
    inject: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    daemon_health: Option<DaemonHealth>,
}

impl NudgeItem {
    fn from_violation(violation: &StructureViolation, suggested_command: String) -> Self {
        let category = category_for_rule(&violation.rule);
        Self {
            category,
            path: Some(path_string(&violation.path)),
            rule: Some(violation.rule.clone()),
            severity: severity_static(&violation.severity),
            message: violation.message.clone(),
            suggested_command,
            inject: true,
            daemon_health: None,
        }
    }

    fn daemon_unavailable(health: &DaemonHealth, agent_fallback_command: &str) -> Self {
        Self {
            category: "daemon",
            path: None,
            rule: Some("daemon_health".to_string()),
            severity: "medium",
            message: health.reason.clone(),
            suggested_command: agent_fallback_command.to_string(),
            inject: true,
            daemon_health: Some(health.clone()),
        }
    }

    fn daemon_error(rule: &'static str, path: &Path, message: &str, health: &DaemonHealth) -> Self {
        Self {
            category: "daemon",
            path: Some(path_string(path)),
            rule: Some(rule.to_string()),
            severity: "medium",
            message: message.to_string(),
            suggested_command: health.fallback_command.clone(),
            inject: true,
            daemon_health: Some(health.clone()),
        }
    }

    fn reference_context(context: &DaemonAffectedReferences) -> Self {
        Self {
            category: "reference",
            path: Some(path_string(&context.path)),
            rule: Some("repository_reference_context".to_string()),
            severity: "low",
            message: format!(
                "{} repository reference(s) may be affected for changed {} path",
                context.bounds.returned, context.mode
            ),
            suggested_command: format!(
                "assura daemon references --{} {} --format json",
                context.mode,
                quote_path(&context.path)
            ),
            inject: true,
            daemon_health: None,
        }
    }

    fn performance_gate(path: &Path) -> Self {
        Self {
            category: "performance",
            path: Some(path_string(path)),
            rule: Some("performance_no_slower".to_string()),
            severity: "high",
            message: "Changed path can affect the LS-Lint no-slower beta gate.".to_string(),
            suggested_command: "cargo xtask performance-no-slower".to_string(),
            inject: true,
            daemon_health: None,
        }
    }
}

struct Findings {
    nudges: Vec<NudgeItem>,
    shown_count: usize,
}

struct RenderedNudge {
    output: AgentNudgeOutput,
    format: OutputFormat,
    project_root: PathBuf,
}

impl RenderedNudge {
    fn render(&self) -> String {
        match self.format {
            OutputFormat::Json => serde_json::to_string_pretty(&self.output).unwrap_or_default(),
            OutputFormat::Yaml => serde_yaml::to_string(&self.output).unwrap_or_default(),
            OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => {
                self.output.render_text()
            }
        }
    }
}

impl AgentNudgeOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![
            format!("Assura nudge: {}", self.event),
            format!("target_agent={}", self.target_agent),
            format!("daemon_state={}", self.daemon.state),
            format!("should_inject={}", self.summary.should_inject),
            format!("suggested_command={}", self.summary.suggested_command),
        ];
        for nudge in &self.nudges {
            lines.push(format!(
                "- {} {} {}",
                nudge.category,
                nudge.path.as_deref().unwrap_or("-"),
                nudge.rule.as_deref().unwrap_or("-")
            ));
            lines.push(format!("  severity={}", nudge.severity));
            lines.push(format!("  message={}", nudge.message));
            lines.push(format!("  next={}", nudge.suggested_command));
        }
        lines.join("\n")
    }
}

fn relative_path_string(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(path_string)
        .unwrap_or_else(|_| path_string(path))
}
