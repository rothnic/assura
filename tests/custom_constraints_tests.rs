use std::fs;
use std::path::PathBuf;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn permissive_pair_config(source: &str, target: &str, extra: &str) -> String {
    format!(
        r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "{source}"
      target: "{target}"
{extra}
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - target/**
"#
    )
}

#[test]
fn check_custom_paired_file_constraint_reports_missing_target() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &permissive_pair_config("src/*.rs", "tests/{stem}_test.rs", "      severity: high"),
    );
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("tests")).unwrap();
    fs::write(project.path().join("src/parser.rs"), "pub fn parse() {}\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("src/parser.rs"));
    assert_eq!(violation.rule, "custom:source_test_pair");
    assert_eq!(violation.severity, "high");
    assert!(violation.message.contains("tests/parser_test.rs"));
}

#[test]
fn check_custom_paired_file_constraint_passes_when_target_exists() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &permissive_pair_config("src/*.rs", "tests/{stem}_test.rs", ""),
    );
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("tests")).unwrap();
    fs::write(project.path().join("src/parser.rs"), "pub fn parse() {}\n").unwrap();
    fs::write(
        project.path().join("tests/parser_test.rs"),
        "#[test]\nfn parser() {}\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_custom_paired_file_constraint_respects_exclusions() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
extensions:
  custom_constraints:
    - id: generated_source_test_pair
      type: paired_file_exists
      source: "generated/*.rs"
      target: "tests/{stem}_test.rs"
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - generated/**
"#,
    );
    fs::create_dir(project.path().join("generated")).unwrap();
    fs::write(
        project.path().join("generated/parser.rs"),
        "pub fn parse() {}\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}
