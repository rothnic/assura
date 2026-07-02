use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn daemon_status_json_reports_management_contract() {
    let project = daemon_project();

    let json = assura_json(
        &project,
        &["daemon", "status", project.path_str(), "--format", "json"],
    );

    assert_eq!(json["schema"], "assura.daemon.status.v1");
    assert_eq!(json["protocol_version"], "assura.daemon.v1");
    assert_eq!(json["health"]["state"], "running");
    assert!(
        json["project"]["config_fingerprint"]
            .as_str()
            .unwrap()
            .len()
            >= 16
    );
    assert_eq!(json["process"]["running"], false);
    assert_eq!(json["process"]["state"], "not_started");
    assert_eq!(json["process"]["mode"], "local_probe");
    assert!(json["management"]["doctor"]
        .as_str()
        .unwrap()
        .contains("assura daemon doctor --format json"));
    assert!(json["management"]["start"]
        .as_str()
        .unwrap()
        .contains("assura daemon start --format json"));
}

#[test]
fn daemon_status_json_reports_git_dirty_paths() {
    let project = daemon_project();
    git(&project, &["init"]);
    git(&project, &["add", "."]);
    git(&project, &["commit", "-m", "init"]);
    fs::write(project.path().join("docs/new-note.md"), "# New\n").unwrap();

    let json = assura_json(
        &project,
        &["daemon", "status", project.path_str(), "--format", "json"],
    );

    assert!(json["project"]["dirty_paths"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path == "docs/new-note.md"));
}

#[test]
fn daemon_start_stop_json_are_idempotent_and_status_reflects_runtime() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["schema"], "assura.daemon.lifecycle.v1");
    assert_eq!(start["action"], "start");
    assert_eq!(start["changed"], true);
    assert_eq!(start["runtime"]["state"], "started");
    assert_eq!(start["runtime"]["running"], true);
    let started_pid = start["runtime"]["pid"].as_u64().unwrap() as u32;
    assert!(started_pid > 0);
    assert!(!start["runtime"]["listen_addr"].as_str().unwrap().is_empty());
    assert!(project.path().join(".assura/daemon/status.json").is_file());

    let repeat_start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(repeat_start["action"], "start");
    assert_eq!(repeat_start["changed"], false);
    assert_eq!(repeat_start["runtime"]["state"], "started");
    assert_eq!(repeat_start["runtime"]["running"], true);

    let status = assura_json(
        &project,
        &["daemon", "status", project.path_str(), "--format", "json"],
    );
    assert_eq!(status["process"]["state"], "started");
    assert_eq!(status["process"]["mode"], "managed_process");
    assert_eq!(status["process"]["running"], true);
    assert!(status["process"]["pid"].as_u64().unwrap() > 0);
    assert!(!status["process"]["listen_addr"]
        .as_str()
        .unwrap()
        .is_empty());

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["action"], "stop");
    assert_eq!(stop["changed"], true);
    assert_eq!(stop["runtime"]["state"], "stopped");
    wait_for_pid_to_exit(started_pid);

    let repeat_stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(repeat_stop["changed"], false);
    assert_eq!(repeat_stop["runtime"]["state"], "stopped");
}

