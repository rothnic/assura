use std::fs;
use std::path::Path;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn expanded_support_matrix_config(entries: &str) -> String {
    format!(
        r#"
extensions:
  manifest_semantics:
    - id: cargo_public
      severity: high
      manifests:
        - path: Cargo.toml
          package: sample-tool
          publish: public
          binaries:
            - sample-tool
  support_matrices:
    - id: public_surface
      severity: high
      docs_claim_sources:
        - path: docs/support.md
      manifest_policies:
        - cargo_public
      entries:
{entries}
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

fn write_expanded_support_matrix_files(project: &TempDir, docs_status: &str) {
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/support.md"),
        format!(
            r#"# Support

| Command | Status | Evidence |
| --- | --- | --- |
| `assura info` | {docs_status} | fixture |
"#
        ),
    )
    .unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        r#"
[package]
name = "sample-tool"
version = "0.1.0"
edition = "2021"
publish = true

[[bin]]
name = "sample-tool"
path = "src/main.rs"
"#,
    )
    .unwrap();
}

#[test]
fn check_support_matrix_passes_with_docs_claims_and_manifest_surfaces() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &expanded_support_matrix_config(
            r#"        - surface: "command:assura info"
          status: supported
        - surface: "package:sample-tool"
          status: supported
        - surface: "binary:sample-tool"
          status: supported"#,
        ),
    );
    write_expanded_support_matrix_files(&project, "Supported");

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
}

#[test]
fn check_support_matrix_reports_unclassified_manifest_surface() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &expanded_support_matrix_config(
            r#"        - surface: "command:assura info"
          status: supported
        - surface: "binary:sample-tool"
          status: supported"#,
        ),
    );
    write_expanded_support_matrix_files(&project, "Supported");

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("Cargo.toml")
                && violation.rule == "support_matrix:public_surface"
                && violation.message.contains("package:sample-tool")
                && violation.message.contains("cargo_public")
        }),
        "{:#?}",
        report.violations
    );
}

#[test]
fn check_support_matrix_reports_unclassified_docs_claim_status() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &expanded_support_matrix_config(
            r#"        - surface: "package:sample-tool"
          status: supported
        - surface: "binary:sample-tool"
          status: supported"#,
        ),
    );
    write_expanded_support_matrix_files(&project, "Experimental diagnostic");

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("docs/support.md")
                && violation.rule == "support_matrix:public_surface"
                && violation.message.contains("command:assura info")
                && violation.message.contains("claimed as `experimental`")
        }),
        "{:#?}",
        report.violations
    );
}

#[test]
fn check_support_matrix_reports_supported_docs_claim_contradiction() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &expanded_support_matrix_config(
            r#"        - surface: "command:assura info"
          status: experimental
        - surface: "package:sample-tool"
          status: supported
        - surface: "binary:sample-tool"
          status: supported"#,
        ),
    );
    write_expanded_support_matrix_files(&project, "Supported diagnostic");

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("docs/support.md")
                && violation.rule == "support_matrix:public_surface"
                && violation.message.contains("command:assura info")
                && violation.message.contains("as `experimental`")
                && violation.message.contains("says it is `supported`")
        }),
        "{:#?}",
        report.violations
    );
}
