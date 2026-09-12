//! Fast automatic feedback selection over cached trajectory facts.

use crate::config::config::{AgentFeedbackConfig, AgentFeedbackMode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STATE_SCHEMA: &str = "assura.agent-feedback-delivery.v1";
const HOUR_SECONDS: i64 = 3_600;
const MAX_STATE_SENDS: usize = 64;
const MAX_STATE_BYTES: u64 = 64 * 1024;
const ROUTINE_WRAPPER_BYTES: usize = "<assura-feedback>\n\n</assura-feedback>".len();
const REFRESH_TOKEN_ENV: &str = "ASSURA_FEEDBACK_REFRESH_TOKEN";

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct DeliveryState {
    schema: String,
    first_pending_at: Option<i64>,
    clean_since: Option<i64>,
    last_sent_at: Option<i64>,
    #[serde(default)]
    signals: SignalStates,
    // Read legacy state once and migrate it into the per-signal fields.
    #[serde(default, skip_serializing)]
    last_signal: Option<String>,
    #[serde(default, skip_serializing)]
    reminders_sent: u8,
    last_refresh_at: Option<i64>,
    #[serde(default)]
    last_refresh_at_ms: Option<i64>,
    #[serde(default)]
    last_refresh_event: Option<String>,
    sends: Vec<SendRecord>,
}

#[derive(Clone, Copy, Debug)]
enum SignalFamily {
    Pending,
    Coordination,
    Patch,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct SignalState {
    #[serde(default)]
    last_key: Option<String>,
    #[serde(default)]
    reminders_sent: u8,
}

impl SignalState {
    fn clear(&mut self) {
        self.last_key = None;
        self.reminders_sent = 0;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct SignalStates {
    #[serde(default)]
    pending: SignalState,
    #[serde(default)]
    coordination: SignalState,
    #[serde(default)]
    patch: SignalState,
}

impl SignalStates {
    fn get(&self, family: SignalFamily) -> &SignalState {
        match family {
            SignalFamily::Pending => &self.pending,
            SignalFamily::Coordination => &self.coordination,
            SignalFamily::Patch => &self.patch,
        }
    }

    fn get_mut(&mut self, family: SignalFamily) -> &mut SignalState {
        match family {
            SignalFamily::Pending => &mut self.pending,
            SignalFamily::Coordination => &mut self.coordination,
            SignalFamily::Patch => &mut self.patch,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct SendRecord {
    at: i64,
    bytes: usize,
    reminder: bool,
}

struct SignalSelection {
    key: String,
    reminder: bool,
    family: Option<SignalFamily>,
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
    let previous_state = state.clone();
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
                family: None,
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
                usize::from(config.max_bytes).saturating_sub(ROUTINE_WRAPPER_BYTES),
                &config.trajectory.metrics,
            );
            let emitted_bytes = candidate.len().saturating_add(ROUTINE_WRAPPER_BYTES);
            if candidate.is_empty() {
                reason = "line_unavailable";
            } else if emitted_bytes > usize::from(config.max_bytes) {
                reason = "line_exceeds_byte_budget";
            } else if bytes.saturating_add(emitted_bytes) > config.max_bytes_per_hour as usize {
                reason = "hourly_byte_budget";
            } else {
                state.last_sent_at = Some(now);
                if let Some(family) = selection.family {
                    let signal = state.signals.get_mut(family);
                    if signal.last_key.as_deref() != Some(selection.key.as_str()) {
                        signal.reminders_sent = 0;
                    }
                    signal.last_key = Some(selection.key.clone());
                    if selection.reminder {
                        signal.reminders_sent = signal.reminders_sent.saturating_add(1);
                    }
                }
                state.sends.push(SendRecord {
                    at: now,
                    bytes: emitted_bytes,
                    reminder: selection.reminder,
                });
                line = Some(candidate);
                reason = "delivered";
            }
        }
    }
    if write_state(&path, &state).is_err() && line.is_some() {
        state = previous_state;
        line = None;
        reason = "state_write_failed";
    }
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

// allow-reason: keep the status constructor explicit so every suppression
// reason is visible in the inspect envelope and no delivery policy is hidden in a builder.
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
            state.signals.get_mut(SignalFamily::Pending).clear();
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
                if state.signals.get(SignalFamily::Pending).last_key.as_deref()
                    != Some(key.as_str())
                {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                        family: Some(SignalFamily::Pending),
                    });
                }
                if config.reminder_seconds > 0
                    && state.signals.get(SignalFamily::Pending).reminders_sent
                        < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                        family: Some(SignalFamily::Pending),
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
                if state
                    .signals
                    .get(SignalFamily::Coordination)
                    .last_key
                    .as_deref()
                    != Some(key.as_str())
                {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                        family: Some(SignalFamily::Coordination),
                    });
                }
                if config.reminder_seconds > 0
                    && state.signals.get(SignalFamily::Coordination).reminders_sent
                        < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                        family: Some(SignalFamily::Coordination),
                    });
                }
            } else if only_coordination < signals.coordination.clear_below_only_coordination_commits
                && state
                    .signals
                    .get(SignalFamily::Coordination)
                    .last_key
                    .as_deref()
                    == Some("coordination")
            {
                state.signals.get_mut(SignalFamily::Coordination).clear();
            }
        }
    }

    if signals.patch_size.enabled {
        if let Some(lines) = snapshot.pending_changed_lines() {
            if lines >= signals.patch_size.changed_lines {
                let step = signals.patch_size.step_lines.max(1);
                let key = format!("patch:{}", lines / step);
                if state.signals.get(SignalFamily::Patch).last_key.as_deref() != Some(key.as_str())
                {
                    return Some(SignalSelection {
                        key,
                        reminder: false,
                        family: Some(SignalFamily::Patch),
                    });
                }
                if config.reminder_seconds > 0
                    && state.signals.get(SignalFamily::Patch).reminders_sent
                        < config.max_reminders_per_episode
                    && state.last_sent_at.is_some_and(|last| {
                        now.saturating_sub(last) >= config.reminder_seconds as i64
                    })
                {
                    return Some(SignalSelection {
                        key,
                        reminder: true,
                        family: Some(SignalFamily::Patch),
                    });
                }
            } else if lines < signals.patch_size.clear_below_changed_lines
                && state
                    .signals
                    .get(SignalFamily::Patch)
                    .last_key
                    .as_deref()
                    .is_some_and(|value| value.starts_with("patch:"))
            {
                state.signals.get_mut(SignalFamily::Patch).clear();
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
        if stale && lease_owner_alive(&lock) != Some(true) {
            let _ = fs::remove_file(&lock);
        } else {
            let queued = lock.with_extension("queued");
            let _ = OpenOptions::new().write(true).create_new(true).open(queued);
            return "in_flight";
        }
    }
    let state_path = state_path(project_root);
    if !state_is_valid(&state_path) {
        return "state_corrupt";
    }
    let Some(_state_lease) = DeliveryLease::acquire(&state_path) else {
        return "in_flight";
    };
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
    let token = format!("{}:{}", std::process::id(), now_millis());
    if fs::write(&lock, &token).is_err() {
        let _ = fs::remove_file(&lock);
        return "unavailable";
    }
    state.last_refresh_at = Some(now);
    state.last_refresh_at_ms = Some(now_millis());
    state.last_refresh_event = Some(event.to_string());
    if write_state(&state_path, &state).is_err() {
        let _ = fs::remove_file(&lock);
        return "unavailable";
    }
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
        .env(REFRESH_TOKEN_ENV, &token)
        .env(
            "ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS",
            config.collection.timeout_ms.to_string(),
        )
        .env(
            "ASSURA_FEEDBACK_REFRESH_DEADLINE_MS",
            now_millis()
                .saturating_add(config.collection.timeout_ms as i64 * 1_000)
                .to_string(),
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
        let path = PathBuf::from(path);
        let Some(token) = std::env::var_os(REFRESH_TOKEN_ENV) else {
            return;
        };
        if !refresh_lease_owned(&path, &token) {
            return;
        }
        let queued = path.with_extension("queued");
        if queued.is_file() {
            let next_token = format!("{}:{}", std::process::id(), now_millis());
            if !replace_refresh_lease(&path, &next_token) {
                return;
            }
            let spawned = std::env::current_exe().ok().is_some_and(|executable| {
                let args = std::env::args_os().skip(1).collect::<Vec<_>>();
                Command::new(executable)
                    .args(args)
                    .env("ASSURA_FEEDBACK_REFRESH_LOCK", &path)
                    .env(REFRESH_TOKEN_ENV, &next_token)
                    .env(
                        "ASSURA_FEEDBACK_REFRESH_DEADLINE_MS",
                        now_millis()
                            .saturating_add(
                                std::env::var("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS")
                                    .ok()
                                    .and_then(|value| value.parse::<i64>().ok())
                                    .unwrap_or(2_000)
                                    .saturating_mul(1_000),
                            )
                            .to_string(),
                    )
                    .env_remove("ASSURA_AGENT_LOG")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .is_ok()
            });
            if spawned {
                let _ = fs::remove_file(&queued);
            } else {
                let _ = remove_refresh_lease_if_owned(&path, std::ffi::OsStr::new(&next_token));
            }
            return;
        }
        let _ = remove_refresh_lease_if_owned(&path, &token);
    }
}

/// Bound the whole inspect worker, including config and daemon setup before Git.
pub(super) fn start_refresh_watchdog() {
    let Some(deadline) = refresh_deadline_millis() else {
        return;
    };
    let Some(lock) = std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK") else {
        return;
    };
    let Some(token) = std::env::var_os(REFRESH_TOKEN_ENV) else {
        return;
    };
    let remaining = deadline.saturating_sub(now_millis());
    if remaining <= 0 {
        return;
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(remaining as u64));
        if refresh_deadline_expired() {
            let lock = PathBuf::from(lock);
            let _ = remove_refresh_lease_if_owned(&lock, &token);
            std::process::exit(1);
        }
    });
}

