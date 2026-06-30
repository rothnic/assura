use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

#[test]
fn check_warn_reports_violations_but_exits_successfully() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--warn")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file_naming"), "stdout was:\n{}", stdout);
}

#[test]
fn low_severity_structure_findings_are_advisory_but_visible() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
      severity: low
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true);
    let violation = &report["violations"][0];
    assert_eq!(violation["rule"], "file_naming");
    assert_eq!(violation["severity"], "low");
    assert_eq!(violation["severity_label"], "Low");
    assert_eq!(violation["blocking"], false);
    assert!(
        violation["corrective_context"]
            .as_str()
            .is_some_and(|context| !context.is_empty()),
        "violation should include corrective context: {violation:#}"
    );
}

#[test]
fn blocking_structure_findings_still_fail_without_warn() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
      severity: medium
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], false);
    assert_eq!(report["violations"][0]["severity"], "medium");
    assert_eq!(report["violations"][0]["severity_label"], "Medium");
    assert_eq!(report["violations"][0]["blocking"], true);
}

#[test]
fn agent_feedback_keeps_advisory_findings_structured() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
      severity: low
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.agent-feedback.v1");
    assert_eq!(json["blocking"], false);
    assert_eq!(json["feedback"][0]["status"], "advisory");
    let message = &json["feedback"][0]["messages"][0];
    assert_eq!(message["rule"], "file_naming");
    assert_eq!(message["severity"], "low");
    assert_eq!(message["severity_label"], "Low");
    assert_eq!(message["blocking"], false);
    assert!(
        message["corrective_context"]
            .as_str()
            .is_some_and(|context| !context.is_empty()),
        "message should include corrective context: {message:#}"
    );
}

#[test]
fn text_and_yaml_reports_include_blocking_contract_fields() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
      severity: low
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let text = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("text")
        .output()
        .unwrap();
    assert!(text.status.success());
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("Blocking: false"), "stdout was:\n{stdout}");

    let yaml = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("yaml")
        .output()
        .unwrap();
    assert!(yaml.status.success());
    let report: serde_yaml::Value = serde_yaml::from_slice(&yaml.stdout).unwrap();
    assert_eq!(report["success"], serde_yaml::Value::Bool(true));
    assert_eq!(
        report["violations"][0]["severity_label"],
        serde_yaml::Value::String("Low".to_string())
    );
    assert_eq!(
        report["violations"][0]["blocking"],
        serde_yaml::Value::Bool(false)
    );
}

#[test]
fn fail_fast_continues_after_advisory_findings_to_later_blocking_checks() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
extensions:
  custom_constraints:
    - id: docs_pair
      type: paired_file_exists
      source: "docs/*.md"
      target: "docs/{stem}_test.md"
      severity: high
structure:
  ./:
    files:
      naming: kebab-case
      severity: low
      allow_extra: true
    directories:
      allow_extra: true
    children:
      .assura/:
        files:
          naming: kebab-case
      docs/:
        files:
          allow_extra: true
"#,
    );
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();
    fs::write(project.path().join("docs/guide.md"), "# Guide\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--fail-fast")
        .arg("--format")
        .arg("agent")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["blocking"], true);
    assert_eq!(json["feedback"][0]["status"], "fail");
    let messages = json["feedback"][0]["messages"].as_array().unwrap();
    assert!(messages.iter().any(|message| {
        message["rule"] == "file_naming"
            && message["severity"] == "low"
            && message["blocking"] == false
    }));
    assert!(messages.iter().any(|message| {
        message["rule"] == "custom:docs_pair"
            && message["severity"] == "high"
            && message["blocking"] == true
    }));
}
