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
    assert_eq!(json["process"]["running"], false);
    assert_eq!(json["process"]["mode"], "local_probe");
    assert!(json["management"]["doctor"]
        .as_str()
        .unwrap()
        .contains("assura daemon doctor --format json"));
    assert_eq!(json["management"]["start"], Value::Null);
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
