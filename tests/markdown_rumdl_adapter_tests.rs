use std::fs;
use std::process::{Command, Output};

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn yaml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn write_project(config_markdown: &str, note: &str) -> TempDir {
    let project = TempDir::new().expect("temp project");
    fs::create_dir_all(project.path().join(".assura")).expect("create .assura");
    fs::create_dir_all(project.path().join("docs")).expect("create docs");
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    extra: true
    children:
      docs/:
        markdown:
{config_markdown}
exclude:
  - target/**
"#
        ),
    )
    .expect("write config");
    fs::write(project.path().join("docs/note.md"), note).expect("write note");
    project
}

fn run_check(project: &TempDir) -> Output {
    Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .expect("assura check runs")
}

fn check_json(project: &TempDir) -> (std::process::ExitStatus, serde_json::Value) {
    let output = run_check(project);
    let json = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse stdout as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, json)
}

#[cfg(unix)]
fn write_fake_rumdl(project: &TempDir, script: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let fake_rumdl = project.path().join("fake-rumdl");
    fs::write(&fake_rumdl, script).expect("write fake rumdl");
    let mut permissions = fs::metadata(&fake_rumdl)
        .expect("fake rumdl metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_rumdl, permissions).expect("chmod fake rumdl");
    fake_rumdl
}

#[test]
fn markdownlint_candidate_is_opt_in_even_when_configured() {
    let project = write_project(
        "          markdownlint_candidate:\n            enabled: false\n            engine: rumdl\n            binary: missing-rumdl-binary-for-assura-test\n",
        "# Note\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["success"], true);
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdownlint_candidate_missing_binary_reports_markdown_engine_without_mutating_source() {
    let project = write_project(
        "          markdownlint_candidate:\n            enabled: true\n            engine: rumdl\n            binary: missing-rumdl-binary-for-assura-test\n",
        "# Note\n",
    );
    let note = project.path().join("docs/note.md");
    let before = fs::read_to_string(&note).expect("read note before check");

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    assert_eq!(
        fs::read_to_string(&note).expect("read note after check"),
        before,
        "candidate failures must not mutate source Markdown"
    );
    let violation = &json["violations"][0];
    assert_eq!(violation["rule"], "markdown_engine");
    assert!(violation["message"]
        .as_str()
        .unwrap()
        .contains("failed to spawn"));
}

#[test]
fn markdownlint_candidate_rejects_unsupported_engine() {
    let project = write_project(
        "          markdownlint_candidate:\n            enabled: true\n            engine: other\n",
        "# Note\n",
    );

    let output = run_check(&project);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("expected supported candidate engine 'rumdl'"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn markdownlint_candidate_rejects_empty_binary() {
    let project = write_project(
        "          markdownlint_candidate:\n            enabled: true\n            engine: rumdl\n            binary: \"\"\n",
        "# Note\n",
    );

    let output = run_check(&project);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("candidate binary cannot be empty"),
        "stderr:\n{stderr}"
    );
}

#[cfg(unix)]
#[test]
fn markdownlint_candidate_runs_against_isolated_copy_and_maps_diagnostics() {
    let project = TempDir::new().expect("temp project");
    let fake_rumdl = write_fake_rumdl(
        &project,
        r#"#!/bin/sh
printf 'candidate touched isolated copy\n' > "$5"
printf '[{"line":1,"rule":"MD009","message":"Trailing spaces"}]'
exit 1
"#,
    );

    fs::create_dir_all(project.path().join(".assura")).expect("create .assura");
    fs::create_dir_all(project.path().join("docs")).expect("create docs");
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    extra: true
    children:
      docs/:
        markdown:
          markdownlint_candidate:
            enabled: true
            engine: rumdl
            binary: {}
exclude:
  - target/**
"#,
            yaml_string(&fake_rumdl.to_string_lossy())
        ),
    )
    .expect("write config");
    let note = project.path().join("docs/note.md");
    fs::write(&note, "# Note  \n").expect("write note");
    let before = fs::read_to_string(&note).expect("read note before check");

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(
        fs::read_to_string(&note).expect("read note after check"),
        before,
        "candidate process must receive an isolated copy, not the source file"
    );
    let violation = json["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|violation| violation["rule"] == "markdown_trailing_spaces")
        .unwrap_or_else(|| panic!("missing mapped rumdl diagnostic: {json:#}"));
    assert_eq!(violation["severity"], "low");
    assert_eq!(violation["blocking"], false);
}

#[cfg(unix)]
#[test]
fn markdownlint_candidate_does_not_duplicate_native_owned_rules_or_consume_suppressions() {
    let project = TempDir::new().expect("temp project");
    let fake_rumdl = write_fake_rumdl(
        &project,
        r#"#!/bin/sh
printf '[{"line":4,"rule":"MD009","message":"Trailing spaces"}]'
exit 1
"#,
    );

    fs::create_dir_all(project.path().join(".assura")).expect("create .assura");
    fs::create_dir_all(project.path().join("docs")).expect("create docs");
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    extra: true
    children:
      docs/:
        markdown:
          lint_trailing_spaces: true
          markdownlint_candidate:
            enabled: true
            engine: rumdl
            binary: {}
exclude:
  - target/**
"#,
            yaml_string(&fake_rumdl.to_string_lossy())
        ),
    )
    .expect("write config");
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\n<!-- assura-ignore markdown_trailing_spaces: fixture proves native check owns MD009 -->\n  \n",
    )
    .expect("write note");

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["success"], true);
    assert_eq!(
        json["violations"].as_array().unwrap().len(),
        0,
        "candidate MD009 must not re-report after native suppression handling"
    );
}