fn refresh_lease_owned(path: &Path, token: &std::ffi::OsStr) -> bool {
    fs::read_to_string(path)
        .ok()
        .is_some_and(|current| current == token.to_string_lossy())
}

fn remove_refresh_lease_if_owned(path: &Path, token: &std::ffi::OsStr) -> bool {
    refresh_lease_owned(path, token) && fs::remove_file(path).is_ok()
}

fn replace_refresh_lease(path: &Path, token: &str) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let temporary = parent.join(format!(".{}.{}.tmp", std::process::id(), now_millis()));
    if fs::write(&temporary, token).is_err() {
        return false;
    }
    let result = super::replace_file(&temporary, path).is_ok();
    if !result {
        let _ = fs::remove_file(&temporary);
    }
    result
}

pub(super) fn refresh_deadline_expired() -> bool {
    refresh_deadline_millis().is_some_and(|deadline| now_millis() >= deadline)
}

fn refresh_deadline_millis() -> Option<i64> {
    std::env::var("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
}

pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

pub(super) fn now_millis() -> i64 {
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
    let mut state = read_bounded(path)
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .filter(|state: &DeliveryState| state.schema == STATE_SCHEMA)
        .unwrap_or_else(|| DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            ..Default::default()
        });
    if let Some(key) = state.last_signal.take() {
        if let Some(family) = signal_family(&key) {
            let legacy_reminders = state.reminders_sent;
            let signal = state.signals.get_mut(family);
            if signal.last_key.is_none() {
                signal.last_key = Some(key);
                signal.reminders_sent = legacy_reminders;
            }
        }
    }
    state.reminders_sent = 0;
    state
}

