//! Fast automatic feedback selection over cached trajectory facts.

use crate::config::config::{AgentFeedbackConfig, AgentFeedbackMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_SCHEMA: &str = "assura.agent-feedback-delivery.v1";
const HOUR_SECONDS: i64 = 3_600;
const MAX_STATE_SENDS: usize = 64;

/// Effective automatic feedback settings exposed by inspect output.
#[derive(Debug, Serialize)]
pub(super) struct EffectiveDeliveryConfig {
    max_bytes: u64,
    min_interval_seconds: u64,
    max_messages_per_hour: u64,
    max_bytes_per_hour: u64,
    reminder_seconds: u64,
    max_reminders_per_episode: u64,
    periodic_seconds: u64,
    stale_after_seconds: u64,
}

impl From<&AgentFeedbackConfig> for EffectiveDeliveryConfig {
    fn from(config: &AgentFeedbackConfig) -> Self {
        Self {
            max_bytes: u64::from(config.max_bytes),
            min_interval_seconds: config.min_interval_seconds,
            max_messages_per_hour: u64::from(config.max_messages_per_hour),
            max_bytes_per_hour: u64::from(config.max_bytes_per_hour),
            reminder_seconds: config.reminder_seconds,
            max_reminders_per_episode: u64::from(config.max_reminders_per_episode),
            periodic_seconds: config.periodic_seconds,
            stale_after_seconds: config.collection.stale_after_seconds,
        }
    }
}

/// Inspectable automatic delivery status.
#[derive(Debug, Serialize)]
pub(super) struct DeliveryStatus {
    pub(super) configured: bool,
    pub(super) mode: &'static str,
    pub(super) snapshot: &'static str,
    pub(super) refresh: &'static str,
    pub(super) reason: &'static str,
    pub(super) messages_last_hour: usize,
    pub(super) bytes_last_hour: usize,
    pub(super) effective: EffectiveDeliveryConfig,
}

pub(super) struct DeliveryResult {
    pub(super) status: DeliveryStatus,
    pub(super) line: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct DeliveryState {
    schema: String,
    first_pending_at: Option<i64>,
    clean_since: Option<i64>,
    last_sent_at: Option<i64>,
    last_signal: Option<String>,
    reminders_sent: u8,
    last_refresh_at: Option<i64>,
    #[serde(default)]
    last_refresh_at_ms: Option<i64>,
    #[serde(default)]
    last_refresh_event: Option<String>,
    sends: Vec<SendRecord>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SendRecord {
    at: i64,
    bytes: usize,
    reminder: bool,
}

struct SignalSelection {
    key: String,
    reminder: bool,
}

/// Apply automatic delivery to one already-collected event.
pub(super) fn automatic(
    project_root: &Path,
    config: &AgentFeedbackConfig,
    event: &str,
    now: i64,
) -> DeliveryResult {
    let mode = config.mode.as_str();
    let effective = EffectiveDeliveryConfig::from(config);
    if config.mode == AgentFeedbackMode::Off {
        return result(
            mode,
            "disabled",
            "not_requested",
            "mode_off",
            0,
            0,
            effective,
            None,
        );
    }

    let Some(snapshot) = super::agent_trajectory::cached_snapshot(project_root, config) else {
        let refresh = request_refresh(project_root, config, now, event);
        return result(
            mode,
            "missing",
            refresh,
            "cold_snapshot",
            0,
            0,
            effective,
            None,
        );
    };
    if snapshot.coverage() != "complete" {
        let refresh = request_refresh(project_root, config, now, event);
        return result(
            mode,
            "incomplete",
            refresh,
            "incomplete_snapshot",
            0,
            0,
            effective,
            None,
        );
    }
    if snapshot.age_seconds(now) > config.collection.stale_after_seconds as i64 {
        let refresh = request_refresh(project_root, config, now, event);
        return result(
            mode,
            "stale",
            refresh,
            "stale_snapshot",
            0,
            0,
            effective,
            None,
        );
    }

    let path = state_path(project_root);
    if !state_is_valid(&path) {
        return result(
            mode,
            "fresh",
            "not_requested",
            "state_corrupt",
            0,
            0,
            effective,
            None,
        );
    }
    let Some(_lease) = DeliveryLease::acquire(&path) else {
        return result(
            mode,
            "fresh",
            "not_requested",
            "delivery_in_flight",
            0,
            0,
            effective,
            None,
        );
    };
    let mut state = read_state(&path);
    prune_sends(&mut state, now);
    let messages = state.sends.len();
    let bytes = state.sends.iter().map(|send| send.bytes).sum::<usize>();
    let pending = snapshot.has_pending_work();
    update_pending_episode(&mut state, pending, snapshot.captured_at(), config);
    let selection = match config.mode {
        AgentFeedbackMode::Periodic => state
            .last_sent_at
            .is_none_or(|last| now.saturating_sub(last) >= config.periodic_seconds as i64)
            .then_some(SignalSelection {
                key: "periodic".to_string(),
                reminder: false,
            }),
        AgentFeedbackMode::Threshold => threshold_selection(&mut state, &snapshot, now, config),
        AgentFeedbackMode::Off => None,
    };

    let mut line = None;
    let mut reason = "no_signal";
    if let Some(selection) = selection {
        if state
            .last_sent_at
            .is_some_and(|last| now.saturating_sub(last) < config.min_interval_seconds as i64)
        {
            reason = "minimum_interval";
        } else if messages >= usize::from(config.max_messages_per_hour) {
            reason = "hourly_message_budget";
        } else {
            let candidate = snapshot.compact_line_for(
                state.first_pending_at,
                usize::from(config.max_bytes),
                &config.trajectory.metrics,
            );
            if candidate.is_empty() {
                reason = "line_unavailable";
            } else if candidate.len() > usize::from(config.max_bytes) {
                reason = "line_exceeds_byte_budget";
            } else if bytes.saturating_add(candidate.len()) > config.max_bytes_per_hour as usize {
                reason = "hourly_byte_budget";
            } else {
                state.last_sent_at = Some(now);
                state.last_signal = Some(selection.key);
                if selection.reminder {
                    state.reminders_sent = state.reminders_sent.saturating_add(1);
                }
                state.sends.push(SendRecord {
                    at: now,
                    bytes: candidate.len(),
                    reminder: selection.reminder,
                });
                line = Some(candidate);
                reason = "delivered";
            }
        }
    }
    let _ = write_state(&path, &state);
    DeliveryResult {
        status: DeliveryStatus {
            configured: true,
            mode,
            snapshot: "fresh",
            refresh: "not_requested",
            reason,
            messages_last_hour: state.sends.len(),
            bytes_last_hour: state.sends.iter().map(|send| send.bytes).sum(),
            effective,
        },
        line,
    }
}

// Keep the status constructor explicit so every suppression reason is visible
// in the inspect envelope and no delivery policy is hidden in a builder.
#[allow(clippy::too_many_arguments)]
fn result(
    mode: &'static str,
    snapshot: &'static str,
    refresh: &'static str,
    reason: &'static str,
    messages: usize,
    bytes: usize,
    effective: EffectiveDeliveryConfig,
    line: Option<String>,
) -> DeliveryResult {
    DeliveryResult {
        status: DeliveryStatus {
            configured: true,
            mode,
            snapshot,
            refresh,
            reason,
            messages_last_hour: messages,
            bytes_last_hour: bytes,
            effective,
        },
        line,
    }
}

pub(super) fn inspect_status(project_root: &Path, config: &AgentFeedbackConfig) -> DeliveryStatus {
    let state = read_state(&state_path(project_root));
    let current = now();
    let (messages_last_hour, bytes_last_hour) = state
        .sends
        .iter()
        .filter(|send| current.saturating_sub(send.at) < HOUR_SECONDS)
        .fold((0usize, 0usize), |(messages, bytes), send| {
            (messages + 1, bytes.saturating_add(send.bytes))
        });
    DeliveryStatus {
        configured: true,
        mode: config.mode.as_str(),
        snapshot: "explicit_inspect",
        refresh: "not_requested",
        reason: "explicit_inspect",
        messages_last_hour,
        bytes_last_hour,
        effective: EffectiveDeliveryConfig::from(config),
    }
}

fn update_pending_episode(
    state: &mut DeliveryState,
    pending: bool,
    observed_at: i64,
    config: &AgentFeedbackConfig,
) {
    let clear_after = config
        .trajectory
        .signals
        .unintegrated
        .clear_after_clean_seconds as i64;
    if pending {
        state.clean_since = None;
        state.first_pending_at.get_or_insert(observed_at);
    } else if state.first_pending_at.is_some() {
        let clean_since = state.clean_since.get_or_insert(observed_at);
        if observed_at.saturating_sub(*clean_since) >= clear_after {
            state.first_pending_at = None;
            state.clean_since = None;
            state.last_signal = None;
            state.reminders_sent = 0;
        }
    }
}

fn threshold_selection(
    state: &mut DeliveryState,
    snapshot: &super::agent_trajectory::TrajectorySnapshot,
    now: i64,
    config: &AgentFeedbackConfig,
) -> Option<SignalSelection> {
    let signals = &config.trajectory.signals;
    if signals.unintegrated.enabled {
        if let Some(started) = state.first_pending_at {
            let age = now.saturating_sub(started);
            let threshold = signals.unintegrated.pending_minutes.saturating_mul(60) as i64;
            if snapshot.has_pending_work() && age >= threshold {
                let step = signals.unintegrated.step_minutes.max(1).saturating_mul(60) as i64;
                let key = format!("pending:{}", age.div_euclid(step));
                if state.last_signal.as_deref() != Some(key.as_str()) {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                    });
                }
                if config.reminder_seconds > 0
                    && state.reminders_sent < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                    });
                }
            }
        }
    }

    if signals.coordination.enabled {
        if let Some(only_coordination) =
            snapshot.coordination_only_commits(signals.coordination.commits)
        {
            if only_coordination >= signals.coordination.min_only_coordination_commits {
                let key = "coordination".to_string();
                if state.last_signal.as_deref() != Some(key.as_str()) {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                    });
                }
                if config.reminder_seconds > 0
                    && state.reminders_sent < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                    });
                }
            } else if only_coordination < signals.coordination.clear_below_only_coordination_commits
                && state.last_signal.as_deref() == Some("coordination")
            {
                state.last_signal = None;
                state.reminders_sent = 0;
            }
        }
    }

    if signals.patch_size.enabled {
        if let Some(lines) = snapshot.pending_changed_lines() {
            if lines >= signals.patch_size.changed_lines {
                let step = signals.patch_size.step_lines.max(1);
                let key = format!("patch:{}", lines / step);
                if state.last_signal.as_deref() != Some(key.as_str()) {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                    });
                }
                if config.reminder_seconds > 0
                    && state.reminders_sent < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                    });
                }
            } else if lines < signals.patch_size.clear_below_changed_lines
                && state
                    .last_signal
                    .as_deref()
                    .is_some_and(|value| value.starts_with("patch:"))
            {
                state.last_signal = None;
                state.reminders_sent = 0;
            }
        }
    }
    None
}

