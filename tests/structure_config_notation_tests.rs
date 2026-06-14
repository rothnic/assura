use std::fs;
use std::path::PathBuf;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn relationship_pair_config() -> &'static str {
    r#"
structure:
  ./:
    extra: true
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
exclude:
  - target/**
"#
}

fn package_doc_config() -> &'static str {
    r#"
structure:
  ./:
    extra: true
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
  docs/:
    required: false
    packages.md:
      sections:
        "{package}":
          provides: doc
exclude:
  - target/**
"#
}

#[test]
fn strict_root_policy_ignores_assura_tool_state_without_user_exclude() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: false
    README.md: exists:1
"#,
    );
    fs::write(project.path().join("README.md"), "# Project\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn captured_counterpart_reports_missing_test_file() {
    let project = TempDir::new().unwrap();
    write_config(&project, relationship_pair_config());
    fs::create_dir_all(project.path().join("src/components")).unwrap();
    fs::write(
        project.path().join("src/components/Button.tsx"),
        "export function Button() { return null; }\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("src/components/Button.tsx"));
    assert_eq!(violation.rule, "relationship:captured-counterpart-1");
    assert!(violation.message.contains("counterpart-1"));
}

#[test]
fn captured_counterpart_passes_when_test_file_exists() {
    let project = TempDir::new().unwrap();
    write_config(&project, relationship_pair_config());
    fs::create_dir_all(project.path().join("src/components")).unwrap();
    fs::write(
        project.path().join("src/components/Button.tsx"),
        "export function Button() { return null; }\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/components/Button.test.tsx"),
        "test('Button', () => {});\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn package_doc_relationship_passes_with_aggregate_section() {
    let project = TempDir::new().unwrap();
    write_config(&project, package_doc_config());
    fs::create_dir_all(project.path().join("packages/core")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/packages.md"),
        "# Packages\n\n## core\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn package_doc_relationship_passes_with_dedicated_doc_file() {
    let project = TempDir::new().unwrap();
    write_config(&project, package_doc_config());
    fs::create_dir_all(project.path().join("packages/core")).unwrap();
    fs::create_dir_all(project.path().join("docs/packages")).unwrap();
    fs::write(
        project.path().join("docs/packages/core.md"),
        "# Core package\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn package_doc_relationship_reports_missing_doc_provider() {
    let project = TempDir::new().unwrap();
    write_config(&project, package_doc_config());
    fs::create_dir_all(project.path().join("packages/core")).unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("packages/core"));
    assert_eq!(violation.rule, "relationship:captured-doc-1");
    assert!(violation.message.contains("doc"));
}
