use std::fs;
use std::path::Path;
use std::process::Command;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn normalized_json_path(value: &serde_json::Value) -> String {
    value.as_str().unwrap().replace('\\', "/")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn test_relationship_config() -> &'static str {
    r#"
extensions:
  test_relationships:
    - id: supported_tests
      severity: high
      relationships:
        - source: "src/cli/check/*.rs"
          required_tests:
            - "tests/custom_constraints_tests.rs"
      fixture_roots:
        - tests/fixtures
      fixture_families:
        - path: tests/fixtures/test-relationship
          owner: validation-tests
          purpose: reusable test relationship rule coverage
      allowed_ignore_reasons:
        - manual_performance_audit
      ignored_tests:
        - path: tests/manual_performance.rs
          test: manual_performance_audit
          reason: manual_performance_audit
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - target/**
"#
}

fn write_test_relationship_files(project: &TempDir) {
    fs::create_dir_all(project.path().join("src/cli/check")).unwrap();
    fs::create_dir_all(project.path().join("tests/fixtures/test-relationship")).unwrap();
    fs::write(
        project.path().join("src/cli/check/test_relationship.rs"),
        "pub fn validate() {}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("tests/custom_constraints_tests.rs"),
        "#[test]\nfn test_relationship_rule() {}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("tests/manual_performance.rs"),
        concat!("#[test]\n#[", "ignore]\nfn manual_performance_audit() {}\n"),
    )
    .unwrap();
}

#[test]
fn check_test_relationship_passes_when_evidence_and_fixtures_match() {
    let project = TempDir::new().unwrap();
    write_config(&project, test_relationship_config());
    write_test_relationship_files(&project);

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());

    let scoped_report = run_structure_check(
        Some(project.path().join("src/cli/check").to_path_buf()),
        None,
        false,
    )
    .unwrap();
    assert!(scoped_report.success, "{:#?}", scoped_report.violations);
}

#[test]
fn check_test_relationship_reports_missing_tests_ignored_tests_and_fixtures() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
extensions:
  test_relationships:
    - id: supported_tests
      severity: high
      relationships:
        - source: "src/cli/check/*.rs"
          required_tests:
            - "tests/missing_tests.rs"
      fixture_roots:
        - tests/fixtures
      fixture_families:
        - path: tests/fixtures/test-relationship
          owner: validation-tests
          purpose: reusable test relationship rule coverage
      allowed_ignore_reasons:
        - manual_performance_audit
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - target/**
"#,
    );
    write_test_relationship_files(&project);
    fs::write(
        project.path().join("tests/manual_performance.rs"),
        concat!(
            "#[test]\n#[",
            "ignore]\nfn manual_performance_audit() {}\n",
            "#[test]\n#[",
            "ignore]\nfn unclassified_same_file_audit() {}\n"
        ),
    )
    .unwrap();
    fs::create_dir_all(project.path().join("tests/fixtures/unowned")).unwrap();
    fs::write(
        project.path().join("tests/manual_unclassified.rs"),
        concat!("#[test]\n#[", "ignore]\nfn manual_unclassified() {}\n"),
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("src/cli/check/test_relationship.rs")
                && violation.rule == "test_relationship:supported_tests"
                && violation.severity == "high"
                && violation.message.contains("tests/missing_tests.rs")
        }),
        "{:#?}",
        report.violations
    );
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("tests/manual_performance.rs")
                && violation.rule == "test_relationship:supported_tests"
                && violation.message.contains("unclassified_same_file_audit")
        }),
        "{:#?}",
        report.violations
    );
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("tests/manual_unclassified.rs")
                && violation.rule == "test_relationship:supported_tests"
                && violation.message.contains("accepted reason category")
        }),
        "{:#?}",
        report.violations
    );
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("tests/fixtures/unowned")
                && violation.rule == "test_relationship:supported_tests"
                && violation.message.contains("owner and purpose")
        }),
        "{:#?}",
        report.violations
    );
}

#[test]
fn check_test_relationship_cli_json_reports_actionable_rule_context() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
extensions:
  test_relationships:
    - id: supported_tests
      severity: high
      relationships:
        - source: "src/cli/check/*.rs"
          required_tests:
            - "tests/missing_tests.rs"
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - target/**
"#,
    );
    fs::create_dir_all(project.path().join("src/cli/check")).unwrap();
    fs::write(
        project.path().join("src/cli/check/test_relationship.rs"),
        "pub fn validate() {}\n",
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let violations = report["violations"].as_array().unwrap();
    assert!(
        violations.iter().any(|violation| {
            normalized_json_path(&violation["path"]) == "src/cli/check/test_relationship.rs"
                && violation["rule"] == "test_relationship:supported_tests"
                && violation["message"]
                    .as_str()
                    .unwrap()
                    .contains("tests/missing_tests.rs")
                && violation["corrective_context"]
                    .as_str()
                    .unwrap()
                    .contains("test evidence")
        }),
        "{report:#?}"
    );
}
