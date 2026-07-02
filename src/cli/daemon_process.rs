//! Managed local daemon process and IPC helpers.

use super::transport::{ClientStream, Listener};
use crate::cli::check::StructureCheckReport;
use crate::daemon::{DaemonCoreError, DaemonHealth, LocalDaemonCore};
use serde::Serialize;
use serde_json::Value;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const DAEMON_PROTOCOL_VERSION: &str = "assura.daemon.v1";

pub(super) struct SpawnedDaemon {
    pub(super) pid: u32,
    pub(super) listen_addr: String,
    pub(super) socket_path: Option<PathBuf>,
}

pub(super) struct IpcResponse {
    pub(super) value: Value,
    pub(super) exit_code: i32,
}

pub(super) fn default_listen_addr(project_root: &Path) -> String {
    #[cfg(unix)]
    {
        format!(
            "unix:{}",
            project_root
                .join(".assura")
                .join("daemon")
                .join("assura.sock")
                .display()
        )
    }
    #[cfg(not(unix))]
    {
        let _ = project_root;
        "127.0.0.1:0".to_string()
    }
}

pub(super) fn spawn_daemon(
    project_root: &Path,
    config: Option<&Path>,
    listen_addr: &str,
    log_file: &Path,
) -> Result<SpawnedDaemon, String> {
    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create daemon log dir: {error}"))?;
    }
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|error| format!("open daemon log: {error}"))?;

    let mut command = Command::new(std::env::current_exe().map_err(|error| error.to_string())?);
    if let Some(config) = config {
        command.arg("--config").arg(config);
    }
    command
        .arg("daemon")
        .arg("serve")
        .arg(project_root)
        .arg("--listen")
        .arg(listen_addr)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(log));

    let mut child = command
        .spawn()
        .map_err(|error| format!("start daemon process: {error}"))?;

    let listen_addr = read_daemon_address(&mut child).map_err(|error| {
        let _ = child.kill();
        let _ = child.wait();
        error
    })?;
    let pid = child.id();
    Ok(SpawnedDaemon {
        pid,
        socket_path: socket_path_from_addr(&listen_addr),
        listen_addr,
    })
}

fn read_daemon_address(child: &mut Child) -> Result<String, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "daemon stdout was not captured".to_string())?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader
            .read_line(&mut line)
            .map_err(|error| format!("read daemon address: {error}"))?;
        if read == 0 {
            return Err("daemon exited before publishing an IPC address".to_string());
        }
        if let Some(addr) = line.trim().strip_prefix("ASSURA_DAEMON_ADDR\t") {
            return Ok(addr.to_string());
        }
    }
}

pub(super) fn probe_health(listen_addr: &str) -> Result<DaemonHealth, String> {
    let response = send_request(listen_addr, format!("PING\t{DAEMON_PROTOCOL_VERSION}\n"))?;
    let value: Value =
        serde_json::from_str(&response).map_err(|error| format!("parse daemon health: {error}"))?;
    let protocol = value
        .get("protocol_version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if protocol != DAEMON_PROTOCOL_VERSION {
        return Err(format!(
            "daemon protocol mismatch: expected {DAEMON_PROTOCOL_VERSION}, got {protocol}"
        ));
    }
    serde_json::from_value(
        value
            .get("health")
            .cloned()
            .ok_or_else(|| "daemon health response omitted health".to_string())?,
    )
    .map_err(|error| format!("decode daemon health: {error}"))
}

pub(super) fn same_daemon_identity(expected: &DaemonHealth, actual: &DaemonHealth) -> bool {
    paths_match(&expected.project_root, &actual.project_root)
        && paths_match(&expected.config_path, &actual.config_path)
}

pub(super) fn request_check_path(listen_addr: &str, changed: &Path) -> Result<IpcResponse, String> {
    let response = send_request(
        listen_addr,
        format!("CHECK-PATH\t{}\n", changed.to_string_lossy()),
    )?;
    let value: Value =
        serde_json::from_str(&response).map_err(|error| format!("parse daemon check: {error}"))?;
    let exit_code = if value.get("schema").and_then(Value::as_str) == Some("assura.daemon.error.v1")
    {
        3
    } else {
        0
    };
    Ok(IpcResponse { value, exit_code })
}

pub(super) fn request_shutdown(listen_addr: &str) -> Result<(), String> {
    let _ = send_request(listen_addr, "SHUTDOWN\n".to_string())?;
    Ok(())
}

fn send_request(listen_addr: &str, request: String) -> Result<String, String> {
    let mut stream = ClientStream::connect(listen_addr)?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| error.to_string())?;
    stream.flush().map_err(|error| error.to_string())?;
    stream.read_line()
}