fn prune_sends(state: &mut DeliveryState, now: i64) {
    state
        .sends
        .retain(|send| now.saturating_sub(send.at) < HOUR_SECONDS);
    if state.sends.len() > MAX_STATE_SENDS {
        let keep_from = state.sends.len() - MAX_STATE_SENDS;
        state.sends.drain(..keep_from);
    }
}

fn request_refresh(
    project_root: &Path,
    config: &AgentFeedbackConfig,
    now: i64,
    event: &str,
) -> &'static str {
    let lock = refresh_lock_path(project_root);
    let Some(parent) = lock.parent() else {
        return "unavailable";
    };
    if fs::create_dir_all(parent).is_err() {
        return "unavailable";
    }
    if let Ok(metadata) = fs::metadata(&lock) {
        let stale_after = (config.collection.timeout_ms / 1000).max(10) as i64 * 2;
        let stale = metadata
            .modified()
            .ok()
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .map(|value| now.saturating_sub(value.as_secs() as i64) > stale_after)
            .unwrap_or(false);
        if stale {
            let _ = fs::remove_file(&lock);
        } else {
            return "in_flight";
        }
    }
    let state_path = state_path(project_root);
    if !state_is_valid(&state_path) {
        return "state_corrupt";
    }
    let mut state = read_state(&state_path);
    if state.last_refresh_at_ms.is_some_and(|last| {
        now_millis().saturating_sub(last) < config.collection.debounce_ms as i64
    }) {
        return "debounced";
    }
    if state
        .last_refresh_at
        .is_some_and(|last| now.saturating_sub(last) < config.collection.min_refresh_seconds as i64)
    {
        return "cooldown";
    }
    if OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .is_err()
    {
        return "in_flight";
    }
    state.last_refresh_at = Some(now);
    state.last_refresh_at_ms = Some(now_millis());
    state.last_refresh_event = Some(event.to_string());
    let _ = write_state(&state_path, &state);
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            let _ = fs::remove_file(&lock);
            return "unavailable";
        }
    };
    let child = Command::new(executable)
        .args([
            "agent",
            "nudge",
            project_root.to_string_lossy().as_ref(),
            "--delivery",
            "inspect",
            "--format",
            "json",
        ])
        .env("ASSURA_FEEDBACK_REFRESH_LOCK", &lock)
        .env(
            "ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS",
            config.collection.timeout_ms.to_string(),
        )
        .env_remove("ASSURA_AGENT_LOG")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if child.is_err() {
        let _ = fs::remove_file(lock);
        return "unavailable";
    }
    "scheduled"
}

