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

fn first_time_rust_project_config() -> &'static str {
    r#"
structure:
  ./:
    extra: false
    README.md: exists:1
    Cargo.toml: exists:1
    src/: exists:1
    "*.lock": exists:0-1
  src/:
    .rs: snake_case
exclude:
  - target/**
"#
}

fn first_time_package_project_config() -> &'static str {
    r#"
rules:
  "@package-standard":
    README.md: exists:1
    package.json: exists:1
    src/: exists:1
    .ts: kebab-case
structure:
  ./:
    extra: true
  packages/:
    "{package}/":
      use: "@package-standard"
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
exclude:
  - node_modules/**
  - dist/**
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
fn first_time_rust_project_config_accepts_minimal_useful_shape() {
    let project = TempDir::new().unwrap();
    write_config(&project, first_time_rust_project_config());
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(project.path().join("src/main.rs"), "fn main() {}\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn first_time_rust_project_config_reports_actionable_drift() {
    let project = TempDir::new().unwrap();
    write_config(&project, first_time_rust_project_config());
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"example\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::write(project.path().join("src/BadName.rs"), "fn main() {}\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("src/BadName.rs"));
    assert_eq!(violation.rule, "file_naming");
    assert!(
        violation.corrective_context.contains("Rename"),
        "{violation:#?}"
    );
}

#[test]
fn first_time_package_project_config_accepts_reusable_rules_and_docs() {
    let project = TempDir::new().unwrap();
    write_config(&project, first_time_package_project_config());
    fs::create_dir_all(project.path().join("packages/core/src")).unwrap();
    fs::create_dir_all(project.path().join("docs/packages")).unwrap();
    fs::write(project.path().join("packages/core/README.md"), "# Core\n").unwrap();
    fs::write(
        project.path().join("packages/core/package.json"),
        "{\"name\":\"@example/core\"}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("packages/core/src/index.ts"),
        "export const core = true;\n",
    )
    .unwrap();
    fs::write(project.path().join("docs/packages/core.md"), "# Core\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn first_time_package_project_config_reports_missing_doc_provider() {
    let project = TempDir::new().unwrap();
    write_config(&project, first_time_package_project_config());
    fs::create_dir_all(project.path().join("packages/core/src")).unwrap();
    fs::write(project.path().join("packages/core/README.md"), "# Core\n").unwrap();
    fs::write(
        project.path().join("packages/core/package.json"),
        "{\"name\":\"@example/core\"}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("packages/core/src/index.ts"),
        "export const core = true;\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("packages/core"));
    assert!(
        violation.rule.starts_with("relationship:captured-doc-"),
        "{violation:#?}"
    );
    assert!(violation.message.contains("provider kind 'doc'"));
    assert!(violation.message.contains("packages/{package}"));
    assert!(violation.message.contains("docs/packages/{package}.md"));
}

#[test]
fn first_time_configs_do_not_accept_removed_alpha_capture_forms() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  src/:
    "${module}.rs": {}
"#,
    );
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/main.rs"), "fn main() {}\n").unwrap();

    let error = run_structure_check(Some(project.path().to_path_buf()), None, false)
        .expect_err("removed alpha capture syntax should be rejected");

    assert!(
        error
            .to_string()
            .contains("captures use single braces like {name}"),
        "{error:#}"
    );
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
    assert!(violation.message.contains("missing counterpart"));
    assert!(violation.message.contains("src/components/{component}.tsx"));
    assert!(violation.message.contains("src/components/Button.test.tsx"));
    assert!(violation
        .message
        .contains("src/components/{component}.test.tsx"));
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
fn package_doc_relationship_passes_when_file_and_section_providers_overlap() {
    let project = TempDir::new().unwrap();
    write_config(&project, package_doc_config());
    fs::create_dir_all(project.path().join("packages/core")).unwrap();
    fs::create_dir_all(project.path().join("docs/packages")).unwrap();
    fs::write(
        project.path().join("docs/packages/core.md"),
        "# Core package\n",
    )
    .unwrap();
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
    assert!(violation.message.contains("provider kind 'doc'"));
    assert!(violation.message.contains("packages/{package}"));
    assert!(violation.message.contains("docs/packages/core.md"));
    assert!(violation.message.contains("docs/packages.md#core"));
    assert!(violation.message.contains("docs/packages/{package}.md"));
    assert!(violation.message.contains("docs/packages.md"));
}

#[test]
fn same_name_captures_in_separate_scopes_do_not_cross_require_counterparts() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
  src/components/:
    "{name}.tsx": {}
    "{name}.test.tsx": exists:1
  src/hooks/:
    "{name}.ts": {}
    "{name}.test.ts": exists:1
exclude:
  - target/**
"#,
    );
    fs::create_dir_all(project.path().join("src/components")).unwrap();
    fs::create_dir_all(project.path().join("src/hooks")).unwrap();
    fs::write(project.path().join("src/components/Button.tsx"), "").unwrap();
    fs::write(project.path().join("src/components/Button.test.tsx"), "").unwrap();
    fs::write(project.path().join("src/hooks/use-data.ts"), "").unwrap();
    fs::write(project.path().join("src/hooks/use-data.test.ts"), "").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}
