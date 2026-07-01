//! Management-preview output contracts for `assura daemon`.

use super::DaemonTextRender;
use crate::daemon::{DaemonHealth, LocalDaemonCore};
use serde::Serialize;
use std::path::PathBuf;

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
    DaemonStatusOutput {
        schema: "assura.daemon.status.v1",
        protocol_version: DAEMON_PROTOCOL_VERSION,
        process: DaemonProcessStatus {
            running: false,
            pid: None,
            socket_path: None,
            mode: "local_probe",
            message: "managed daemon process is not started by this preview surface",
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
    process: DaemonProcessStatus,
    management: DaemonManagementCommands,
}

#[derive(Debug, Serialize)]
struct DaemonProcessStatus {
    running: bool,
    pid: Option<u32>,
    socket_path: Option<PathBuf>,
    mode: &'static str,
    message: &'static str,
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
            start: None,
            stop: None,
            restart: None,
            logs: None,
            fallback: health.fallback_command.clone(),
        }
    }
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
