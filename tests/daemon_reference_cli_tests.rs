use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn daemon_references_source_json_matches_content_references() {
    let project = daemon_project();

    let daemon = daemon_references_json(&project, &["--source", "docs/note.md"]);
    let content = content_references_json(&project, &["--source", "docs/note.md"]);

    assert_eq!(daemon["mode"], "source");
    assert_eq!(daemon["health"]["state"], "running");
    assert_eq!(target_paths(&daemon), target_paths(&content));
}

#[test]
fn daemon_references_target_json_matches_content_references() {
    let project = daemon_project();

    let daemon = daemon_references_json(&project, &["--target", "docs/guide.md"]);
    let content = content_references_json(&project, &["--target", "docs/guide.md"]);

    assert_eq!(daemon["mode"], "target");
    assert_eq!(daemon["health"]["state"], "running");
    assert_eq!(source_paths(&daemon), source_paths(&content));
}

#[test]
#[cfg_attr(
    any(windows, tarpaulin),
    ignore = "managed daemon subprocess lifecycle is covered by normal Unix CI"
)]
fn daemon_references_json_uses_running_ipc_process_for_source_and_target() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);

    let source = daemon_references_json(&project, &["--source", "docs/note.md"]);
    let content_source = content_references_json(&project, &["--source", "docs/note.md"]);
    assert_eq!(source["schema"], "assura.daemon.references.v1");
    assert_eq!(source["protocol_version"], "assura.daemon.v1");
    assert_eq!(source["mode"], "source");
    assert_eq!(source["health"]["state"], "running");
    assert_eq!(target_paths(&source), target_paths(&content_source));

    let target = daemon_references_json(&project, &["--target", "docs/guide.md"]);
    let content_target = content_references_json(&project, &["--target", "docs/guide.md"]);
    assert_eq!(target["schema"], "assura.daemon.references.v1");
    assert_eq!(target["protocol_version"], "assura.daemon.v1");
    assert_eq!(target["mode"], "target");
    assert_eq!(target["health"]["state"], "running");
    assert_eq!(source_paths(&target), source_paths(&content_target));

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
#[cfg_attr(
    any(windows, tarpaulin),
    ignore = "managed daemon subprocess lifecycle is covered by normal Unix CI"
)]
fn daemon_references_target_ipc_refreshes_when_source_changes() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);

    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nNo guide link.\n",
    )
    .unwrap();

    let target = daemon_references_json(&project, &["--target", "docs/guide.md"]);
    let content_target = content_references_json(&project, &["--target", "docs/guide.md"]);
    assert_eq!(target["schema"], "assura.daemon.references.v1");
    assert_eq!(target["protocol_version"], "assura.daemon.v1");
    assert_eq!(target["health"]["state"], "running");
    assert_eq!(source_paths(&target), source_paths(&content_target));
    assert!(source_paths(&target).is_empty());

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
#[cfg_attr(
    any(windows, tarpaulin),
    ignore = "managed daemon subprocess lifecycle is covered by normal Unix CI"
)]
fn daemon_references_moved_target_json_uses_running_ipc_process() {
    let project = daemon_project();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);

    fs::rename(
        project.path().join("docs/guide.md"),
        project.path().join("docs/guide-renamed.md"),
    )
    .unwrap();

    let json = daemon_references_json(
        &project,
        &[
            "--moved-target",
            "docs/guide.md",
            "--new-target",
            "docs/guide-renamed.md",
        ],
    );

    assert_eq!(json["schema"], "assura.daemon.references.v1");
    assert_eq!(json["protocol_version"], "assura.daemon.v1");
    assert_eq!(json["previous_path"], "docs/guide.md");
    assert_eq!(json["new_path"], "docs/guide-renamed.md");
    assert_eq!(json["health"]["state"], "degraded");
    assert_eq!(json["references"][0]["source_path"], "docs/note.md");

    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
#[cfg_attr(
    any(windows, tarpaulin),
    ignore = "managed daemon subprocess lifecycle is covered by normal Unix CI"
)]
fn daemon_references_json_reports_stale_config_from_running_ipc_process() {
    let project = daemon_project();
    let config_path = project.path().join(".assura/config.yml");
    let original_config = fs::read_to_string(&config_path).unwrap();

    let start = assura_json(
        &project,
        &["daemon", "start", project.path_str(), "--format", "json"],
    );
    assert_eq!(start["runtime"]["running"], true);

    fs::write(
        &config_path,
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
            "references",
            project.path_str(),
            "--source",
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

    fs::write(&config_path, original_config).unwrap();
    let stop = assura_json(
        &project,
        &["daemon", "stop", project.path_str(), "--format", "json"],
    );
    assert_eq!(stop["runtime"]["state"], "stopped");
}

#[test]
fn daemon_references_moved_target_reports_move_context() {
    let project = daemon_project();

    let json = daemon_references_json(
        &project,
        &[
            "--moved-target",
            "docs/guide.md",
            "--new-target",
            "docs/guide-renamed.md",
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

fn daemon_references_json(project: &DaemonProject, selectors: &[&str]) -> Value {
    references_json(project, "daemon", selectors)
}

fn content_references_json(project: &DaemonProject, selectors: &[&str]) -> Value {
    references_json(project, "content", selectors)
}

fn references_json(project: &DaemonProject, command: &str, selectors: &[&str]) -> Value {
    let mut args = vec![command, "references", project.path_str()];
    args.extend_from_slice(selectors);
    args.extend_from_slice(&["--format", "json"]);
    assura_json(project, &args)
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
