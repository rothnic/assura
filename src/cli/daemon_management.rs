//! Management-preview output contracts for `assura daemon`.

use super::{lifecycle::runtime_status_for_health, DaemonTextRender};
use crate::daemon::{DaemonHealth, LocalDaemonCore};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const DAEMON_PROTOCOL_VERSION: &str = "assura.daemon.v1";

pub(super) fn health_for_path(path: PathBuf, config: Option<PathBuf>) -> (DaemonHealth, bool) {
    match LocalDaemonCore::load(path.clone(), config.clone()) {
        Ok(core) => (core.health(), true),
        Err(error) => {
            let config_path = config.unwrap_or_else(|| path.join(".assura/config.yml"));
            (
                DaemonHealth::unavailable(path, config_path, error.to_string()),
                false,
            )
        }
    }
}

pub(super) fn daemon_status_output(health: DaemonHealth) -> DaemonStatusOutput {
    let runtime = runtime_status_for_health(&health);
    DaemonStatusOutput {
        schema: "assura.daemon.status.v1",
        protocol_version: DAEMON_PROTOCOL_VERSION,
        project: DaemonProjectStatus::for_health(&health),
        process: DaemonProcessStatus {
            state: runtime.state,
            running: runtime.running,
            pid: runtime.pid,
            socket_path: runtime.socket_path,
            mode: runtime.mode,
            message: runtime.message,
            updated_at_unix: runtime.updated_at_unix,
        },
        management: DaemonManagementCommands::for_health(&health),
        health,
    }
}

pub(super) fn daemon_doctor_output(health: DaemonHealth, loaded: bool) -> DaemonDoctorOutput {
    let mut checks = Vec::new();
    checks.push(DaemonDoctorCheck {
        id: "project_state",
        status: if loaded { "ok" } else { "error" },
        message: if loaded {
            "project state loaded"
        } else {
            "project state could not be loaded"
        },
        remediation_command: if loaded {
            None
        } else {
            Some(health.fallback_command.clone())
        },
    });
    checks.push(DaemonDoctorCheck {
        id: "runtime_paths",
        status: "ok",
        message: "daemon runtime paths are project-local",
        remediation_command: Some(format!(
            "mkdir -p {}",
            health.runtime_paths.status_dir.display()
        )),
    });
    checks.push(DaemonDoctorCheck {
        id: "managed_process",
        status: "warning",
        message: "managed daemon process lifecycle is not running in this preview",
        remediation_command: Some("assura daemon status --format json".to_string()),
    });

    DaemonDoctorOutput {
        schema: "assura.daemon.doctor.v1",
        protocol_version: DAEMON_PROTOCOL_VERSION,
        health,
        checks,
    }
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonStatusOutput {
    schema: &'static str,
    protocol_version: &'static str,
    health: DaemonHealth,
    project: DaemonProjectStatus,
    process: DaemonProcessStatus,
    management: DaemonManagementCommands,
}

#[derive(Debug, Serialize)]
struct DaemonProjectStatus {
    project_root: PathBuf,
    config_path: PathBuf,
    config_fingerprint: Option<String>,
    dirty_paths: Vec<String>,
}

impl DaemonProjectStatus {
    fn for_health(health: &DaemonHealth) -> Self {
        Self {
            project_root: health.project_root.clone(),
            config_path: health.config_path.clone(),
            config_fingerprint: config_fingerprint(&health.config_path),
            dirty_paths: dirty_paths(&health.project_root),
        }
    }
}

#[derive(Debug, Serialize)]
struct DaemonProcessStatus {
    state: String,
    running: bool,
    pid: Option<u32>,
    socket_path: Option<PathBuf>,
    mode: String,
    message: String,
    updated_at_unix: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DaemonManagementCommands {
    status: String,
    doctor: String,
    start: Option<String>,
    stop: Option<String>,
    restart: Option<String>,
    logs: Option<String>,
    fallback: String,
}

impl DaemonManagementCommands {
    fn for_health(health: &DaemonHealth) -> Self {
        let root = health.project_root.display();
        Self {
            status: format!("assura daemon status --format json {root}"),
            doctor: format!("assura daemon doctor --format json {root}"),
            start: Some(format!("assura daemon start --format json {root}")),
            stop: Some(format!("assura daemon stop --format json {root}")),
            restart: Some(format!("assura daemon restart --format json {root}")),
            logs: Some(format!("assura daemon logs --format json {root}")),
            fallback: health.fallback_command.clone(),
        }
    }
}

fn config_fingerprint(path: &PathBuf) -> Option<String> {
    let content = fs::read(path).ok()?;
    Some(format!(
        "{:016x}",
        crate::stable_hash::stable_hash(&content)
    ))
}

fn dirty_paths(project_root: &PathBuf) -> Vec<String> {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(project_root)
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=all")
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[derive(Debug, Serialize)]
pub(super) struct DaemonDoctorOutput {
    schema: &'static str,
    protocol_version: &'static str,
    health: DaemonHealth,
    checks: Vec<DaemonDoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DaemonDoctorCheck {
    id: &'static str,
    status: &'static str,
    message: &'static str,
    remediation_command: Option<String>,
}

impl DaemonTextRender for DaemonStatusOutput {
    fn render_text(&self) -> String {
        format!(
            "{}\nprocess_running={}\nprocess_mode={}\ndoctor={}",
            self.health.render_text(),
            self.process.running,
            self.process.mode,
            self.management.doctor
        )
    }
}

impl DaemonTextRender for DaemonDoctorOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![self.health.render_text()];
        for check in &self.checks {
            lines.push(format!(
                "check={} status={} message={} remediation={}",
                check.id,
                check.status,
                check.message,
                check.remediation_command.as_deref().unwrap_or("-")
            ));
        }
        lines.join("\n")
    }
}