fn signal_family(key: &str) -> Option<SignalFamily> {
    if key.starts_with("pending:") {
        Some(SignalFamily::Pending)
    } else if key == "coordination" {
        Some(SignalFamily::Coordination)
    } else if key.starts_with("patch:") {
        Some(SignalFamily::Patch)
    } else {
        None
    }
}

fn state_is_valid(path: &Path) -> bool {
    !path.exists()
        || read_bounded(path)
            .and_then(|bytes| serde_json::from_slice::<DeliveryState>(&bytes).ok())
            .is_some_and(|state| state.schema == STATE_SCHEMA)
}

fn read_bounded(path: &Path) -> Option<Vec<u8>> {
    let file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(MAX_STATE_BYTES.saturating_add(1))
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= MAX_STATE_BYTES).then_some(bytes)
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
    super::replace_file(&temporary, path).map_err(|error| error.to_string())
}

pub(super) fn lease_owner_alive(path: &Path) -> Option<bool> {
    let token = fs::read_to_string(path).ok()?;
    let pid = token.split(':').next()?.parse::<u32>().ok()?;
    process_owner_alive(pid)
}

#[cfg(unix)]
fn process_owner_alive(pid: u32) -> Option<bool> {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .ok()
        .map(|status| status.success())
}

#[cfg(windows)]
fn process_owner_alive(pid: u32) -> Option<bool> {
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    // SAFETY: OpenProcess receives a PID parsed from the lease and no borrowed pointers.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if handle.is_null() {
        // Access failures are unknown; only an invalid PID proves the owner is gone.
        return match unsafe { GetLastError() } {
            ERROR_INVALID_PARAMETER => Some(false),
            ERROR_ACCESS_DENIED => None,
            _ => None,
        };
    }
    let mut exit_code = 0;
    // SAFETY: handle is owned above and exit_code is writable storage.
    let result = unsafe { GetExitCodeProcess(handle, &mut exit_code) } != 0;
    // SAFETY: handle is the process handle returned by OpenProcess.
    unsafe { CloseHandle(handle) };
    result.then_some(exit_code == STILL_ACTIVE as u32)
}

#[cfg(not(any(unix, windows)))]
fn process_owner_alive(_pid: u32) -> Option<bool> {
    None
}

