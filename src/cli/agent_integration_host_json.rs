//! Ownership-safe JSON host configuration for Codex and Claude Code.

use super::bundle::{
    host_approval_required, path_string, ActivationReport, FileAction, IntegrationBundle,
};
use super::host::{projected_state, remove_empty_parents, supported_events, ActivationMutation};
use crate::cli::AgentIntegrationTarget;
use serde_json::{json, Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn status(bundle: &IntegrationBundle) -> Result<ActivationReport, String> {
    let path = config_path(bundle);
    let Some(config) = read_json(&path)? else {
        return Ok(projected_state(bundle, false));
    };
    let expected = expected_groups(bundle.agent);
    let mut assura_groups = 0usize;
    let mut exact_groups = 0usize;
    for (event, groups) in hook_groups(&config) {
        for group in groups {
            if group_references_assura(group, bundle.agent) {
                assura_groups += 1;
                if expected.iter().any(|(expected_event, expected_group)| {
                    event == *expected_event && group == expected_group
                }) {
                    exact_groups += 1;
                }
            }
        }
    }
    let activated = assura_groups > 0;
    let verified = exact_groups == expected.len() && assura_groups == expected.len();
    Ok(ActivationReport {
        generated: bundle.is_installed(),
        activated,
        verified,
        conflicted: activated && !verified,
        config_path: path_string(&path),
        events: supported_events(bundle.agent),
        verification_scope: "managed files and project host configuration",
        host_approval_required: host_approval_required(bundle.agent),
    })
}

pub(super) fn activate(
    bundle: &IntegrationBundle,
    dry_run: bool,
) -> Result<ActivationMutation, String> {
    let path = config_path(bundle);
    let before = read_json(&path)?.unwrap_or_else(|| Value::Object(Map::new()));
    let state = status(bundle)?;
    if state.conflicted {
        return Err(format!(
            "Assura host activation conflicts with unmanaged edits in {}; restore the managed hook entries or deactivate them manually",
            path_string(&path)
        ));
    }
    let mut after = before.clone();
    let hooks = object_field(&mut after, "hooks")?;
    for (event, group) in expected_groups(bundle.agent) {
        let groups = array_field(hooks, event)?;
        if !groups.iter().any(|existing| existing == &group) {
            groups.push(group);
        }
    }
    let changed = before != after;
    if changed && !dry_run {
        write_json(&path, &after)?;
    }
    let action = FileAction {
        path: path_string(&path),
        kind: "host_config",
        action: if dry_run && changed {
            "would_patch"
        } else if changed {
            "patch"
        } else {
            "unchanged"
        },
        existed: path.exists(),
        managed: true,
    };
    Ok(ActivationMutation {
        changed,
        files: vec![action],
        state: if dry_run {
            projected_state(bundle, true)
        } else {
            status(bundle)?
        },
    })
}

pub(super) fn deactivate(
    bundle: &IntegrationBundle,
    dry_run: bool,
) -> Result<ActivationMutation, String> {
    let path = config_path(bundle);
    let Some(before) = read_json(&path)? else {
        return Ok(ActivationMutation {
            changed: false,
            files: vec![FileAction {
                path: path_string(&path),
                kind: "host_config",
                action: "unchanged",
                existed: false,
                managed: false,
            }],
            state: projected_state(bundle, false),
        });
    };
    let state = status(bundle)?;
    if state.conflicted {
        return Err(format!(
            "refusing to remove drifted Assura host activation from {}; restore the managed hook entries first",
            path_string(&path)
        ));
    }
    let mut after = before.clone();
    remove_expected_groups(&mut after, bundle.agent)?;
    prune_empty_hooks(&mut after);
    let changed = before != after;
    if changed && !dry_run {
        if empty_object(&after) {
            fs::remove_file(&path).map_err(|error| error.to_string())?;
            remove_empty_parents(&path, &bundle.project_root);
        } else {
            write_json(&path, &after)?;
        }
    }
    Ok(ActivationMutation {
        changed,
        files: vec![FileAction {
            path: path_string(&path),
            kind: "host_config",
            action: if dry_run && changed {
                "would_unpatch"
            } else if changed {
                "unpatch"
            } else {
                "unchanged"
            },
            existed: true,
            managed: true,
        }],
        state: if dry_run {
            projected_state(bundle, false)
        } else {
            status(bundle)?
        },
    })
}

pub(super) fn config_path(bundle: &IntegrationBundle) -> PathBuf {
    match bundle.agent {
        AgentIntegrationTarget::Codex => bundle.project_root.join(".codex/hooks.json"),
        AgentIntegrationTarget::Claude => bundle.project_root.join(".claude/settings.json"),
        _ => unreachable!("JSON activation only applies to Codex and Claude"),
    }
}

fn expected_groups(agent: AgentIntegrationTarget) -> Vec<(&'static str, Value)> {
    let path = format!(".assura/integrations/{}/assura-hook.py", agent.as_str());
    match agent {
        AgentIntegrationTarget::Codex => {
            let command =
                format!("python3 \"$(git rev-parse --show-toplevel 2>/dev/null || pwd)/{path}\"");
            let command_windows = format!(
                "powershell.exe -NoProfile -Command \"$root = git rev-parse --show-toplevel 2>$null; if ($LASTEXITCODE -ne 0) {{ $root = (Get-Location).Path }}; py -3 (Join-Path $root '{path}')\""
            );
            [
                "SessionStart",
                "UserPromptSubmit",
                "PreToolUse",
                "PostToolUse",
            ]
            .into_iter()
            .map(|event| {
                let mut group = json!({
                    "hooks": [{
                        "type": "command",
                        "command": command,
                        "commandWindows": command_windows,
                        "timeout": 10,
                        "statusMessage": "Reviewing Assura feedback"
                    }]
                });
                if matches!(event, "PreToolUse" | "PostToolUse") {
                    group["matcher"] = Value::String("*".to_string());
                }
                (event, group)
            })
            .collect()
        }
        AgentIntegrationTarget::Claude => [
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "PostToolUseFailure",
        ]
        .into_iter()
        .map(|event| {
            let mut group = json!({
                "hooks": [{
                    "type": "command",
                    "command": "python3",
                    "args": [format!("${{CLAUDE_PROJECT_DIR}}/{path}")],
                    "timeout": 10,
                    "statusMessage": "Reviewing Assura feedback"
                }]
            });
            if matches!(event, "PreToolUse" | "PostToolUse" | "PostToolUseFailure") {
                group["matcher"] = Value::String("*".to_string());
            }
            (event, group)
        })
        .collect(),
        AgentIntegrationTarget::Opencode | AgentIntegrationTarget::Pi => Vec::new(),
    }
}

fn read_json(path: &Path) -> Result<Option<Value>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&contents)
        .map(Some)
        .map_err(|error| format!("invalid host JSON {}: {error}", path_string(path)))
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut contents = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    contents.push('\n');
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn hook_groups(config: &Value) -> Vec<(&str, &[Value])> {
    config
        .get("hooks")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|hooks| hooks.iter())
        .filter_map(|(event, groups)| {
            groups
                .as_array()
                .map(|groups| (event.as_str(), groups.as_slice()))
        })
        .collect()
}

