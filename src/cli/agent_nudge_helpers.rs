//! Small helper functions for the shared agent nudge contract.

use super::EventPolicy;
use crate::cli::{AgentNudgeEvent, AgentNudgeTarget};
use crate::daemon::{DaemonHealthState, DaemonHealthState::*};
use std::path::Path;

pub(super) fn event_policy(event: AgentNudgeEvent, has_changed_paths: bool) -> EventPolicy {
    match event {
        AgentNudgeEvent::SessionStart => EventPolicy {
            timing: "session_start",
            inject_when: "only compact daemon health or recovery context",
            changed_paths_required: false,
        },
        AgentNudgeEvent::BeforeTool => EventPolicy {
            timing: "before_tool",
            inject_when: "the next tool is likely to inspect or edit affected paths",
            changed_paths_required: !has_changed_paths,
        },
        AgentNudgeEvent::AfterTool => EventPolicy {
            timing: "after_tool",
            inject_when: "changed files create Assura findings or affected-reference context",
            changed_paths_required: !has_changed_paths,
        },
        AgentNudgeEvent::FileRead => EventPolicy {
            timing: "file_read",
            inject_when: "the read path has structure, content, or reference context",
            changed_paths_required: !has_changed_paths,
        },
        AgentNudgeEvent::Recovery => EventPolicy {
            timing: "recovery",
            inject_when: "daemon state is stale, unavailable, or recent tool context needs repair",
            changed_paths_required: false,
        },
    }
}

pub(super) fn suggested_command(path: &Path, agent: AgentNudgeTarget) -> String {
    match agent {
        AgentNudgeTarget::Codex => format!(
            "assura check --format agent --agent codex --warn {}",
            quote_path(path)
        ),
        _ => format!("assura check --format agent --warn {}", quote_path(path)),
    }
}

pub(super) fn suggested_check_command(
    path: &Path,
    agent: AgentNudgeTarget,
    min_severity: &str,
    max_issues: usize,
) -> String {
    match agent {
        AgentNudgeTarget::Codex => format!(
            "assura check --format agent --agent codex --warn --min-severity {min_severity} --max-issues {max_issues} {}",
            quote_path(path)
        ),
        _ => format!(
            "assura check --format agent --warn --min-severity {min_severity} --max-issues {max_issues} {}",
            quote_path(path)
        ),
    }
}

pub(super) fn agent_name(agent: AgentNudgeTarget) -> &'static str {
    match agent {
        AgentNudgeTarget::Generic => "generic",
        AgentNudgeTarget::Codex => "codex",
        AgentNudgeTarget::Opencode => "opencode",
        AgentNudgeTarget::Claude => "claude",
        AgentNudgeTarget::Pi => "pi",
    }
}

pub(super) fn event_name(event: AgentNudgeEvent) -> &'static str {
    match event {
        AgentNudgeEvent::SessionStart => "session_start",
        AgentNudgeEvent::BeforeTool => "before_tool",
        AgentNudgeEvent::AfterTool => "after_tool",
        AgentNudgeEvent::FileRead => "file_read",
        AgentNudgeEvent::Recovery => "recovery",
    }
}

pub(super) fn health_state_name(state: DaemonHealthState) -> &'static str {
    match state {
        Warming => "warming",
        Running => "running",
        Stale => "stale",
        Degraded => "degraded",
        Unavailable => "unavailable",
        Incompatible => "incompatible",
    }
}

pub(super) fn category_for_rule(rule: &str) -> &'static str {
    if rule.starts_with("content_runtime:") {
        "content"
    } else if rule.starts_with("markdown_") {
        "markdown"
    } else if rule.starts_with("repository_reference") {
        "reference"
    } else {
        "structure"
    }
}

pub(super) fn meets_minimum_severity(severity: &str, minimum_severity: &str) -> bool {
    match (severity_rank(severity), severity_rank(minimum_severity)) {
        (Some(severity_rank), Some(minimum_rank)) => severity_rank >= minimum_rank,
        _ => severity == minimum_severity,
    }
}

pub(super) fn severity_static(severity: &str) -> &'static str {
    match severity {
        "low" => "low",
        "medium" => "medium",
        "high" => "high",
        "critical" => "critical",
        _ => "medium",
    }
}

pub(super) fn performance_sensitive_path(path: &Path) -> bool {
    let text = path_string(path);
    text.starts_with("src/cli/check")
        || text.starts_with("src/cli/performance_report")
        || text.starts_with("benches/")
        || text == "benches/history/current.json"
        || text == "xtask/src/main.rs"
        || text == ".github/workflows/ci.yml"
}

pub(super) fn unique(values: Vec<String>) -> Vec<String> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    unique
}

pub(super) fn quote_path(path: &Path) -> String {
    let value = path.display().to_string();
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        value
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub(super) fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
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
