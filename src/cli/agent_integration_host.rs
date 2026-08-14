//! Ownership-safe project-local activation for supported agent hosts.

use super::bundle::{
    file_status, host_approval_required, path_string, ActivationReport, FileAction,
    IntegrationBundle,
};
use super::host_json;
use crate::cli::AgentIntegrationTarget;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) struct ActivationMutation {
    pub(super) changed: bool,
    pub(super) files: Vec<FileAction>,
    pub(super) state: ActivationReport,
}

pub(super) fn status(bundle: &IntegrationBundle) -> Result<ActivationReport, String> {
    match bundle.agent {
        AgentIntegrationTarget::Codex | AgentIntegrationTarget::Claude => host_json::status(bundle),
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => file_status_report(bundle),
    }
}

pub(super) fn activate(
    bundle: &IntegrationBundle,
    dry_run: bool,
) -> Result<ActivationMutation, String> {
    match bundle.agent {
        AgentIntegrationTarget::Codex | AgentIntegrationTarget::Claude => {
            host_json::activate(bundle, dry_run)
        }
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => {
            activate_files(bundle, dry_run)
        }
    }
}

pub(super) fn deactivate(
    bundle: &IntegrationBundle,
    dry_run: bool,
) -> Result<ActivationMutation, String> {
    match bundle.agent {
        AgentIntegrationTarget::Codex | AgentIntegrationTarget::Claude => {
            host_json::deactivate(bundle, dry_run)
        }
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => {
            deactivate_files(bundle, dry_run)
        }
    }
}

pub(super) fn managed_paths(bundle: &IntegrationBundle) -> Vec<PathBuf> {
    match bundle.agent {
        AgentIntegrationTarget::Codex | AgentIntegrationTarget::Claude => {
            vec![host_json::config_path(bundle)]
        }
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => bundle
            .activation_files()
            .into_iter()
            .map(|file| file.path)
            .collect(),
    }
}

fn file_status_report(bundle: &IntegrationBundle) -> Result<ActivationReport, String> {
    let files = bundle.activation_files();
    let expected = files
        .first()
        .ok_or_else(|| "missing activation file".to_string())?;
    let current = file_status(&expected.path);
    let activated = current.is_some();
    let conflicted = current.as_ref().is_some_and(|status| {
        !status.managed || status.content.as_deref() != Some(expected.content.as_str())
    });
    let verified = current.as_ref().is_some_and(|status| {
        status.managed && status.content.as_deref() == Some(expected.content.as_str())
    });
    Ok(ActivationReport {
        generated: bundle.is_installed(),
        activated,
        verified,
        conflicted,
        config_path: path_string(&expected.path),
        events: supported_events(bundle.agent),
        verification_scope: "managed files and project host configuration",
        host_approval_required: host_approval_required(bundle.agent),
    })
}

fn activate_files(bundle: &IntegrationBundle, dry_run: bool) -> Result<ActivationMutation, String> {
    let mut changed = false;
    let mut actions = Vec::new();
    for file in bundle.activation_files() {
        let current = file_status(&file.path);
        if current.as_ref().is_some_and(|status| !status.managed) {
            return Err(format!(
                "refusing to overwrite non-Assura-managed file: {}",
                path_string(&file.path)
            ));
        }
        let write = current.as_ref().map_or(true, |status| {
            status.content.as_deref() != Some(file.content.as_str())
        });
        changed |= write;
        actions.push(FileAction {
            path: path_string(&file.path),
            kind: file.kind,
            action: if dry_run && write {
                "would_write"
            } else if write {
                "write"
            } else {
                "unchanged"
            },
            existed: current.is_some(),
            managed: current.as_ref().is_some_and(|status| status.managed),
        });
        if write && !dry_run {
            if let Some(parent) = file.path.parent() {
                fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            fs::write(&file.path, file.content).map_err(|error| error.to_string())?;
        }
    }
    let state = if dry_run {
        projected_state(bundle, true)
    } else {
        status(bundle)?
    };
    Ok(ActivationMutation {
        changed,
        files: actions,
        state,
    })
}

fn deactivate_files(
    bundle: &IntegrationBundle,
    dry_run: bool,
) -> Result<ActivationMutation, String> {
    let mut changed = false;
    let mut actions = Vec::new();
    for file in bundle.activation_files() {
        let current = file_status(&file.path);
        if current.as_ref().is_some_and(|status| {
            !status.managed || status.content.as_deref() != Some(file.content.as_str())
        }) {
            return Err(format!(
                "refusing to remove drifted or non-Assura-managed host activation: {}",
                path_string(&file.path)
            ));
        }
        let remove = current.is_some();
        changed |= remove;
        actions.push(FileAction {
            path: path_string(&file.path),
            kind: file.kind,
            action: if dry_run && remove {
                "would_remove"
            } else if remove {
                "remove"
            } else {
                "unchanged"
            },
            existed: current.is_some(),
            managed: current.as_ref().is_some_and(|status| status.managed),
        });
        if remove && !dry_run {
            fs::remove_file(&file.path).map_err(|error| error.to_string())?;
            remove_empty_parents(&file.path, &bundle.project_root);
        }
    }
    let state = if dry_run {
        projected_state(bundle, false)
    } else {
        status(bundle)?
    };
    Ok(ActivationMutation {
        changed,
        files: actions,
        state,
    })
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
            path_string(&host_json::config_path(bundle))
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
