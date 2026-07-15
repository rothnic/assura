use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

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

fn copy_missing_model_frontmatter_fixture_with_markdown_lint() -> TempDir {
    let source = format!("{FIXTURE_ROOT}/missing_model_frontmatter_field");
    let project = TempDir::new().unwrap();
    let root = project.path();

    for dir in [".assura", "docs/goals", "schemas", "specs"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }

    fs::copy(
        format!("{source}/schemas/content_runtime.schema.json"),
        root.join("schemas/content_runtime.schema.json"),
    )
    .unwrap();
    fs::copy(
        format!("{source}/specs/spec_portable_structure.json"),
        root.join("specs/spec_portable_structure.json"),
    )
    .unwrap();

    let config = fs::read_to_string(format!("{source}/.assura/config.yml"))
        .unwrap()
        .replace("\r\n", "\n");
    let config = config.replace(
        "structure: {}\n",
        "structure:\n  ./:\n    docs/:\n      goals/:\n        markdown:\n          lint_trailing_spaces: true\n",
    );
    fs::write(root.join(".assura/config.yml"), config).unwrap();

    let markdown = fs::read_to_string(format!("{source}/docs/goals/goal_portable_structure.md"))
        .unwrap()
        .replace("\r\n", "\n");
    let markdown = markdown.replace(
        "---\n# Portable Structure Policy",
        "---\n   \n# Portable Structure Policy",
    );
    fs::write(root.join("docs/goals/goal_portable_structure.md"), markdown).unwrap();

    project
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
fn check_reports_assura_root_model_artifact_layout_error() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let config = fs::read_to_string(format!("{FIXTURE_ROOT}/valid/.assura/config.yml"))
        .unwrap()
        .replace(
            "schemas/content_runtime.schema.json",
            "./.assura/content_runtime.schema.json",
        );
    fs::write(project.path().join(".assura/config.yml"), config).unwrap();

    let output = check_path(project.path().to_str().unwrap(), &["--format", "json"]);

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
        .find(|violation| {
            violation["rule"] == "content_runtime:content_model_artifact_outside_models_dir"
        })
        .expect("model artifact layout violation is emitted");

    assert_eq!(
        json_path(&violation["path"]),
        ".assura/content_runtime.schema.json"
    );
    assert!(violation["message"]
        .as_str()
        .unwrap()
        .contains(".assura/models/**"));
}

#[test]
fn markdown_lint_coexists_with_model_owned_frontmatter_validation() {
    let project = copy_missing_model_frontmatter_fixture_with_markdown_lint();
    let output = check_path(project.path().to_str().unwrap(), &["--format", "json"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let violations = report["violations"].as_array().unwrap();

    assert!(violations.iter().any(|violation| {
        violation["rule"] == "content_runtime:invalid_object_shape"
            && json_path(&violation["path"]) == "docs/goals/goal_portable_structure.md"
            && violation["message"]
                .as_str()
                .is_some_and(|message| message.contains("field=title"))
    }));
    assert!(violations.iter().any(|violation| {
        violation["rule"] == "markdown_trailing_spaces"
            && json_path(&violation["path"]) == "docs/goals/goal_portable_structure.md"
    }));
    assert!(!violations
        .iter()
        .any(|violation| violation["rule"] == "markdown_frontmatter_field"));
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