/// Release a refresh lease when the short-lived inspect child exits.
pub(super) fn finish_refresh() {
    if let Some(path) = std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK") {
        let _ = fs::remove_file(path);
    }
}

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or_default()
}

fn feedback_root(project_root: &Path) -> PathBuf {
    let marker = project_root.join(".git");
    let git_dir = if marker.is_dir() {
        marker
    } else {
        let Ok(value) = fs::read_to_string(&marker) else {
            return project_root.join(".assura/feedback");
        };
        let Some(value) = value.strip_prefix("gitdir: ").map(str::trim) else {
            return project_root.join(".assura/feedback");
        };
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            project_root.join(path)
        }
    };
    let common = if git_dir.file_name().and_then(|name| name.to_str()) == Some(".git") {
        git_dir
    } else {
        git_dir
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or(git_dir)
    };
    common.join("assura/feedback")
}

fn refresh_lock_path(project_root: &Path) -> PathBuf {
    feedback_root(project_root).join(format!("{}.refresh", digest(&identity_path(project_root))))
}

fn state_path(project_root: &Path) -> PathBuf {
    feedback_root(project_root).join(format!("{}.json", digest(&identity_path(project_root))))
}

fn identity_path(project_root: &Path) -> PathBuf {
    let marker = project_root.join(".git");
    if marker.is_dir() {
        return marker;
    }
    let Ok(value) = fs::read_to_string(marker) else {
        return project_root.to_path_buf();
    };
    let Some(value) = value.strip_prefix("gitdir: ").map(str::trim) else {
        return project_root.to_path_buf();
    };
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        project_root.join(path)
    }
}

