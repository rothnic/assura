//! Bounded background refresh worker lifecycle for automatic feedback.

use super::{
    acquire_refresh_worker_lease, claim_refresh_queue, discard_refresh_lease, mark_refresh_queued,
    now, now_millis, owner_alive_for_token, process_owner_alive, prune_feedback_sidecars,
    read_state, reclaim_lease_if_token, refresh_lease_is_stale, refresh_lease_owned,
    refresh_lease_pid, refresh_lease_token, refresh_lock_path, release_refresh_lease_if_owned,
    replace_refresh_lease_if_owned, restore_refresh_queue, state_is_valid, state_path, write_state,
    AgentFeedbackConfig, DeliveryLease, LeaseMutex, HANDOFF_GRACE_MILLIS, REFRESH_MIN_INTERVAL_ENV,
    REFRESH_READY_ENV, REFRESH_SUPERVISOR_CLAIM_ENV, REFRESH_SUPERVISOR_DEADLINE_ENV,
    REFRESH_SUPERVISOR_ENV, REFRESH_SUPERVISOR_PID_ENV, REFRESH_TOKEN_ENV,
};

use crate::cli::agent_trajectory;

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub(super) fn request_refresh(
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
    let state_path = state_path(project_root);
    if !state_is_valid(&state_path) {
        return "state_corrupt";
    }
    let Some(state_lease) = DeliveryLease::acquire(&state_path) else {
        mark_refresh_queued(&lock);
        return "in_flight";
    };
    let Some(refresh_guard) = LeaseMutex::try_acquire(&lock) else {
        mark_refresh_queued(&lock);
        return "in_flight";
    };
    prune_feedback_sidecars(parent, now);
    if let Ok(metadata) = fs::metadata(&lock) {
        let observed_token = refresh_lease_token(&lock).ok();
        let malformed = observed_token
            .as_deref()
            .and_then(refresh_lease_pid)
            .is_none();
        let stale = refresh_lease_is_stale(&metadata, now, config.collection.timeout_ms);
        let reclaimed = !malformed
            && stale
            && observed_token.as_deref().is_some_and(|token| {
                owner_alive_for_token(token) == Some(false)
                    && reclaim_lease_if_token(&lock, token, &refresh_guard)
            });
        if !reclaimed {
            let queued = lock.with_extension("queued");
            let _ = OpenOptions::new().write(true).create_new(true).open(queued);
            return "in_flight";
        }
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
    let previous_state = state.clone();
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
        let _ = discard_refresh_lease(&lock);
        return "unavailable";
    }
    state.last_refresh_at = Some(now);
    state.last_refresh_at_ms = Some(now_millis());
    state.last_refresh_event = Some(event.to_string());
    if write_state(&state_path, &state).is_err() {
        let _ = release_refresh_lease_if_owned(&lock, &token, &refresh_guard);
        return "unavailable";
    }
    let queued = lock.with_extension("queued");
    let claimed_queue = claim_refresh_queue(&queued);
    let executable = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => {
            if let Some(claimed) = claimed_queue.as_deref() {
                restore_refresh_queue(claimed, &queued);
            }
            let _ = write_state(&state_path, &previous_state);
            let _ = release_refresh_lease_if_owned(&lock, &token, &refresh_guard);
            return "unavailable";
        }
    };
    let deadline = now_millis().saturating_add(config.collection.timeout_ms as i64);
    let args = refresh_args(project_root);
    if !spawn_refresh_pair(
        &executable,
        &args,
        &lock,
        &token,
        &refresh_guard,
        config.collection.timeout_ms,
        config.collection.min_refresh_seconds,
        deadline,
        claimed_queue.as_deref(),
    ) {
        if let Some(claimed) = claimed_queue.as_deref() {
            restore_refresh_queue(claimed, &queued);
        }
        let _ = write_state(&state_path, &previous_state);
        let _ = release_refresh_lease_if_owned(&lock, &token, &refresh_guard);
        return "unavailable";
    }
    drop(state_lease);
    "scheduled"
}