struct DeliveryLease {
    path: PathBuf,
    token: String,
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
            if stale && lease_owner_alive(&path) != Some(true) {
                let _ = fs::remove_file(&path);
            }
        }
        let token = format!("{}:{}", std::process::id(), now_millis());
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .ok()?;
        drop(file);
        if fs::write(&path, &token).is_err() {
            let _ = fs::remove_file(&path);
            return None;
        }
        Some(Self { path, token })
    }
}

impl Drop for DeliveryLease {
    fn drop(&mut self) {
        if fs::read_to_string(&self.path).ok().as_deref() == Some(self.token.as_str()) {
            let _ = fs::remove_file(&self.path);
        }
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
    fn state_reads_are_bounded() {
        let root = tempdir().expect("state directory");
        let path = root.path().join("state.json");
        fs::write(&path, vec![b'x'; (MAX_STATE_BYTES + 1) as usize]).expect("large state");
        assert!(read_bounded(&path).is_none());
    }

    #[test]
    fn legacy_signal_state_migrates_without_cross_family_reuse() {
        let root = tempdir().expect("state directory");
        let path = root.path().join("state.json");
        fs::write(
            &path,
            serde_json::json!({
                "schema": STATE_SCHEMA,
                "last_signal": "patch:2",
                "reminders_sent": 1,
                "sends": []
            })
            .to_string(),
        )
        .expect("legacy state");

        let state = read_state(&path);
        assert_eq!(state.signals.patch.last_key.as_deref(), Some("patch:2"));
        assert_eq!(state.signals.patch.reminders_sent, 1);
        assert!(state.signals.pending.last_key.is_none());
        assert_eq!(state.signals.pending.reminders_sent, 0);
    }

    #[test]
    fn pending_episode_table_keeps_short_clean_gaps_in_one_episode() {
        let config = AgentFeedbackConfig::default();
        let mut state = DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            ..Default::default()
        };
        for (pending, observed_at, expected) in [
            (true, 1_000, Some(1_000)),
            (false, 1_001, Some(1_000)),
            (false, 1_060, Some(1_000)),
            (false, 1_061, None),
            (true, 1_062, Some(1_062)),
        ] {
            update_pending_episode(&mut state, pending, observed_at, &config);
            assert_eq!(state.first_pending_at, expected);
        }
    }

    #[test]
    fn delivery_lease_allows_only_one_writer() {
        let root = tempdir().expect("lease directory");
        let state = root.path().join("state.json");
        let first = DeliveryLease::acquire(&state).expect("first writer");
        assert!(DeliveryLease::acquire(&state).is_none());
        drop(first);
        assert!(DeliveryLease::acquire(&state).is_some());
    }

    #[test]
    fn lease_owner_probe_distinguishes_live_and_gone_processes() {
        let root = tempdir().expect("lease directory");
        let path = root.path().join("lease");
        fs::write(&path, format!("{}:0", std::process::id())).expect("live owner");
        assert_eq!(lease_owner_alive(&path), Some(true));
        #[cfg(windows)]
        let mut child = Command::new("cmd")
            .args(["/C", "exit", "0"])
            .spawn()
            .expect("short-lived owner");
        #[cfg(not(windows))]
        let mut child = Command::new("sh")
            .args(["-c", "exit 0"])
            .spawn()
            .expect("short-lived owner");
        let child_pid = child.id();
        child.wait().expect("short-lived owner exits");
        fs::write(&path, format!("{child_pid}:0")).expect("gone owner");
        assert_eq!(lease_owner_alive(&path), Some(false));
    }

    #[test]
    fn threshold_delivery_has_one_step_and_one_reminder() {
        let mut config = AgentFeedbackConfig::default();
        config.trajectory.signals.unintegrated.step_minutes = 120;
        let snapshot = crate::cli::agent_trajectory::test_snapshot(0, true);
        let mut state = DeliveryState {
            schema: STATE_SCHEMA.to_string(),
            first_pending_at: Some(0),
            ..Default::default()
        };

        assert!(threshold_selection(&mut state, &snapshot, 1_799, &config).is_none());
        let first =
            threshold_selection(&mut state, &snapshot, 1_800, &config).expect("entry threshold");
        assert_eq!(first.key, "pending:0");
        assert!(!first.reminder);
        state.signals.pending.last_key = Some(first.key.clone());
        state.last_sent_at = Some(1_800);
        assert!(threshold_selection(&mut state, &snapshot, 1_801, &config).is_none());
        let reminder =
            threshold_selection(&mut state, &snapshot, 3_600, &config).expect("one reminder");
        assert_eq!(reminder.key, "pending:0");
        assert!(reminder.reminder);
        state.signals.pending.reminders_sent = 1;
        assert!(threshold_selection(&mut state, &snapshot, 5_400, &config).is_none());
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
