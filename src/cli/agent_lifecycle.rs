//! Shared lifecycle profiles for agent-ready onboarding surfaces.

use super::{AgentContentTemplate, AgentIntegrationTarget};
use serde::Serialize;
use std::path::Path;

/// One recommended lifecycle profile for an agent-ready project.
#[derive(Clone, Serialize)]
pub(super) struct LifecycleProfile {
    /// Stable lifecycle profile name.
    pub(super) name: &'static str,
    /// Lifecycle mode: nudge, warn, or gate.
    pub(super) mode: &'static str,
    /// Where this profile should run.
    pub(super) trigger: &'static str,
    /// Whether this profile can block the caller.
    pub(super) blocking: bool,
    /// Whether setup has side effects outside the project checkout.
    pub(super) side_effects: &'static str,
    /// Whether generated assets can be removed through Assura.
    pub(super) reversible: bool,
    /// Concrete command using the current shared Assura surface.
    pub(super) command: String,
    /// Follow-up command for checking setup or diagnostics.
    pub(super) follow_up: String,
}

/// Ranked next action for agent-facing onboarding output.
#[derive(Clone, Serialize)]
pub(super) struct RankedNextAction {
    /// Stable display priority.
    pub(super) priority: u32,
    /// Short action text.
    pub(super) action: &'static str,
    /// Why this action is recommended.
    pub(super) reason: &'static str,
    /// Paths the action applies to when known.
    pub(super) affected_paths: Vec<&'static str>,
    /// Concrete command or file to inspect next.
    pub(super) follow_up: String,
}

/// Build the default nudge, warn, and gate lifecycle profiles.
pub(super) fn lifecycle_profiles(
    project_root: &Path,
    integration_target: Option<AgentIntegrationTarget>,
) -> Vec<LifecycleProfile> {
    let quoted_root = quote_path(project_root);
    let agent = integration_target.map(|target| target.as_str());
    let agent_arg = agent
        .map(|value| format!(" --agent {value}"))
        .unwrap_or_default();
    let check_agent_arg = if agent == Some("codex") {
        " --agent codex"
    } else {
        ""
    };

    vec![
        LifecycleProfile {
            name: "agent-working-loop",
            mode: "nudge",
            trigger: "session start, before-tool, after-tool, file-read, idle, or recovery",
            blocking: false,
            side_effects: "none unless a reviewable integration bundle is explicitly installed",
            reversible: true,
            command: format!(
                "assura agent nudge{agent_arg} --event before-tool --changed <path> --format json {quoted_root}"
            ),
            follow_up: integration_target
                .map(|target| {
                    format!(
                        "assura agent integration doctor {} --format json {quoted_root}",
                        target.as_str()
                    )
                })
                .unwrap_or_else(|| {
                    format!("assura check --format agent --warn --max-issues 5 {quoted_root}")
                }),
        },
        LifecycleProfile {
            name: "pre-commit-warning",
            mode: "warn",
            trigger: "before local commit or during draft agent work",
            blocking: false,
            side_effects: "reports feedback only; use assura hooks install explicitly for git hooks",
            reversible: true,
            command: format!(
                "assura check --format agent{check_agent_arg} --warn --min-severity low --max-issues 10 {quoted_root}"
            ),
            follow_up: "assura hooks status".to_string(),
        },
        LifecycleProfile {
            name: "pre-push-or-ci-gate",
            mode: "gate",
            trigger: "before push, merge, or CI required check",
            blocking: true,
            side_effects: "standard nonzero exit on medium, high, or critical findings",
            reversible: true,
            command: format!(
                "assura check --format agent{check_agent_arg} --min-severity medium --max-issues 20 {quoted_root}"
            ),
            follow_up: "assura check --format json .".to_string(),
        },
    ]
}

/// Build ranked onboarding next actions from detected setup state.
pub(super) fn ranked_next_actions(
    integration_target: Option<AgentIntegrationTarget>,
    content_template: AgentContentTemplate,
) -> Vec<RankedNextAction> {
    let mut actions = vec![RankedNextAction {
        priority: 1,
        action: "Read the onboarding handoff",
        reason: "The broad baseline is active but project specialization is still pending.",
        affected_paths: vec![".assura/onboarding/agent-next.md"],
        follow_up: ".assura/onboarding/agent-next.md".to_string(),
    }];
    let generic_base_priority = 2;

    actions.extend([
        RankedNextAction {
            priority: generic_base_priority,
            action: "Ask remaining specialization questions",
            reason:
                "Assura should not invent language, layout, naming, hook, or domain conventions.",
            affected_paths: vec![".assura/onboarding/questions.md"],
            follow_up: ".assura/onboarding/questions.md".to_string(),
        },
        RankedNextAction {
            priority: generic_base_priority + 1,
            action: "Use advisory feedback while drafting",
            reason: "Warn mode reports guidance without blocking normal agent work.",
            affected_paths: vec![".assura/config.yml"],
            follow_up: "assura check --format agent --warn --max-issues 5 .".to_string(),
        },
        RankedNextAction {
            priority: generic_base_priority + 2,
            action: "Use gate feedback before push or CI",
            reason: "Gate mode preserves the configured severity contract before merge.",
            affected_paths: vec![".assura/config.yml"],
            follow_up: "assura check --format agent --min-severity medium .".to_string(),
        },
    ]);

    if let Some(target) = integration_target {
        actions.push(RankedNextAction {
            priority: generic_base_priority + 3,
            action: "Review host-agent integration bundle",
            reason: "Host wiring remains manual opt-in and should be reviewed before use.",
            affected_paths: vec![".assura/integrations/"],
            follow_up: format!("assura agent integration doctor {} .", target.as_str()),
        });
    }

    if !content_template.activates_content() {
        actions.push(RankedNextAction {
            priority: generic_base_priority + 4,
            action: "Choose whether to activate content models",
            reason: "Content facts stay inactive until a template is selected deliberately.",
            affected_paths: vec![".assura/onboarding/questions.md"],
            follow_up: "assura agent onboard . --content-template agent-project".to_string(),
        });
    }

    actions
}

fn quote_path(path: &Path) -> String {
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