fn refresh_args(project_root: &Path) -> Vec<std::ffi::OsString> {
    vec![
        "agent".into(),
        "nudge".into(),
        project_root.as_os_str().to_os_string(),
        "--delivery".into(),
        "inspect".into(),
        "--format".into(),
        "json".into(),
    ]
}

// allow-reason: the spawn boundary keeps worker, lease, cadence, and deadline inputs explicit.
#[allow(clippy::too_many_arguments)]
fn spawn_refresh_pair(
    executable: &Path,
    args: &[std::ffi::OsString],
    lock: &Path,
    token: &str,
    guard: &LeaseMutex,
    timeout_ms: u64,
    min_refresh_seconds: u64,
    deadline: i64,
    claimed_queue: Option<&Path>,
) -> bool {
    let Some(parent) = lock.parent() else {
        return false;
    };
    let Some(lock_name) = lock.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    let ready = parent.join(format!(
        ".{lock_name}.{}.{}.ready",
        std::process::id(),
        now_millis()
    ));
    let mut worker = Command::new(executable);
    worker
        .args(args)
        .env("ASSURA_FEEDBACK_REFRESH_LOCK", lock)
        .env(REFRESH_TOKEN_ENV, token)
        .env("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS", timeout_ms.to_string())
        .env(REFRESH_MIN_INTERVAL_ENV, min_refresh_seconds.to_string())
        .env("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS", deadline.to_string())
        .env(REFRESH_READY_ENV, &ready)
        .env_remove(REFRESH_SUPERVISOR_ENV)
        .env_remove(REFRESH_SUPERVISOR_PID_ENV)
        .env_remove(REFRESH_SUPERVISOR_DEADLINE_ENV)
        .env_remove(REFRESH_SUPERVISOR_CLAIM_ENV)
        .env_remove("ASSURA_AGENT_LOG")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    agent_trajectory::isolate_process_tree(&mut worker);
    let mut child = match worker.spawn() {
        Ok(child) => child,
        Err(_) => return false,
    };
    let child_token = format!("{}:{}", child.id(), now_millis());
    if !replace_refresh_lease_if_owned(lock, token, &child_token, guard) {
        agent_trajectory::terminate_process_tree(child.id());
        let _ = child.wait();
        let _ = fs::remove_file(&ready);
        return false;
    }

    let mut supervisor = Command::new(executable);
    supervisor
        .args(args)
        .env(REFRESH_SUPERVISOR_ENV, "1")
        .env(REFRESH_SUPERVISOR_PID_ENV, child.id().to_string())
        .env(REFRESH_SUPERVISOR_DEADLINE_ENV, deadline.to_string())
        .env("ASSURA_FEEDBACK_REFRESH_LOCK", lock)
        .env(REFRESH_TOKEN_ENV, token)
        .env("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS", timeout_ms.to_string())
        .env(REFRESH_MIN_INTERVAL_ENV, min_refresh_seconds.to_string())
        .env("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS", deadline.to_string())
        .env(REFRESH_READY_ENV, &ready)
        .env_remove("ASSURA_AGENT_LOG")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(claimed_queue) = claimed_queue {
        supervisor.env(REFRESH_SUPERVISOR_CLAIM_ENV, claimed_queue);
    }
    let supervisor_spawned = supervisor.spawn().is_ok();
    if !supervisor_spawned {
        agent_trajectory::terminate_process_tree(child.id());
        let _ = child.wait();
        let _ = replace_refresh_lease_if_owned(lock, &child_token, token, guard);
        let _ = fs::remove_file(&ready);
    }
    supervisor_spawned
}

pub(in crate::cli) fn run_refresh_supervisor_if_requested() -> bool {
    if std::env::var(REFRESH_SUPERVISOR_ENV).ok().as_deref() != Some("1") {
        return false;
    }
    let pid = std::env::var(REFRESH_SUPERVISOR_PID_ENV)
        .ok()
        .and_then(|value| value.parse::<u32>().ok());
    let deadline = std::env::var(REFRESH_SUPERVISOR_DEADLINE_ENV)
        .ok()
        .and_then(|value| value.parse::<i64>().ok());
    let lock = std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK").map(PathBuf::from);
    let token = std::env::var(REFRESH_TOKEN_ENV).ok();
    let ready = std::env::var_os(REFRESH_READY_ENV).map(PathBuf::from);
    let claimed_queue = std::env::var_os(REFRESH_SUPERVISOR_CLAIM_ENV).map(PathBuf::from);
    if let (Some(pid), Some(deadline), Some(lock), Some(token)) = (pid, deadline, lock, token) {
        supervise_refresh(
            pid,
            deadline,
            &lock,
            &token,
            ready.as_deref(),
            claimed_queue.as_deref(),
        );
    }
    true
}

