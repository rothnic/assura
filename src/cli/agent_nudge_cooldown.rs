//! Bounded repeated-message suppression for agent lifecycle events.

use super::NudgeItem;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_SCHEMA: &str = "assura.agent-nudge-cooldown.v1";

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
    messages: BTreeMap<String, u64>,
}

pub(super) fn apply(
    project_root: &Path,
    event: &str,
    agent: &str,
    nudges: &mut Vec<NudgeItem>,
    seconds: u64,
) -> CooldownSummary {
    if seconds == 0 || nudges.is_empty() {
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
    state
        .messages
        .retain(|_, timestamp| within_cooldown(now, *timestamp, seconds));
    let before = nudges.len();
    nudges.retain(|nudge| {
        let fingerprint = fingerprint(&session, event, agent, nudge);
        if state
            .messages
            .get(&fingerprint)
            .is_some_and(|timestamp| *timestamp <= now && now.saturating_sub(*timestamp) < seconds)
        {
            false
        } else {
            state.messages.insert(fingerprint, now);
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
    use super::within_cooldown;

    #[test]
    fn cooldown_discards_future_and_expired_timestamps() {
        assert!(within_cooldown(1_000, 950, 60));
        assert!(!within_cooldown(1_000, 900, 60));
        assert!(!within_cooldown(1_000, 1_001, 60));
    }
}
