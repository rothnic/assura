//! Process lifecycle management for `assura daemon`.

use super::{process, DaemonTextRender};
use crate::daemon::{serialize_optional_path, serialize_path, DaemonHealth};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const DAEMON_PROTOCOL_VERSION: &str = "assura.daemon.v1";

pub(super) fn daemon_start_output(
    health: DaemonHealth,
    project_loaded: bool,
) -> DaemonLifecycleOutput {
    start_like_output("start", health, project_loaded)
}

pub(super) fn daemon_restart_output(
    health: DaemonHealth,
    project_loaded: bool,
) -> DaemonLifecycleOutput {
    start_like_output("restart", health, project_loaded)
}

pub(super) fn daemon_stop_output(health: DaemonHealth) -> DaemonLifecycleOutput {
    let prior = read_runtime_status(&health.runtime_paths.status_file);
    let changed = prior
        .as_ref()
        .is_some_and(|status| verified_runtime_health(status).is_some());
    if let Some(status) = &prior {
        stop_verified_process(status);
    }
    let status = RuntimeStatus::stopped(
        health.clone(),
        if changed {
            "daemon process stopped"
        } else {
            "daemon process already stopped"
        },
    );
    let write_error = write_runtime_status(&health.runtime_paths.status_file, &status).err();
    append_log(
        &health,
        if changed { "stop" } else { "stop-idempotent" },
        &status.message,
    );
    DaemonLifecycleOutput::from_status("stop", changed, status, write_error)
}