fn digest(path: &Path) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
                .as_bytes()
        )
    )
}

fn read_state(path: &Path) -> DeliveryState {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .filter(|state: &DeliveryState| state.schema == STATE_SCHEMA)
        .unwrap_or_else(|| DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            ..Default::default()
        })
}

fn state_is_valid(path: &Path) -> bool {
    !path.exists()
        || fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<DeliveryState>(&bytes).ok())
            .is_some_and(|state| state.schema == STATE_SCHEMA)
}

fn write_state(path: &Path, state: &DeliveryState) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temporary = parent.join(format!(
        ".{}.{}.tmp",
        std::process::id(),
        now().unsigned_abs()
    ));
    fs::write(
        &temporary,
        serde_json::to_vec(state).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::rename(&temporary, path).map_err(|error| error.to_string())
}

struct DeliveryLease {
    path: PathBuf,
}

impl DeliveryLease {
    fn acquire(state: &Path) -> Option<Self> {
        let parent = state.parent()?;
        if fs::create_dir_all(parent).is_err() {
            return None;
        }
        let path = state.with_extension("lock");
        if let Ok(metadata) = fs::metadata(&path) {
            let stale = metadata
                .modified()
                .ok()
                .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                .map(|value| now().saturating_sub(value.as_secs() as i64) > 30)
                .unwrap_or(false);
            if stale {
                let _ = fs::remove_file(&path);
            }
        }
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()
            .map(|_| Self { path })
    }
}

impl Drop for DeliveryLease {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn threshold_episode_requires_observed_age_and_clears_after_clean_period() {
        let mut state = DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            ..Default::default()
        };
        let config = AgentFeedbackConfig::default();
        update_pending_episode(&mut state, true, 1_000, &config);
        assert_eq!(state.first_pending_at, Some(1_000));
        update_pending_episode(&mut state, false, 1_001, &config);
        assert_eq!(state.first_pending_at, Some(1_000));
        update_pending_episode(&mut state, false, 1_062, &config);
        assert!(state.first_pending_at.is_none());
    }

    #[test]
    fn state_budget_records_bytes_not_just_message_count() {
        let mut state = DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            sends: vec![SendRecord {
                at: 1_000,
                bytes: 256,
                reminder: false,
            }],
            ..Default::default()
        };
        prune_sends(&mut state, 1_100);
        assert_eq!(
            state.sends.iter().map(|send| send.bytes).sum::<usize>(),
            256
        );
    }

    #[test]
    fn linked_worktrees_share_cache_root_but_not_delivery_state() {
        let root = tempdir().expect("repository root");
        let common = root.path().join(".git");
        let linked_git = common.join("worktrees").join("linked");
        fs::create_dir_all(&linked_git).expect("linked git dir");
        let linked = root.path().join("linked");
        fs::create_dir_all(&linked).expect("linked worktree");
        fs::write(
            linked.join(".git"),
            format!("gitdir: {}\n", linked_git.display()),
        )
        .expect("worktree marker");

        assert_eq!(feedback_root(root.path()), feedback_root(&linked));
        assert_ne!(identity_path(root.path()), identity_path(&linked));
        assert_ne!(state_path(root.path()), state_path(&linked));
    }
}
