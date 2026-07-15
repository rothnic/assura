use std::fs;
use std::path::{Path, PathBuf};

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
    exists: 0-1
    "{package}.md":
      provides: doc
  docs/:
    exists: 0-1
    packages.md:
      exists: 0-1
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
    exists: 0-1
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
fn nested_cardinality_and_scalar_tree_rules_enforce_the_displayed_contract() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
rules:
  "@source-file": { naming: kebab-case, max_lines: 5 }
  "@web-app":
    package.json: exists:1
    src/: exists:1
structure:
  ./:
    extra: false
    .ts: "@source-file"
    .tsx: "@source-file"
    docs/: exists:0-1
    apps/:
      .dir: kebab-case
      web/: "@web-app"
exclude:
  - "**/{generated,vendor,dist}/**"
"#,
    );
    fs::create_dir_all(project.path().join("apps/web/src")).unwrap();
    fs::write(project.path().join("apps/web/package.json"), "{}\n").unwrap();
    fs::write(
        project.path().join("apps/web/src/user-menu.tsx"),
        "export const userMenu = true;\n",
    )
    .unwrap();

    let passing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(passing.success, "{:#?}", passing.violations);

    fs::write(
        project.path().join("apps/web/src/BadName.tsx"),
        "export const badName = true;\n",
    )
    .unwrap();
    fs::write(
        project.path().join("apps/web/src/too-long.ts"),
        "export const value = true;\n".repeat(6),
    )
    .unwrap();
    fs::create_dir_all(project.path().join("tmp-output")).unwrap();

    let failing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(!failing.success);
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("BadName.tsx") && violation.rule == "file_naming"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.ts") && violation.rule == "max_lines"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path == Path::new("tmp-output") && violation.rule == "unexpected_directory"
    }));
}

#[test]
fn optional_nested_directory_skips_child_requirements_when_absent() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    docs/:
      exists: 0-1
      README.md: exists:1
"#,
    );

    let absent = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(absent.success, "{:#?}", absent.violations);

    fs::create_dir_all(project.path().join("docs")).unwrap();
    let present = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(!present.success);
    assert!(present
        .violations
        .iter()
        .any(|violation| violation.path == Path::new("docs") && violation.rule == "exists_count"));
}

#[test]
fn captured_directory_counts_match_capture_names() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    packages/:
      "{package}/":
        exists: 2
        package.json: exists:1
"#,
    );
    for package in ["core", "ui-kit"] {
        fs::create_dir_all(project.path().join("packages").join(package)).unwrap();
        fs::write(
            project
                .path()
                .join("packages")
                .join(package)
                .join("package.json"),
            "{}\n",
        )
        .unwrap();
    }

    let passing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(passing.success, "{:#?}", passing.violations);

    fs::remove_dir_all(project.path().join("packages/ui-kit")).unwrap();
    let failing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(!failing.success);
    assert!(failing.violations.iter().any(|violation| {
        violation.path == Path::new("packages")
            && violation.rule == "exists_count"
            && violation.message.contains("expected 2")
    }));
}

#[test]
fn unmatched_captured_tree_rule_is_match_only() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
rules:
  "@package":
    package.json: exists:1
structure:
  ./:
    packages/:
      "{package}/": "@package"
"#,
    );
    fs::create_dir_all(project.path().join("packages")).unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(report.success, "{:#?}", report.violations);
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