#[test]
fn daemon_restart_and_logs_json_use_runtime_area() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["state"], "started");
    let first_pid = start["runtime"]["pid"].as_u64().unwrap() as u32;

    let restart = assura_json(
        &project,
        &["daemon", "restart", project.path_str(), "--format", "json"],
    );
    assert_eq!(restart["action"], "restart");
    assert_eq!(restart["changed"], true);
    assert_eq!(restart["runtime"]["state"], "started");
    let restart_pid = restart["runtime"]["pid"].as_u64().unwrap() as u32;
    assert_ne!(restart_pid, first_pid);
    wait_for_pid_to_exit(first_pid);

    let repeat_restart = assura_json(
        &project,
        &["daemon", "restart", project.path_str(), "--format", "json"],
    );
    assert_eq!(repeat_restart["action"], "restart");
    assert_eq!(repeat_restart["changed"], true);
    assert_eq!(repeat_restart["runtime"]["state"], "started");
    let repeat_restart_pid = repeat_restart["runtime"]["pid"].as_u64().unwrap() as u32;
    assert_ne!(repeat_restart_pid, restart_pid);
    wait_for_pid_to_exit(restart_pid);

    let logs = assura_json(
        &project,
        &[
            "daemon",
            "logs",
            project.path_str(),
            "--tail",
            "10",
            "--format",
            "json",
        ],
    );
    assert_eq!(logs["schema"], "assura.daemon.logs.v1");
    assert!(logs["log_file"]
        .as_str()
        .unwrap()
        .ends_with(".assura/daemon/daemon.log"));
    assert_eq!(logs["tail"], 10);
    assert!(logs["returned_lines"].as_u64().unwrap() <= 10);
    assert!(logs["lines"]
        .as_array()
        .unwrap()
        .iter()
        .any(|line| line.as_str().unwrap().contains("action=restart")));

    let truncated = assura_json(
        &project,
        &[
            "daemon",
            "logs",
            project.path_str(),
            "--tail",
            "2",
            "--format",
            "json",
        ],
    );
    assert_eq!(truncated["returned_lines"], 2);
    assert_eq!(truncated["truncated"], true);

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
    wait_for_pid_to_exit(repeat_restart_pid);
}

#[test]
fn daemon_doctor_json_reports_actionable_checks() {
    let project = daemon_project();

    let json = assura_json(
        &project,
        &["daemon", "doctor", project.path_str(), "--format", "json"],
    );

    assert_eq!(json["schema"], "assura.daemon.doctor.v1");
    assert_eq!(json["health"]["state"], "running");
    assert!(json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["id"] == "project_state" && check["status"] == "ok" }));
    assert!(json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["id"] == "managed_process" && check["status"] == "warning" }));
}

#[test]
fn daemon_doctor_json_reports_unavailable_project_with_remediation() {
    let project = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "doctor",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.daemon.doctor.v1");
    assert_eq!(json["health"]["state"], "unavailable");
    let project_state = json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| check["id"] == "project_state")
        .unwrap();
    assert_eq!(project_state["status"], "error");
    assert!(project_state["remediation_command"]
        .as_str()
        .unwrap()
        .contains("assura"));
}

#[test]
fn daemon_stop_and_logs_are_safe_when_project_is_unavailable() {
    let project = TempDir::new().unwrap();

    let stop_output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "stop",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(stop_output.status.success());
    let stop: Value = serde_json::from_slice(&stop_output.stdout).unwrap();
    assert_eq!(stop["schema"], "assura.daemon.lifecycle.v1");
    assert_eq!(stop["health"]["state"], "unavailable");
    assert_eq!(stop["changed"], false);
    assert_eq!(stop["runtime"]["state"], "stopped");

    let logs_output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "logs",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(logs_output.status.success());
    let logs: Value = serde_json::from_slice(&logs_output.stdout).unwrap();
    assert_eq!(logs["schema"], "assura.daemon.logs.v1");
    assert_eq!(logs["health"]["state"], "unavailable");
    assert_eq!(logs["total_lines"], 1);
}

#[test]
fn daemon_health_json_exposes_running_state_and_fallback() {
    let project = daemon_project();

    let json = assura_json(
        &project,
        &["daemon", "health", project.path_str(), "--format", "json"],
    );

    assert_eq!(json["state"], "running");
    assert_eq!(json["generation"], 1);
    assert!(json["fallback_command"]
        .as_str()
        .unwrap()
        .contains("assura check --format json"));
    assert!(json["runtime_paths"]["status_file"]
        .as_str()
        .unwrap()
        .ends_with(".assura/daemon/status.json"));
}

#[test]
fn daemon_check_path_json_wraps_structure_report_with_health() {
    let project = daemon_project();

    let json = assura_json(
        &project,
        &[
            "daemon",
            "check-path",
            project.path_str(),
            "--changed",
            "docs/note.md",
            "--format",
            "json",
        ],
    );

    assert_eq!(json["schema"], "assura.daemon.check_path.v1");
    assert_eq!(json["health"]["state"], "running");
    assert_eq!(json["report"]["success"], true);
}

