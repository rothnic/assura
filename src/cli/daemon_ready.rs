//! Atomic readiness handshake for managed daemon startup.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DAEMON_READY_SCHEMA: &str = "assura.daemon.ready.v1";
pub(super) const DAEMON_READY_FILE_ENV: &str = "ASSURA_DAEMON_READY_FILE";

pub(super) fn ready_file_from_env() -> Option<PathBuf> {
    std::env::var_os(DAEMON_READY_FILE_ENV).map(PathBuf::from)
}

pub(super) fn ready_file_for(log_file: &Path) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    log_file
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("ready-{}-{suffix}.json", std::process::id()))
}

pub(super) fn cleanup_ready_files(ready_file: &Path) -> Result<(), String> {
    for path in [
        ready_file.to_path_buf(),
        ready_file.with_extension("json.tmp"),
    ] {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "remove daemon readiness artifact {}: {error}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn wait_for_daemon_address(
    child: &mut Child,
    ready_file: &Path,
    timeout: Duration,
    protocol_version: &str,
    expected_pid: u32,
) -> Result<String, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if ready_file.is_file() {
            return read_daemon_address(ready_file, protocol_version, expected_pid);
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("inspect daemon startup: {error}"))?
        {
            return Err(format!(
                "daemon exited before publishing an IPC address: {status}"
            ));
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "daemon did not publish an IPC address within {} seconds",
                timeout.as_secs()
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn read_daemon_address(
    ready_file: &Path,
    protocol_version: &str,
    expected_pid: u32,
) -> Result<String, String> {
    let raw = fs::read_to_string(ready_file)
        .map_err(|error| format!("read daemon readiness: {error}"))?;
    let ready: DaemonReadyOutput =
        serde_json::from_str(&raw).map_err(|error| format!("parse daemon readiness: {error}"))?;
    if ready.schema != DAEMON_READY_SCHEMA {
        return Err(format!(
            "daemon readiness schema mismatch: expected {DAEMON_READY_SCHEMA}, got {}",
            ready.schema
        ));
    }
    if ready.protocol_version != protocol_version {
        return Err(format!(
            "daemon protocol mismatch: expected {protocol_version}, got {}",
            ready.protocol_version
        ));
    }
    if ready.pid != expected_pid {
        return Err(format!(
            "daemon readiness pid mismatch: expected {expected_pid}, got {}",
            ready.pid
        ));
    }
    if ready.listen_addr.trim().is_empty() {
        return Err("daemon readiness omitted an IPC address".to_string());
    }
    Ok(ready.listen_addr)
}

pub(super) fn publish_daemon_address(
    ready_file: Option<&Path>,
    project_root: &Path,
    listen_addr: &str,
    protocol_version: &str,
) -> Result<(), String> {
    let Some(ready_file) = ready_file else {
        println!("ASSURA_DAEMON_ADDR\t{listen_addr}");
        return std::io::stdout()
            .flush()
            .map_err(|error| format!("publish daemon address: {error}"));
    };
    validate_ready_file(ready_file, project_root)?;
    let parent = ready_file
        .parent()
        .ok_or_else(|| "daemon readiness path omitted a parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("create daemon readiness dir: {error}"))?;
    let temporary = ready_file.with_extension("json.tmp");
    let payload = serde_json::to_vec(&DaemonReadyOutput {
        schema: DAEMON_READY_SCHEMA.to_string(),
        protocol_version: protocol_version.to_string(),
        pid: std::process::id(),
        listen_addr: listen_addr.to_string(),
    })
    .map_err(|error| format!("encode daemon readiness: {error}"))?;
    if let Err(error) = fs::write(&temporary, payload) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("write daemon readiness: {error}"));
    }
    fs::rename(&temporary, ready_file).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("publish daemon readiness: {error}")
    })
}

fn validate_ready_file(ready_file: &Path, project_root: &Path) -> Result<(), String> {
    let expected_parent = project_root.join(".assura").join("daemon");
    let file_name = ready_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if ready_file.parent() != Some(expected_parent.as_path())
        || !file_name.starts_with("ready-")
        || !file_name.ends_with(".json")
    {
        return Err(format!(
            "daemon readiness path must be a ready-*.json file under {}",
            expected_parent.display()
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct DaemonReadyOutput {
    schema: String,
    protocol_version: String,
    pid: u32,
    listen_addr: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn readiness_round_trip_requires_the_publishing_process() {
        let project = TempDir::new().unwrap();
        let ready_file = project.path().join(".assura/daemon/ready-test.json");

        publish_daemon_address(
            Some(&ready_file),
            project.path(),
            "127.0.0.1:4321",
            "assura.daemon.test",
        )
        .unwrap();

        assert_eq!(
            read_daemon_address(&ready_file, "assura.daemon.test", std::process::id()).unwrap(),
            "127.0.0.1:4321"
        );
        assert!(read_daemon_address(
            &ready_file,
            "assura.daemon.test",
            std::process::id().wrapping_add(1)
        )
        .unwrap_err()
        .contains("pid mismatch"));
        cleanup_ready_files(&ready_file).unwrap();
        assert!(!ready_file.exists());
    }

    #[test]
    fn readiness_rejects_paths_outside_the_project_runtime_directory() {
        let project = TempDir::new().unwrap();
        let outside = project.path().join("ready-test.json");

        let error = publish_daemon_address(
            Some(&outside),
            project.path(),
            "127.0.0.1:4321",
            "assura.daemon.test",
        )
        .unwrap_err();

        let expected_parent = project.path().join(".assura").join("daemon");
        assert!(
            error.contains(&expected_parent.display().to_string()),
            "unexpected error: {error}"
        );
        assert!(!outside.exists());
    }
}
