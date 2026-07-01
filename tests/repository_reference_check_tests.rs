use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_project(with_policy: bool) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();

    let repository_references = if with_policy {
        r#"
extensions:
  repository_references:
    - id: source_refs
      paths:
        - "src/**"
      severity: high
"#
    } else {
        ""
    };
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    extra: true
{repository_references}
exclude:
  - target/**
"#
        ),
    )
    .unwrap();
    fs::write(
        project.path().join("docs/guide.md"),
        "# Guide\n\n## Good Section\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/target.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        r#"// See docs/missing.md before moving docs.
// See docs/guide.md#missing-section before editing headings.
// See docs/guide.md#good-section for the current heading.
// See src/target.rs#L9 before deleting target lines.
// See src/target.rs#L1-L2 for valid target lines.
"#,
    )
    .unwrap();
    project
}

fn check_json(project: &TempDir) -> (Output, Value) {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    let json = serde_json::from_slice(&output.stdout).unwrap();
    (output, json)
}

#[test]
fn repository_reference_check_reports_source_comment_breakage() {
    let project = write_project(true);

    let (output, json) = check_json(&project);

    assert_eq!(output.status.code(), Some(1), "report: {json:#}");
    let violations = json["violations"].as_array().unwrap();
    let rules = violations
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        rules.contains(&"repository_reference_target"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"repository_reference_anchor"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"repository_reference_line_anchor"),
        "rules: {rules:?}"
    );
    assert_eq!(violations.len(), 3, "report: {json:#}");
    assert!(violations
        .iter()
        .all(|violation| { violation["path"] == "src/lib.rs" && violation["severity"] == "high" }));
    assert!(violations.iter().any(|violation| {
        violation["message"]
            .as_str()
            .is_some_and(|message| message.contains("comment_reference; confidence=medium"))
    }));
}

#[test]
fn repository_reference_check_is_opt_in() {
    let project = write_project(false);

    let (output, json) = check_json(&project);

    assert!(output.status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}
