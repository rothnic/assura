//! Bounded repeated-message suppression for agent lifecycle events.

use super::{helpers::path_string, NudgeItem};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_SCHEMA: &str = "assura.agent-nudge-cooldown.v2";

#[derive(Debug, Serialize)]
pub(super) struct CooldownSummary {
    pub(super) seconds: u64,
    pub(super) suppressed: usize,
    mode: &'static str,
    fallback_reason: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct CachePolicy {
    pub(super) stable_by_default: bool,
    pub(super) volatile_fields: Vec<&'static str>,
    pub(super) default_detail: &'static str,
    pub(super) cooldown: CooldownSummary,
}

#[derive(Default, Deserialize, Serialize)]
struct CooldownState {
    schema: String,
    messages: BTreeMap<String, CachedMessage>,
}

#[derive(Deserialize, Serialize)]
struct CachedMessage {
    timestamp: u64,
    path: Option<String>,
}

pub(super) fn apply(
    project_root: &Path,
    event: &str,
    agent: &str,
    policy_generation: &str,
    changed_paths: &[PathBuf],
    nudges: &mut Vec<NudgeItem>,
    seconds: u64,
) -> CooldownSummary {
    if seconds == 0 {
        return CooldownSummary {
            seconds,
            suppressed: 0,
            mode: "disabled",
            fallback_reason: None,
        };
    }
    let now = unix_seconds();
    let session = std::env::var("ASSURA_AGENT_SESSION_ID").unwrap_or_else(|_| "manual".to_string());
    let (path, mode, fallback_reason) = state_path(project_root);
    let mut state = read_state(&path);
    let agent_generation = format!("{agent}\0{policy_generation}");
    let observed = nudges
        .iter()
        .map(|nudge| fingerprint(&session, event, &agent_generation, nudge))
        .collect::<BTreeSet<_>>();
    let changed_paths = changed_paths
        .iter()
        .map(|path| path_string(path))
        .collect::<BTreeSet<_>>();
    state.messages.retain(|fingerprint, message| {
        within_cooldown(now, message.timestamp, seconds)
            && match message.path.as_ref() {
                Some(path) if changed_paths.contains(path) => observed.contains(fingerprint),
                Some(_) => true,
                None => observed.contains(fingerprint),
            }
    });
    let before = nudges.len();
    nudges.retain(|nudge| {
        let fingerprint = fingerprint(&session, event, &agent_generation, nudge);
        if state.messages.get(&fingerprint).is_some_and(|message| {
            message.timestamp <= now && now.saturating_sub(message.timestamp) < seconds
        }) {
            false
        } else {
            state.messages.insert(
                fingerprint,
                CachedMessage {
                    timestamp: now,
                    path: nudge.path.clone(),
                },
            );
            true
        }
    });
    let _ = write_state(&path, &state);
    CooldownSummary {
        seconds,
        suppressed: before.saturating_sub(nudges.len()),
        mode,
        fallback_reason,
    }
}

pub(super) fn policy_generation(project_root: &Path, config: Option<&Path>) -> String {
    let path = config
        .map(Path::to_path_buf)
        .unwrap_or_else(|| project_root.join(".assura/config.yml"));
    let contents = fs::read(&path).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(path.as_os_str().as_encoded_bytes());
    hasher.update([0]);
    hasher.update(contents);
    format!("{:x}", hasher.finalize())
}

fn state_path(project_root: &Path) -> (PathBuf, &'static str, Option<&'static str>) {
    let project_key = digest(
        &project_root
            .canonicalize()
            .unwrap_or_else(|_| project_root.to_path_buf())
            .to_string_lossy(),
    );
    let git_path = format!("assura/nudge-cooldowns/{project_key}.json");
    let output = std::process::Command::new("git")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(project_root)
        .args(["rev-parse", "--git-path", &git_path])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
            return (
                if path.is_absolute() {
                    path
                } else {
                    project_root.join(path)
                },
                "git-worktree",
                None,
            );
        }
    }
    let key = digest(&project_root.to_string_lossy());
    (
        std::env::temp_dir()
            .join("assura/nudge-cooldowns")
            .join(format!("{key}.json")),
        "temporary",
        Some("Git worktree metadata is unavailable"),
    )
}

fn read_state(path: &Path) -> CooldownState {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<CooldownState>(&bytes).ok())
        .filter(|state| state.schema == STATE_SCHEMA)
        .unwrap_or_else(|| CooldownState {
            schema: STATE_SCHEMA.to_string(),
            messages: BTreeMap::new(),
        })
}