#[test]
fn daemon_check_path_json_uses_running_ipc_process() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);

    let json = assura_json(
        &project,
        &[
            "daemon",
            "check-path",
            project.path_str(),
            "--changed",
            "docs/note.md",
            "--format",
            "json",
        ],
    );

    assert_eq!(json["schema"], "assura.daemon.check_path.v1");
    assert_eq!(json["protocol_version"], "assura.daemon.v1");
    assert_eq!(json["health"]["state"], "running");
    assert_eq!(json["report"]["success"], true);

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
fn daemon_check_path_json_reports_stale_config_from_running_ipc_process() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  docs/:
    files:
      naming_patterns:
        "*.md": snake_case
"#,
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "check-path",
            project.path_str(),
            "--changed",
            "docs/note.md",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.daemon.error.v1");
    assert_eq!(json["protocol_version"], "assura.daemon.v1");
    assert_eq!(json["health"]["state"], "stale");

    let status = assura_json(
        &project,
        &["daemon", "status", project.path_str(), "--format", "json"],
    );
    assert_eq!(status["process"]["state"], "stale");
    assert_eq!(status["process"]["running"], false);

    let doctor = assura_json(
        &project,
        &["daemon", "doctor", project.path_str(), "--format", "json"],
    );
    assert!(doctor["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["id"] == "managed_process" && check["status"] == "error" }));

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
fn daemon_stop_ignores_stale_metadata_for_unverified_pid() {
    let project = daemon_project();

    let mut child = Command::new(env!("CARGO_BIN_EXE_assura-full"))
        .args([
            "daemon",
            "serve",
            project.path_str(),
            "--listen",
            "127.0.0.1:0",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let pid = child.id();
    let status = serde_json::json!({
        "schema": "assura.daemon.runtime-status.v1",
        "protocol_version": "assura.daemon.v1",
        "state": "started",
        "running": true,
        "pid": pid,
        "socket_path": null,
        "listen_addr": "127.0.0.1:9",
        "mode": "managed_process",
        "message": "stale test fixture",
        "updated_at_unix": 1,
        "health": {
            "state": "running",
            "reason": "stale test fixture",
            "project_root": project.path_str(),
            "config_path": project.path().join(".assura/config.yml"),
            "generation": 1,
            "runtime_paths": {
                "status_dir": project.path().join(".assura/daemon"),
                "status_file": project.path().join(".assura/daemon/status.json"),
                "log_file": project.path().join(".assura/daemon/daemon.log")
            },
            "fallback_command": format!("assura check --format json {}", project.path_str())
        }
    });
    fs::create_dir_all(project.path().join(".assura/daemon")).unwrap();
    fs::write(
        project.path().join(".assura/daemon/status.json"),
        serde_json::to_string_pretty(&status).unwrap(),
    )
    .unwrap();

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["changed"], false);
    assert!(pid_is_running(pid));

    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn daemon_status_reports_crashed_process_without_fresh_running_state() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    let pid = start["runtime"]["pid"].as_u64().unwrap() as u32;
    terminate_pid(pid);

    let status = assura_json(
        &project,
        &["daemon", "status", project.path_str(), "--format", "json"],
    );

    assert_eq!(status["process"]["state"], "crashed");
    assert_eq!(status["process"]["running"], false);
    assert!(status["process"]["message"]
        .as_str()
        .unwrap()
        .contains("not reachable"));

    let doctor = assura_json(
        &project,
        &["daemon", "doctor", project.path_str(), "--format", "json"],
    );
    assert!(doctor["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["id"] == "managed_process" && check["status"] == "error" }));
}

#[test]
fn daemon_references_source_json_matches_content_references() {
    let project = daemon_project();

    let daemon = assura_json(
        &project,
        &[
            "daemon",
            "references",
            project.path_str(),
            "--source",
            "docs/note.md",
            "--format",
            "json",
        ],
    );
    let content = assura_json(
        &project,
        &[
            "content",
            "references",
            project.path_str(),
            "--source",
            "docs/note.md",
            "--format",
            "json",
        ],
    );

    assert_eq!(daemon["mode"], "source");
    assert_eq!(daemon["health"]["state"], "running");
    assert_eq!(target_paths(&daemon), target_paths(&content));
}

#[test]
fn daemon_references_target_json_matches_content_references() {
    let project = daemon_project();

    let daemon = assura_json(
        &project,
        &[
            "daemon",
            "references",
            project.path_str(),
            "--target",
            "docs/guide.md",
            "--format",
            "json",
        ],
    );
    let content = assura_json(
        &project,
        &[
            "content",
            "references",
            project.path_str(),
            "--target",
            "docs/guide.md",
            "--format",
            "json",
        ],
    );

    assert_eq!(daemon["mode"], "target");
    assert_eq!(daemon["health"]["state"], "running");
    assert_eq!(source_paths(&daemon), source_paths(&content));
}

#[test]
fn daemon_references_moved_target_reports_move_context() {
    let project = daemon_project();

    let json = assura_json(
        &project,
        &[
            "daemon",
            "references",
            project.path_str(),
            "--moved-target",
            "docs/guide.md",
            "--new-target",
            "docs/guide-renamed.md",
            "--format",
            "json",
        ],
    );

    assert_eq!(json["previous_path"], "docs/guide.md");
    assert_eq!(json["new_path"], "docs/guide-renamed.md");
    assert_eq!(json["health"]["state"], "running");
    assert_eq!(json["bounds"]["returned"], 1);
    assert_eq!(json["references"][0]["source_path"], "docs/note.md");
}

#[test]
fn daemon_references_requires_one_direction() {
    let project = daemon_project();

    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "references",
            project.path_str(),
            "--source",
            "docs/note.md",
            "--target",
            "docs/guide.md",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("requires exactly one of --source, --target, or --moved-target"));
}

#[test]
fn daemon_references_validates_direction_before_project_load() {
    let project = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "references",
            project.path().to_str().unwrap(),
            "--source",
            "docs/note.md",
            "--target",
            "docs/guide.md",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("requires exactly one of --source, --target, or --moved-target"));
}

#[test]
fn daemon_health_json_reports_unavailable_when_project_cannot_load() {
    let project = TempDir::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args([
            "daemon",
            "health",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.daemon.error.v1");
    assert_eq!(json["health"]["state"], "unavailable");
    assert!(json["health"]["reason"]
        .as_str()
        .unwrap()
        .contains("no .assura/config.yml found"));
}

fn assura_json(project: &DaemonProject, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "project: {}\nstdout:\n{}\nstderr:\n{}",
        project.path().display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn target_paths(value: &Value) -> Vec<String> {
    value["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|reference| reference["target_path"].as_str().unwrap().to_string())
        .collect()
}

fn source_paths(value: &Value) -> Vec<String> {
    value["references"]
        .as_array()
        .unwrap()
        .iter()
        .map(|reference| reference["source_path"].as_str().unwrap().to_string())
        .collect()
}

fn daemon_project() -> DaemonProject {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  docs/:
    files:
      naming_patterns:
        "*.md": kebab-case
  src/:
    files:
      naming_patterns:
        "*.rs": snake_case
"#,
    )
    .unwrap();
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nSee [guide](guide.md#install) and [code](../src/lib.rs#L1-L2).\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/guide.md"),
        "# Guide\n\n## Install\n",
    )
    .unwrap();
    fs::write(project.path().join("src/lib.rs"), "fn one() {}\n").unwrap();
    DaemonProject { project }
}

struct DaemonProject {
    project: TempDir,
}

impl DaemonProject {
    fn path(&self) -> &Path {
        self.project.path()
    }

    fn path_str(&self) -> &str {
        self.project.path().to_str().unwrap()
    }
}

fn git(project: &DaemonProject, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(project.path())
        .args(args)
        .env("GIT_AUTHOR_NAME", "Assura Test")
        .env("GIT_AUTHOR_EMAIL", "assura@example.test")
        .env("GIT_COMMITTER_NAME", "Assura Test")
        .env("GIT_COMMITTER_EMAIL", "assura@example.test")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn terminate_pid(pid: u32) {
    #[cfg(unix)]
    let output = Command::new("kill").arg(pid.to_string()).output().unwrap();

    #[cfg(windows)]
    let output = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "failed to terminate pid {pid}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::thread::sleep(std::time::Duration::from_millis(200));
}

fn wait_for_pid_to_exit(pid: u32) {
    for _ in 0..30 {
        if !pid_is_running(pid) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("pid {pid} was still running");
}

fn pid_is_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stderr(std::process::Stdio::null())
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
