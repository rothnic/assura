//! Lifecycle metadata management for `assura daemon`.

use super::DaemonTextRender;
use crate::daemon::{serialize_optional_path, serialize_path, DaemonHealth};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let changed = matches!(
        prior.as_ref().map(|status| status.state.as_str()),
        Some("started")
    );
    let status = RuntimeStatus::stopped(
        health.clone(),
        if changed {
            "daemon runtime metadata stopped"
        } else {
            "daemon runtime metadata already stopped"
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
    let already_started = matches!(
        prior.as_ref().map(|status| status.state.as_str()),
        Some("started")
    );
    let changed = action == "restart" || !already_started;
    let status = RuntimeStatus::started(
        health.clone(),
        if changed {
            if action == "restart" {
                "daemon runtime metadata restarted"
            } else {
                "daemon runtime metadata started"
            }
        } else {
            "daemon runtime metadata already started"
        },
    );
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
        .map(DaemonRuntimeStatus::from)
        .unwrap_or_else(|| DaemonRuntimeStatus {
            state: "not_started".to_string(),
            running: false,
            pid: None,
            socket_path: None,
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
    pub(super) mode: String,
    pub(super) message: String,
    pub(super) updated_at_unix: Option<u64>,
}

impl From<RuntimeStatus> for DaemonRuntimeStatus {
    fn from(status: RuntimeStatus) -> Self {
        Self {
            state: status.state,
            running: status.running,
            pid: status.pid,
            socket_path: status.socket_path,
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
    mode: String,
    message: String,
    updated_at_unix: u64,
    health: DaemonHealth,
}

impl RuntimeStatus {
    fn started(health: DaemonHealth, message: &str) -> Self {
        Self::new("started", health, message)
    }

    fn stopped(health: DaemonHealth, message: &str) -> Self {
        Self::new("stopped", health, message)
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
            mode: "managed_runtime_metadata".to_string(),
            message: message.to_string(),
            updated_at_unix: unix_now(),
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
    let line = format!("{} action={} message={}\n", unix_now(), action, message);
    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&health.runtime_paths.log_file)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(line.as_bytes())
        });
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