pub(super) fn daemon_logs_output(health: DaemonHealth, tail: usize) -> DaemonLogsOutput {
    let log_file = health.runtime_paths.log_file.clone();
    let raw_lines = fs::read_to_string(&health.runtime_paths.log_file)
        .map(|content| content.lines().map(ToString::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let total = raw_lines.len();
    let start = total.saturating_sub(tail);
    DaemonLogsOutput {
        schema: "assura.daemon.logs.v1",
        protocol_version: DAEMON_PROTOCOL_VERSION,
        health,
        log_file,
        tail,
        total_lines: total,
        returned_lines: total - start,
        truncated: start > 0,
        lines: raw_lines.into_iter().skip(start).collect(),
    }
}

fn start_like_output(
    action: &'static str,
    health: DaemonHealth,
    project_loaded: bool,
) -> DaemonLifecycleOutput {
    if !project_loaded {
        return DaemonLifecycleOutput::unavailable(action, health);
    }
    let prior = read_runtime_status(&health.runtime_paths.status_file);
    let already_started = prior.as_ref().is_some_and(fresh_runtime_is_reachable);
    let changed = action == "restart" || !already_started;
    if changed {
        if let Some(status) = &prior {
            stop_verified_process(status);
        }
    }
    let status = if changed {
        match process::spawn_daemon(
            &health.project_root,
            Some(&health.config_path),
            &process::default_listen_addr(&health.project_root),
            &health.runtime_paths.log_file,
        ) {
            Ok(spawned) => RuntimeStatus::started_process(
                health.clone(),
                spawned.pid,
                spawned.listen_addr,
                spawned.socket_path,
                if action == "restart" {
                    "daemon process restarted"
                } else {
                    "daemon process started"
                },
            ),
            Err(error) => RuntimeStatus::failed(
                health.clone(),
                format!("failed to start daemon process: {error}"),
            ),
        }
    } else {
        RuntimeStatus::started_process(
            health.clone(),
            prior
                .as_ref()
                .and_then(|status| status.pid)
                .unwrap_or_default(),
            prior
                .as_ref()
                .and_then(|status| status.listen_addr.clone())
                .unwrap_or_else(|| process::default_listen_addr(&health.project_root)),
            prior.as_ref().and_then(|status| status.socket_path.clone()),
            "daemon process already started",
        )
    };
    let write_error = write_runtime_status(&health.runtime_paths.status_file, &status).err();
    append_log(
        &health,
        if changed { action } else { "start-idempotent" },
        &status.message,
    );
    DaemonLifecycleOutput::from_status(action, changed, status, write_error)
}

pub(super) fn runtime_status_for_health(health: &DaemonHealth) -> DaemonRuntimeStatus {
    read_runtime_status(&health.runtime_paths.status_file)
        .map(DaemonRuntimeStatus::from_runtime_status)
        .unwrap_or_else(|| DaemonRuntimeStatus {
            state: "not_started".to_string(),
            running: false,
            pid: None,
            socket_path: None,
            listen_addr: None,
            mode: "local_probe".to_string(),
            message: "managed daemon runtime metadata has not been started".to_string(),
            updated_at_unix: None,
        })
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DaemonRuntimeStatus {
    pub(super) state: String,
    pub(super) running: bool,
    pub(super) pid: Option<u32>,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) socket_path: Option<PathBuf>,
    pub(super) listen_addr: Option<String>,
    pub(super) mode: String,
    pub(super) message: String,
    pub(super) updated_at_unix: Option<u64>,
}

impl DaemonRuntimeStatus {
    fn from_runtime_status(status: RuntimeStatus) -> Self {
        let mut status = status;
        if status.state == "started" {
            let probed_health = verified_runtime_health(&status);
            if probed_health.is_none() {
                status.state = "crashed".to_string();
                status.running = false;
                status.message = "daemon process is not reachable".to_string();
            } else if let Some(health) = probed_health {
                match health.state {
                    crate::daemon::DaemonHealthState::Running => {
                        status.running = true;
                        status.message = "daemon process is running".to_string();
                    }
                    state => {
                        status.state = health_state_label(state).to_string();
                        status.running = false;
                        status.message = health.reason;
                    }
                }
            }
        }
        Self {
            state: status.state,
            running: status.running,
            pid: status.pid,
            socket_path: status.socket_path,
            listen_addr: status.listen_addr,
            mode: status.mode,
            message: status.message,
            updated_at_unix: Some(status.updated_at_unix),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonLifecycleOutput {
    schema: &'static str,
    protocol_version: &'static str,
    action: &'static str,
    changed: bool,
    health: DaemonHealth,
    runtime: RuntimeStatus,
    error: Option<String>,
}

impl DaemonLifecycleOutput {
    fn from_status(
        action: &'static str,
        changed: bool,
        runtime: RuntimeStatus,
        error: Option<std::io::Error>,
    ) -> Self {
        Self {
            schema: "assura.daemon.lifecycle.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            action,
            changed,
            health: runtime.health.clone(),
            runtime,
            error: error.map(|error| error.to_string()),
        }
    }

    pub(super) fn succeeded(&self) -> bool {
        self.error.is_none() && self.runtime.state != "failed"
    }

    fn unavailable(action: &'static str, health: DaemonHealth) -> Self {
        Self {
            schema: "assura.daemon.lifecycle.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            action,
            changed: false,
            runtime: RuntimeStatus::unavailable(health.clone()),
            health,
            error: Some("project state could not be loaded".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonLogsOutput {
    schema: &'static str,
    protocol_version: &'static str,
    health: DaemonHealth,
    #[serde(serialize_with = "serialize_path")]
    log_file: PathBuf,
    tail: usize,
    total_lines: usize,
    returned_lines: usize,
    truncated: bool,
    lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeStatus {
    schema: String,
    protocol_version: String,
    state: String,
    running: bool,
    pid: Option<u32>,
    #[serde(serialize_with = "serialize_optional_path")]
    socket_path: Option<PathBuf>,
    listen_addr: Option<String>,
    mode: String,
    message: String,
    updated_at_unix: u64,
    health: DaemonHealth,
}

impl RuntimeStatus {
    fn started_process(
        health: DaemonHealth,
        pid: u32,
        listen_addr: String,
        socket_path: Option<PathBuf>,
        message: &str,
    ) -> Self {
        Self {
            schema: "assura.daemon.runtime-status.v1".to_string(),
            protocol_version: DAEMON_PROTOCOL_VERSION.to_string(),
            state: "started".to_string(),
            running: true,
            pid: Some(pid),
            socket_path,
            listen_addr: Some(listen_addr),
            mode: "managed_process".to_string(),
            message: message.to_string(),
            updated_at_unix: process::unix_now(),
            health,
        }
    }

    fn stopped(health: DaemonHealth, message: &str) -> Self {
        Self::new("stopped", health, message)
    }

    fn failed(health: DaemonHealth, message: String) -> Self {
        Self::new("failed", health, &message)
    }

    fn unavailable(health: DaemonHealth) -> Self {
        Self::new("unavailable", health, "project state could not be loaded")
    }

    fn new(state: &str, health: DaemonHealth, message: &str) -> Self {
        Self {
            schema: "assura.daemon.runtime-status.v1".to_string(),
            protocol_version: DAEMON_PROTOCOL_VERSION.to_string(),
            state: state.to_string(),
            running: false,
            pid: None,
            socket_path: None,
            listen_addr: None,
            mode: "managed_runtime_metadata".to_string(),
            message: message.to_string(),
            updated_at_unix: process::unix_now(),
            health,
        }
    }
}

impl DaemonTextRender for DaemonLifecycleOutput {
    fn render_text(&self) -> String {
        format!(
            "daemon {}: state={} changed={} message={} status_file={}",
            self.action,
            self.runtime.state,
            self.changed,
            self.runtime.message,
            self.runtime.health.runtime_paths.status_file.display()
        )
    }
}

impl DaemonTextRender for DaemonLogsOutput {
    fn render_text(&self) -> String {
        if self.lines.is_empty() {
            return format!("daemon logs: {} lines", self.returned_lines);
        }
        self.lines.join("\n")
    }
}

fn read_runtime_status(path: &Path) -> Option<RuntimeStatus> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_runtime_status(path: &Path, status: &RuntimeStatus) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(status)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    fs::write(path, format!("{content}\n"))
}

fn append_log(health: &DaemonHealth, action: &str, message: &str) {
    let Some(parent) = health.runtime_paths.log_file.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let line = format!(
        "{} action={} message={}\n",
        process::unix_now(),
        action,
        message
    );
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&health.runtime_paths.log_file)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(line.as_bytes())
        });
}

fn fresh_runtime_is_reachable(status: &RuntimeStatus) -> bool {
    verified_runtime_health(status)
        .is_some_and(|health| health.state == crate::daemon::DaemonHealthState::Running)
}

fn verified_runtime_health(status: &RuntimeStatus) -> Option<DaemonHealth> {
    if status.state != "started" {
        return None;
    }
    let pid = status.pid?;
    if !process::process_is_running(pid) {
        return None;
    }
    let listen_addr = status.listen_addr.as_deref()?;
    let health = process::probe_health(listen_addr).ok()?;
    if process::same_daemon_identity(&status.health, &health) {
        Some(health)
    } else {
        None
    }
}

fn stop_verified_process(status: &RuntimeStatus) {
    if verified_runtime_health(status).is_none() {
        return;
    }
    if let Some(listen_addr) = status.listen_addr.as_deref() {
        let _ = process::request_shutdown(listen_addr);
    }
    thread::sleep(Duration::from_millis(100));
    if verified_runtime_health(status).is_some() {
        if let Some(pid) = status.pid {
            let _ = process::stop_process(pid);
        }
    }
}

fn health_state_label(state: crate::daemon::DaemonHealthState) -> &'static str {
    match state {
        crate::daemon::DaemonHealthState::Warming => "warming",
        crate::daemon::DaemonHealthState::Running => "started",
        crate::daemon::DaemonHealthState::Stale => "stale",
        crate::daemon::DaemonHealthState::Degraded => "degraded",
        crate::daemon::DaemonHealthState::Unavailable => "unavailable",
        crate::daemon::DaemonHealthState::Incompatible => "incompatible",
    }
}