pub(super) fn supervise_refresh(
    pid: u32,
    deadline: i64,
    lock: &Path,
    token: &str,
    ready: Option<&Path>,
    claimed_queue: Option<&Path>,
) {
    while process_owner_alive(pid) != Some(false) && now_millis() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    if process_owner_alive(pid) != Some(false) {
        agent_trajectory::terminate_process_tree(pid);
        let cleanup_deadline = now_millis().saturating_add(HANDOFF_GRACE_MILLIS);
        while process_owner_alive(pid) != Some(false) && now_millis() < cleanup_deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    settle_refresh_queue(lock, ready, claimed_queue);
    if let Some(active_token) = refresh_token_for_worker(lock, pid, token) {
        finish_refresh_with_deadline(
            lock,
            &active_token,
            now_millis().saturating_add(HANDOFF_GRACE_MILLIS),
        );
    }
}

pub(super) fn settle_refresh_queue(
    lock: &Path,
    ready: Option<&Path>,
    claimed_queue: Option<&Path>,
) {
    let refresh_ready = ready.is_some_and(Path::is_file);
    if let Some(claimed_queue) = claimed_queue {
        let queued = lock.with_extension("queued");
        if refresh_ready {
            let _ = fs::remove_file(claimed_queue);
        } else {
            restore_refresh_queue(claimed_queue, &queued);
        }
    }
    if let Some(ready) = ready {
        let _ = fs::remove_file(ready);
    }
}

fn refresh_token_for_worker(path: &Path, pid: u32, provisional: &str) -> Option<String> {
    let current = refresh_lease_token(path).ok()?;
    (current == provisional || refresh_lease_pid(&current) == Some(pid)).then_some(current)
}

/// Release a refresh lease when the short-lived inspect child exits.
pub(in crate::cli) fn finish_refresh() {
    if let Some(path) = std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK") {
        let path = PathBuf::from(path);
        let Some(token) = std::env::var_os(REFRESH_TOKEN_ENV) else {
            return;
        };
        let token = token.to_string_lossy().into_owned();
        let wait_until = refresh_deadline_millis()
            .map(|deadline| deadline.saturating_add(HANDOFF_GRACE_MILLIS))
            .unwrap_or_else(|| now_millis().saturating_add(500));
        finish_refresh_with_deadline(&path, &token, wait_until);
    }
}

fn finish_refresh_with_deadline(path: &Path, token: &str, wait_until: i64) {
    let min_interval = std::env::var(REFRESH_MIN_INTERVAL_ENV)
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_default();
    finish_refresh_with_min_interval(path, token, wait_until, min_interval);
}