pub(super) fn serve_daemon(
    project_root: PathBuf,
    config: Option<PathBuf>,
    listen_addr: String,
) -> Result<(), String> {
    let mut core =
        LocalDaemonCore::load(project_root, config).map_err(|error| error.to_string())?;
    let listener = Listener::bind(&listen_addr)?;
    println!("ASSURA_DAEMON_ADDR\t{}", listener.addr());
    let _ = std::io::stdout().flush();

    while let Some(mut stream) = listener.accept()? {
        let request = stream.read_line()?;
        let should_shutdown = request.trim_end() == "SHUTDOWN";
        let response = handle_request(&mut core, &request);
        stream
            .write_all(response.as_bytes())
            .map_err(|error| error.to_string())?;
        stream.write_all(b"\n").map_err(|error| error.to_string())?;
        stream.flush().map_err(|error| error.to_string())?;
        if should_shutdown {
            return Ok(());
        }
    }
    Ok(())
}

fn handle_request(core: &mut LocalDaemonCore, request: &str) -> String {
    let request = request.trim_end_matches(['\r', '\n']);
    if request == "SHUTDOWN" {
        return render_json(&DaemonPongOutput {
            schema: "assura.daemon.pong.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            health: core.health(),
        });
    }
    if request == "PING" || request == format!("PING\t{DAEMON_PROTOCOL_VERSION}") {
        return match core.probe_health() {
            Ok(health) => render_json(&DaemonPongOutput {
                schema: "assura.daemon.pong.v1",
                protocol_version: DAEMON_PROTOCOL_VERSION,
                health,
            }),
            Err(error) => render_json(&daemon_error(error, core.health())),
        };
    }
    if let Some(protocol) = request.strip_prefix("PING\t") {
        return render_json(&DaemonErrorOutput {
            schema: "assura.daemon.error.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            error: "daemon protocol mismatch".to_string(),
            health: DaemonHealth::incompatible(
                core.health().project_root,
                core.health().config_path,
                format!("client requested {protocol}; daemon serves {DAEMON_PROTOCOL_VERSION}"),
            ),
        });
    }
    if let Some(path) = request.strip_prefix("CHECK-PATH\t") {
        return match core.check_changed_path(PathBuf::from(path)) {
            Ok(report) => render_json(&DaemonCheckPathOutput {
                schema: "assura.daemon.check_path.v1",
                protocol_version: DAEMON_PROTOCOL_VERSION,
                health: core.health(),
                report,
            }),
            Err(error) => render_json(&daemon_error(error, core.health())),
        };
    }
    render_json(&DaemonErrorOutput {
        schema: "assura.daemon.error.v1",
        protocol_version: DAEMON_PROTOCOL_VERSION,
        error: "unsupported daemon request".to_string(),
        health: core.health(),
    })
}

fn daemon_error(error: DaemonCoreError, fallback_health: DaemonHealth) -> DaemonErrorOutput {
    match error {
        DaemonCoreError::Stale(health) => DaemonErrorOutput {
            schema: "assura.daemon.error.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            error: "daemon state is stale".to_string(),
            health: *health,
        },
        other => DaemonErrorOutput {
            schema: "assura.daemon.error.v1",
            protocol_version: DAEMON_PROTOCOL_VERSION,
            error: other.to_string(),
            health: fallback_health,
        },
    }
}

fn render_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|error| {
        format!(
            r#"{{"schema":"assura.daemon.error.v1","protocol_version":"{}","error":"{}"}}"#,
            DAEMON_PROTOCOL_VERSION, error
        )
    })
}

pub(super) fn process_is_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .output()
            .map(|output| {
                output.status.success()
                    && tasklist_contains_pid(&String::from_utf8_lossy(&output.stdout), pid)
            })
            .unwrap_or(false)
    }
}

#[cfg(windows)]
fn tasklist_contains_pid(output: &str, pid: u32) -> bool {
    let pid = pid.to_string();
    output.lines().any(|line| {
        line.split(',')
            .nth(1)
            .map(|field| field.trim().trim_matches('"') == pid)
            .unwrap_or(false)
    })
}

pub(super) fn stop_process(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

pub(super) fn socket_path_from_addr(addr: &str) -> Option<PathBuf> {
    addr.strip_prefix("unix:").map(PathBuf::from)
}

fn paths_match(expected: &Path, actual: &Path) -> bool {
    canonical_or_original(expected) == canonical_or_original(actual)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

pub(super) fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[derive(Debug, Serialize)]
struct DaemonPongOutput {
    schema: &'static str,
    protocol_version: &'static str,
    health: DaemonHealth,
}

#[derive(Debug, Serialize)]
struct DaemonCheckPathOutput {
    schema: &'static str,
    protocol_version: &'static str,
    health: DaemonHealth,
    report: StructureCheckReport,
}

#[derive(Debug, Serialize)]
struct DaemonErrorOutput {
    schema: &'static str,
    protocol_version: &'static str,
    error: String,
    health: DaemonHealth,
}