fn group_references_assura(group: &Value, agent: AgentIntegrationTarget) -> bool {
    let marker = format!(".assura/integrations/{}/assura-hook.py", agent.as_str());
    group
        .get("hooks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|hook| {
            hook.get("command")
                .and_then(Value::as_str)
                .is_some_and(|command| command.contains(&marker))
                || hook
                    .get("args")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|arg| arg.contains(&marker))
        })
}

fn object_field<'a>(value: &'a mut Value, key: &str) -> Result<&'a mut Map<String, Value>, String> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| "host config root must be a JSON object".to_string())?;
    let field = object
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    field
        .as_object_mut()
        .ok_or_else(|| format!("host config field '{key}' must be an object"))
}

fn array_field<'a>(
    object: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Vec<Value>, String> {
    let field = object
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    field
        .as_array_mut()
        .ok_or_else(|| format!("host hooks field '{key}' must be an array"))
}

fn remove_expected_groups(config: &mut Value, agent: AgentIntegrationTarget) -> Result<(), String> {
    let expected = expected_groups(agent);
    let hooks = object_field(config, "hooks")?;
    for (event, group) in expected {
        if let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) {
            groups.retain(|existing| existing != &group);
        }
    }
    Ok(())
}

fn prune_empty_hooks(config: &mut Value) {
    let Some(root) = config.as_object_mut() else {
        return;
    };
    let remove_hooks = if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        hooks.retain(|_, groups| groups.as_array().is_none_or(|groups| !groups.is_empty()));
        hooks.is_empty()
    } else {
        false
    };
    if remove_hooks {
        root.remove("hooks");
    }
}

fn empty_object(value: &Value) -> bool {
    value.as_object().is_some_and(Map::is_empty)
}