pub(super) fn finish_refresh_with_min_interval(
    path: &Path,
    token: &str,
    wait_until: i64,
    min_interval: i64,
) {
    let state_path = path.with_extension("json");
    let Some(refresh_guard) = LeaseMutex::acquire_until(path, wait_until) else {
        return;
    };
    let Some(_state_lease) = DeliveryLease::acquire(&state_path) else {
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    };
    if let Some(parent) = path.parent() {
        prune_feedback_sidecars(parent, now());
    }
    if !refresh_lease_owned(path, token) {
        return;
    }
    let queued = path.with_extension("queued");
    if !queued.is_file() {
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    }

    let mut state = read_state(&state_path);
    if state
        .last_refresh_at
        .is_some_and(|last| now().saturating_sub(last) < min_interval)
    {
        // Leave the queue marker for the next event after cooldown; do not
        // start a refresh chain that bypasses the shared cadence.
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    }

    let Some(claimed) = claim_refresh_queue(&queued) else {
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    };
    let previous_state = state.clone();
    let handoff_at = now();
    state.last_refresh_at = Some(handoff_at);
    state.last_refresh_at_ms = Some(now_millis());
    state.last_refresh_event = Some("queued_handoff".to_string());
    if write_state(&state_path, &state).is_err() {
        restore_refresh_queue(&claimed, &queued);
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    }

    let next_token = format!("{}:{}", std::process::id(), now_millis());
    if !replace_refresh_lease_if_owned(path, token, &next_token, &refresh_guard) {
        let _ = write_state(&state_path, &previous_state);
        restore_refresh_queue(&claimed, &queued);
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
        return;
    }
    let spawned = std::env::current_exe().ok().is_some_and(|executable| {
        let args = std::env::args_os().skip(1).collect::<Vec<_>>();
        let timeout_ms = std::env::var("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(2_000);
        spawn_refresh_pair(
            &executable,
            &args,
            path,
            &next_token,
            &refresh_guard,
            timeout_ms,
            min_interval.max(0) as u64,
            now_millis().saturating_add(timeout_ms as i64),
            Some(&claimed),
        )
    });
    if !spawned {
        let _ = replace_refresh_lease_if_owned(path, &next_token, token, &refresh_guard);
        let _ = write_state(&state_path, &previous_state);
        restore_refresh_queue(&claimed, &queued);
        let _ = release_refresh_lease_if_owned(path, token, &refresh_guard);
    }
}

/// Bound the whole inspect worker, including config and daemon setup before Git.
pub(in crate::cli) fn start_refresh_watchdog() -> bool {
    let Some(lock) = std::env::var_os("ASSURA_FEEDBACK_REFRESH_LOCK") else {
        return true;
    };
    let Some(expected_token) = std::env::var_os(REFRESH_TOKEN_ENV) else {
        return true;
    };
    let lock = PathBuf::from(lock);
    let expected_token = expected_token.to_string_lossy().into_owned();
    let deadline = refresh_deadline_millis();
    let wait_until = deadline.unwrap_or_else(|| now_millis().saturating_add(500));
    let state_path = lock.with_extension("json");
    let Some(state_lease) = acquire_refresh_worker_lease(&state_path, wait_until) else {
        return false;
    };
    let Some(refresh_guard) = LeaseMutex::acquire_until(&lock, wait_until) else {
        return false;
    };
    let worker_token = format!("{}:{}", std::process::id(), now_millis());
    let current_token = refresh_lease_token(&lock).ok();
    let active_token = if current_token.as_deref() == Some(expected_token.as_str()) {
        if !replace_refresh_lease_if_owned(&lock, &expected_token, &worker_token, &refresh_guard) {
            return false;
        }
        worker_token
    } else if current_token.as_deref().and_then(refresh_lease_pid) != Some(std::process::id()) {
        return false;
    } else {
        // The parent promoted the provisional token to this child PID before
        // setup began; keep that generation for supervisor cleanup.
        current_token.expect("current worker token")
    };
    std::env::set_var(REFRESH_TOKEN_ENV, &active_token);
    agent_trajectory::reset_refresh_cancellation();
    drop(refresh_guard);
    drop(state_lease);
    if let Some(ready) = std::env::var_os(REFRESH_READY_ENV) {
        if fs::write(ready, b"ready").is_err() {
            return false;
        }
    }
    let Some(deadline) = deadline else {
        return true;
    };
    let remaining = deadline.saturating_sub(now_millis());
    if remaining <= 0 {
        agent_trajectory::cancel_refresh_worker();
        return true;
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(remaining as u64));
        if refresh_deadline_expired() {
            // Ask the bounded Git runner to terminate its child, then let the
            // command unwind through finish_refresh and release or hand off.
            agent_trajectory::cancel_refresh_worker();
        }
    });
    true
}

pub(in crate::cli) fn refresh_deadline_expired() -> bool {
    refresh_deadline_millis().is_some_and(|deadline| now_millis() >= deadline)
}

fn refresh_deadline_millis() -> Option<i64> {
    std::env::var("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
}
