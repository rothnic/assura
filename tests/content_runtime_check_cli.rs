use serde_json::Value;
use std::process::{Command, Output};

const FIXTURE_ROOT: &str = "tests/fixtures/content_runtime";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn check_fixture(fixture: &str, args: &[&str]) -> Output {
    let mut command = Command::new(assura_bin());
    command
        .arg("check")
        .arg(format!("{FIXTURE_ROOT}/{fixture}"));
    for arg in args {
        command.arg(arg);
    }
    command.output().unwrap()
}

fn check_path(path: &str, args: &[&str]) -> Output {
    let mut command = Command::new(assura_bin());
    command.arg("check").arg(path);
    for arg in args {
        command.arg(arg);
    }
    command.output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn json_path(value: &Value) -> String {
    value.as_str().unwrap().replace('\\', "/")
}

#[test]
fn check_json_reports_content_reference_diagnostics() {
    let output = check_fixture("missing_reference", &["--format", "json"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], false);
    let violation = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|violation| violation["rule"] == "content_runtime:missing_reference")
        .expect("content missing-reference violation is emitted");

    assert_eq!(
        json_path(&violation["path"]),
        "docs/goals/goal_portable_structure.md"
    );
    assert_eq!(violation["severity"], "high");
    let message = violation["message"].as_str().unwrap();
    assert!(
        message.contains("goals:goal-portable-structure"),
        "{message}"
    );
    assert!(message.contains("object_type=Goal"), "{message}");
    assert!(message.contains("field=specs"), "{message}");
    assert!(
        message.contains("referenced_object=specs:missing-spec"),
        "{message}"
    );
}

#[test]
fn check_agent_codex_reports_content_diagnostic_context() {
    let output = check_fixture(
        "missing_reference",
        &["--format", "agent", "--agent", "codex"],
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["hookSpecificOutput"]["hookEventName"],
        "UserPromptSubmit"
    );
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        context.contains("content_runtime:missing_reference"),
        "{context}"
    );
    assert!(context.contains("object_type=Goal"), "{context}");
    assert!(context.contains("field=specs"), "{context}");
    assert!(
        context.contains("referenced_object=specs:missing-spec"),
        "{context}"
    );
}

#[test]
fn check_text_and_yaml_include_content_diagnostics() {
    for format in ["text", "yaml"] {
        let output = check_fixture("missing_reference", &["--format", format]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "format: {format}\nstdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
        let rendered = stdout(&output);
        assert!(
            rendered.contains("content_runtime:missing_reference"),
            "{rendered}"
        );
        assert!(rendered.contains("object_type=Goal"), "{rendered}");
        assert!(rendered.contains("field=specs"), "{rendered}");
        assert!(
            rendered.contains("referenced_object=specs:missing-spec"),
            "{rendered}"
        );
    }
}

#[test]
fn check_valid_content_runtime_fixture_passes() {
    let output = check_fixture("valid", &["--format", "json"]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn check_reports_content_model_construction_errors() {
    let output = check_fixture("missing_schema", &["--format", "json"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let violation = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|violation| violation["rule"] == "content_runtime:content_schema_missing")
        .expect("missing schema artifact is emitted as a check violation");

    assert_eq!(json_path(&violation["path"]), ".assura/config.yml");
    assert!(violation["message"]
        .as_str()
        .unwrap()
        .contains("models.validation_artifact"));
}

#[test]
fn check_lslint_target_semantics_stays_structure_only() {
    let output = check_path(
        "tests/fixtures/content_runtime/missing_reference/docs/goals/goal_portable_structure.md",
        &["--format", "json", "--ls-lint-target-semantics"],
    );

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
    assert!(
        !stdout(&output).contains("content_runtime:"),
        "{}",
        stdout(&output)
    );
}
