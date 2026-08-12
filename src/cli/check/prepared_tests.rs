//! Tests for prepared structure-check plans.

use super::*;
use std::fs;

fn write_project(root: &std::path::Path, naming: &str, file_name: &str) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        format!(
            r#"
structure:
  src/:
    files:
      naming_patterns:
        "*.ts": {naming}
"#
        ),
    )
    .unwrap();
    fs::write(root.join("src").join(file_name), "").unwrap();
}

fn write_count_project(root: &std::path::Path, file_names: &[&str]) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
structure:
  src/:
    files:
      exists:
        "*.ts": "2"
"#,
    )
    .unwrap();
    for file_name in file_names {
        fs::write(root.join("src").join(file_name), "").unwrap();
    }
}

#[test]
fn prepared_check_validates_changed_file_without_tree_walk() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "kebab-case", "bad_name.ts");
    fs::write(temp.path().join("src").join("good-file.ts"), "").unwrap();

    let prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();

    let report = prepared
        .check_changed_path(temp.path().join("src").join("bad_name.ts"))
        .unwrap();
    assert!(!report.success);
    assert_eq!(report.files_checked, 1);
    assert_eq!(report.dirs_checked, 0);
    assert_eq!(report.violation_count(), 1);

    let report = prepared
        .check_changed_path(temp.path().join("src").join("good-file.ts"))
        .unwrap();
    assert!(report.success);
    assert_eq!(report.files_checked, 1);
    assert_eq!(report.dirs_checked, 0);
}

#[test]
fn prepared_check_rechecks_parent_counts_for_changed_file() {
    let temp = tempfile::tempdir().unwrap();
    write_count_project(temp.path(), &["one.ts"]);

    let prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();

    let report = prepared
        .check_changed_path(temp.path().join("src").join("one.ts"))
        .unwrap();
    assert!(!report.success);
    assert_eq!(report.files_checked, 1);
    assert_eq!(report.dirs_checked, 0);
    assert!(report
        .violations
        .iter()
        .any(|violation| violation.rule == "exists_count"));

    fs::write(temp.path().join("src").join("two.ts"), "").unwrap();
    let report = prepared
        .check_changed_path(temp.path().join("src").join("two.ts"))
        .unwrap();
    assert!(report.success);
    assert_eq!(report.files_checked, 1);
    assert_eq!(report.dirs_checked, 0);
}

#[test]
fn prepared_check_rechecks_parent_counts_for_deleted_file() {
    let temp = tempfile::tempdir().unwrap();
    write_count_project(temp.path(), &["one.ts", "two.ts"]);

    let prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();

    let deleted = temp.path().join("src").join("two.ts");
    fs::remove_file(&deleted).unwrap();
    let report = prepared.check_changed_path(deleted).unwrap();
    assert!(!report.success);
    assert_eq!(report.files_checked, 0);
    assert_eq!(report.dirs_checked, 0);
    assert!(report
        .violations
        .iter()
        .any(|violation| violation.rule == "exists_count"));
}

#[test]
fn prepared_check_reloads_when_config_changes() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "kebab-case", "bad_name.ts");

    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();
    let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
    assert!(!report.success);

    write_project(temp.path(), "snake_case", "bad_name.ts");
    assert!(prepared.reload_if_config_changed().unwrap());
    let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
    assert!(report.success);
}

#[test]
fn prepared_check_keeps_plan_when_config_is_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "kebab-case", "good-file.ts");

    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();

    assert!(!prepared.reload_if_config_changed().unwrap());
    let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
    assert!(report.success);
}

#[test]
fn prepared_check_keeps_plan_for_same_content_rewrite() {
    let temp = tempfile::tempdir().unwrap();
    write_project(temp.path(), "kebab-case", "good-file.ts");

    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();
    let config_path = temp.path().join(".assura/config.yml");
    let content = fs::read_to_string(&config_path).unwrap();
    fs::write(&config_path, content).unwrap();

    assert!(!prepared.reload_if_config_changed().unwrap());
    let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
    assert!(report.success);
}

#[test]
fn prepared_changed_path_uses_full_project_for_cross_path_policy() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join(".assura")).unwrap();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join("tests")).unwrap();
    fs::write(
        temp.path().join(".assura/config.yml"),
        r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "src/*.ts"
      target: "tests/{stem}_test.rs"
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
"#,
    )
    .unwrap();
    let source = temp.path().join("src/new-source.ts");
    fs::write(&source, "export {};\n").unwrap();

    let prepared =
        PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
            .unwrap();
    assert!(!prepared.supports_incremental_path_checks());

    let report = prepared.check_changed_path(source).unwrap();
    assert_eq!(report.checked_path, temp.path().canonicalize().unwrap());
    assert!(!report.success);
    assert!(report
        .violations
        .iter()
        .any(|violation| violation.rule == "custom:source_test_pair"));
}