fn write_state(path: &Path, state: &CooldownState) -> std::io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent)?;
    set_private_permissions(parent, true);
    let bytes = serde_json::to_vec(state)?;
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes)?;
    set_private_permissions(&temporary, false);
    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path, directory: bool) {
    use std::os::unix::fs::PermissionsExt;
    let mode = if directory { 0o700 } else { 0o600 };
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path, _directory: bool) {}

fn fingerprint(session: &str, event: &str, agent: &str, nudge: &NudgeItem) -> String {
    digest(&format!(
        "{session}\0{event}\0{agent}\0{}\0{}\0{}\0{}\0{}",
        nudge.category,
        nudge.path.as_deref().unwrap_or(""),
        nudge.rule.as_deref().unwrap_or(""),
        nudge.severity,
        nudge.message
    ))
}

fn within_cooldown(now: u64, timestamp: u64, seconds: u64) -> bool {
    timestamp <= now && now - timestamp <= seconds
}

fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::super::NudgeItem;
    use super::{apply, policy_generation, within_cooldown};
    use std::fs;
    use std::path::PathBuf;

    fn nudge(rule: &str) -> NudgeItem {
        NudgeItem {
            category: "structure",
            path: Some("src/BadName.js".to_string()),
            rule: Some(rule.to_string()),
            severity: "medium",
            message: format!("{rule} violation"),
            suggested_command: String::new(),
            inject: true,
            daemon_health: None,
        }
    }

    fn pathless_nudge() -> NudgeItem {
        NudgeItem {
            category: "daemon",
            path: None,
            rule: Some("daemon_health".to_string()),
            severity: "medium",
            message: "daemon is unavailable".to_string(),
            suggested_command: String::new(),
            inject: true,
            daemon_health: None,
        }
    }

    #[test]
    fn cooldown_discards_future_and_expired_timestamps() {
        assert!(within_cooldown(1_000, 950, 60));
        assert!(!within_cooldown(1_000, 900, 60));
        assert!(!within_cooldown(1_000, 1_001, 60));
    }

    #[test]
    fn policy_generation_distinguishes_non_utf8_config_bytes() {
        let project = tempfile::tempdir().expect("temporary project");
        let config = project.path().join("config.yml");
        fs::write(&config, b"structure: {}\n# \xff\n").expect("write first config");
        let first = policy_generation(project.path(), Some(&config));
        fs::write(&config, b"structure: {}\n# \xfe\n").expect("write second config");

        assert_ne!(first, policy_generation(project.path(), Some(&config)));
    }

    #[test]
    fn reintroduced_finding_is_not_suppressed_when_another_finding_persists() {
        let project = tempfile::tempdir().expect("temporary project");
        let changed = [PathBuf::from("src/BadName.js")];
        let mut initial = vec![nudge("file_naming"), nudge("file_extension")];
        apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &changed,
            &mut initial,
            600,
        );
        assert_eq!(initial.len(), 2);

        let mut resolved_naming = vec![nudge("file_extension")];
        let summary = apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &changed,
            &mut resolved_naming,
            600,
        );
        assert!(resolved_naming.is_empty());
        assert_eq!(summary.suppressed, 1);

        let mut reintroduced = vec![nudge("file_naming"), nudge("file_extension")];
        let summary = apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &changed,
            &mut reintroduced,
            600,
        );
        assert_eq!(reintroduced.len(), 1);
        assert_eq!(reintroduced[0].rule.as_deref(), Some("file_naming"));
        assert_eq!(summary.suppressed, 1);
    }

    #[test]
    fn reintroduced_pathless_failure_is_not_suppressed_after_recovery() {
        let project = tempfile::tempdir().expect("temporary project");
        let mut failure = vec![pathless_nudge()];
        apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &[],
            &mut failure,
            600,
        );
        assert_eq!(failure.len(), 1);

        let mut recovery = Vec::new();
        apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &[],
            &mut recovery,
            600,
        );
        assert!(recovery.is_empty());

        let mut reintroduced = vec![pathless_nudge()];
        let summary = apply(
            project.path(),
            "after_tool",
            "codex",
            "policy",
            &[],
            &mut reintroduced,
            600,
        );
        assert_eq!(reintroduced.len(), 1);
        assert_eq!(summary.suppressed, 0);
    }
}
