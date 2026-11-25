//! Shared state and path projection for host activation adapters.

use super::bundle::{
    host_approval_required, path_string, ActivationReport, FileAction, IntegrationBundle,
};
use crate::cli::AgentIntegrationTarget;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct ActivationMutation {
    pub(super) changed: bool,
    pub(super) files: Vec<FileAction>,
    pub(super) state: ActivationReport,
}

pub(super) fn json_config_path(bundle: &IntegrationBundle) -> PathBuf {
    match bundle.agent {
        AgentIntegrationTarget::Codex => bundle.project_root.join(".codex/hooks.json"),
        AgentIntegrationTarget::Claude => bundle.project_root.join(".claude/settings.json"),
        _ => unreachable!("JSON activation only applies to Codex and Claude"),
    }
}

pub(super) fn supported_events(agent: AgentIntegrationTarget) -> Vec<&'static str> {
    match agent {
        AgentIntegrationTarget::Codex => vec![
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
        ],
        AgentIntegrationTarget::Claude => vec![
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
        ],
        AgentIntegrationTarget::Opencode => vec![
            "session.created",
            "session.idle",
            "session.error",
            "tool.execute.after",
        ],
        AgentIntegrationTarget::Pi => {
            vec!["session_start", "before_agent_start", "tool_result"]
        }
    }
}

pub(super) fn projected_state(bundle: &IntegrationBundle, activated: bool) -> ActivationReport {
    let config_path = match bundle.agent {
        AgentIntegrationTarget::Codex | AgentIntegrationTarget::Claude => {
            path_string(&json_config_path(bundle))
        }
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => bundle
            .activation_files()
            .first()
            .map(|file| path_string(&file.path))
            .unwrap_or_default(),
    };
    ActivationReport {
        generated: bundle.is_installed(),
        activated,
        verified: activated,
        conflicted: false,
        config_path,
        events: supported_events(bundle.agent),
        verification_scope: "managed files and project host configuration",
        host_approval_required: host_approval_required(bundle.agent),
    }
}

pub(super) fn remove_empty_parents(path: &Path, root: &Path) {
    let mut parent = path.parent();
    while let Some(directory) = parent {
        if directory == root || !directory.starts_with(root) || fs::remove_dir(directory).is_err() {
            break;
        }
        parent = directory.parent();
    }
}
