use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

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

fn command_surface_config() -> &'static str {
    r#"
extensions:
  custom_constraints:
    - id: command_surface_docs
      type: command_surface_docs
      source: "docs/*.md"
      target: ".assura/command-surface.yml"
      severity: high
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

fn write_command_contract(project: &TempDir) {
    fs::write(
        project.path().join(".assura/command-surface.yml"),
        r#"
commands:
  - name: "assura check"
    allow_positionals: true
    flags:
      --format:
        aliases: ["-f"]
        takes_value: true
        values: ["text", "json", "yaml", "agent"]
      --agent:
        takes_value: true
        values: ["generic", "codex"]
        requires:
          --format: "agent"
      --fail-fast: {}
"#,
    )
    .unwrap();
}

fn release_contract_config() -> &'static str {
    r#"
extensions:
  release_contracts:
    - id: cli_release
      severity: high
      artifacts:
        - name: example-linux-x86_64.tar.gz
          checksum_sidecar: true
        - name: example-darwin-aarch64.tar.gz
          checksum_sidecar: true
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
      installer_files:
        - scripts/install.sh
      allowed_url_branches:
        - main
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

fn write_release_contract_files(project: &TempDir, workflow: &str, docs: &str, installer: &str) {
    fs::create_dir_all(project.path().join(".github/workflows")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("scripts")).unwrap();
    fs::write(
        project.path().join(".github/workflows/release.yml"),
        workflow,
    )
    .unwrap();
    fs::write(project.path().join("docs/install.md"), docs).unwrap();
    fs::write(project.path().join("scripts/install.sh"), installer).unwrap();
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

#[test]
fn check_custom_paired_file_constraint_handles_root_source_parent() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        &permissive_pair_config("*.md", "{source_parent}/{stem}_docs.md", ""),
    );
    fs::write(project.path().join("README.md"), "# Project\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("README.md"));
    assert_eq!(violation.rule, "custom:source_test_pair");
    assert!(violation.message.contains("README_docs.md"));
}

#[test]
fn check_command_surface_docs_passes_supported_examples() {
    let project = TempDir::new().unwrap();
    write_config(&project, command_surface_config());
    write_command_contract(&project);
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/usage.md"),
        r#"# Usage

```bash
assura check --format agent --agent codex .
cargo run --quiet -- check -f json .
```
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_command_surface_docs_reports_unsupported_flag_and_value() {
    let project = TempDir::new().unwrap();
    write_config(&project, command_surface_config());
    write_command_contract(&project);
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/usage.md"),
        r#"# Usage

Run `assura check --format codex-hook .` for agent output.

```bash
assura check --maturity .
```
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 2);
    let messages = report
        .violations
        .iter()
        .map(|violation| violation.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("unsupported value `codex-hook`")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("unsupported flag `--maturity`")),
        "{messages:#?}"
    );
    assert!(report
        .violations
        .iter()
        .all(|violation| violation.rule == "custom:command_surface_docs"));
}

#[test]
fn check_command_surface_docs_validates_cargo_run_examples() {
    let project = TempDir::new().unwrap();
    write_config(&project, command_surface_config());
    write_command_contract(&project);
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/usage.md"),
        r#"# Usage

```bash
cargo run --quiet -- check --format codex-hook .
```
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    assert!(report.violations[0]
        .message
        .contains("unsupported value `codex-hook`"));
}

#[test]
fn check_command_surface_docs_enforces_flag_requirements() {
    let project = TempDir::new().unwrap();
    write_config(&project, command_surface_config());
    write_command_contract(&project);
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/usage.md"),
        r#"# Usage

```bash
assura check --agent codex .
```
"#,
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    assert!(report.violations[0]
        .message
        .contains("requires flag `--format` to be `agent`"));
}

#[test]
fn check_command_surface_docs_rejects_unreliable_contracts() {
    let project = TempDir::new().unwrap();
    write_config(&project, command_surface_config());
    fs::write(
        project.path().join(".assura/command-surface.yml"),
        r#"
commands:
  - name: "assura check"
    flags:
      --format:
        aliases: ["--agent"]
      --agent: {}
"#,
    )
    .unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/usage.md"),
        "Run `assura check --format json .`.\n",
    )
    .unwrap();

    let error = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap_err();
    let error = error.to_string();

    assert!(
        error.contains("collides") || error.contains("duplicate flag"),
        "unexpected error: {error}"
    );
}

#[test]
fn check_release_contract_passes_when_workflow_docs_and_installer_match() {
    let project = TempDir::new().unwrap();
    write_config(&project, release_contract_config());
    write_release_contract_files(
        &project,
        r#"
name: release
jobs:
  build:
    strategy:
      matrix:
        asset:
          - example-linux-x86_64.tar.gz
          - example-linux-x86_64.tar.gz.sha256
          - example-darwin-aarch64.tar.gz
          - example-darwin-aarch64.tar.gz.sha256
"#,
        "Download example-linux-x86_64.tar.gz and verify example-linux-x86_64.tar.gz.sha256. Download example-darwin-aarch64.tar.gz and verify example-darwin-aarch64.tar.gz.sha256.\n",
        r#"#!/bin/sh
curl -L https://raw.githubusercontent.com/example/project/main/releases/example-linux-x86_64.tar.gz
curl -L https://raw.githubusercontent.com/example/project/main/releases/example-linux-x86_64.tar.gz.sha256
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_release_contract_reports_documented_asset_outside_contract() {
    let project = TempDir::new().unwrap();
    write_config(&project, release_contract_config());
    write_release_contract_files(
        &project,
        r#"
uploads:
  - example-linux-x86_64.tar.gz
  - example-linux-x86_64.tar.gz.sha256
  - example-darwin-aarch64.tar.gz
  - example-darwin-aarch64.tar.gz.sha256
"#,
        "Download example-linux-x86_64.tar.gz, example-darwin-aarch64.tar.gz, and example-windows-x86_64.zip.\nVerify example-linux-x86_64.tar.gz.sha256 and example-darwin-aarch64.tar.gz.sha256.\n",
        r#"#!/bin/sh
curl -L https://raw.githubusercontent.com/example/project/main/releases/example-linux-x86_64.tar.gz
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new("docs/install.md")
                && violation.rule == "release_contract:cli_release"
                && violation.severity == "high"
                && violation.message.contains("example-windows-x86_64.zip")
        }),
        "{:#?}",
        report.violations
    );
}

#[test]
fn check_release_contract_reports_installer_url_branch_and_asset_drift() {
    let project = TempDir::new().unwrap();
    write_config(&project, release_contract_config());
    write_release_contract_files(
        &project,
        r#"
uploads:
  - example-linux-x86_64.tar.gz
  - example-linux-x86_64.tar.gz.sha256
  - example-darwin-aarch64.tar.gz
  - example-darwin-aarch64.tar.gz.sha256
"#,
        "Download example-linux-x86_64.tar.gz and example-darwin-aarch64.tar.gz. Verify example-linux-x86_64.tar.gz.sha256 and example-darwin-aarch64.tar.gz.sha256.\n",
        r#"#!/bin/sh
curl -L https://raw.githubusercontent.com/example/project/dev/releases/example-freebsd-x86_64.tar.gz
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    let messages = report
        .violations
        .iter()
        .map(|violation| violation.message.as_str())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("branch `dev`")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("example-freebsd-x86_64.tar.gz")),
        "{messages:#?}"
    );
}

#[test]
fn check_release_contract_reports_missing_workflow_checksum_sidecar() {
    let project = TempDir::new().unwrap();
    write_config(&project, release_contract_config());
    write_release_contract_files(
        &project,
        r#"
uploads:
  - example-linux-x86_64.tar.gz
  - example-linux-x86_64.tar.gz.sha256
  - example-darwin-aarch64.tar.gz
"#,
        "Download example-linux-x86_64.tar.gz and example-darwin-aarch64.tar.gz. Verify example-linux-x86_64.tar.gz.sha256 and example-darwin-aarch64.tar.gz.sha256.\n",
        r#"#!/bin/sh
curl -L https://raw.githubusercontent.com/example/project/main/releases/example-linux-x86_64.tar.gz
"#,
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert!(
        report.violations.iter().any(|violation| {
            violation.path == Path::new(".github/workflows/release.yml")
                && violation.rule == "release_contract:cli_release"
                && violation
                    .message
                    .contains("example-darwin-aarch64.tar.gz.sha256")
        }),
        "{:#?}",
        report.violations
    );
}

#[test]
fn check_release_contract_cli_json_reports_actionable_rule_context() {
    let project = TempDir::new().unwrap();
    write_config(&project, release_contract_config());
    write_release_contract_files(
        &project,
        r#"
uploads:
  - example-linux-x86_64.tar.gz
  - example-linux-x86_64.tar.gz.sha256
  - example-darwin-aarch64.tar.gz
  - example-darwin-aarch64.tar.gz.sha256
"#,
        "Download example-linux-x86_64.tar.gz, example-darwin-aarch64.tar.gz, and example-plan9-x86_64.zip.\nVerify example-linux-x86_64.tar.gz.sha256 and example-darwin-aarch64.tar.gz.sha256.\n",
        r#"#!/bin/sh
curl -L https://raw.githubusercontent.com/example/project/main/releases/example-linux-x86_64.tar.gz
"#,
    );

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
            violation["path"] == "docs/install.md"
                && violation["rule"] == "release_contract:cli_release"
                && violation["message"]
                    .as_str()
                    .unwrap()
                    .contains("example-plan9-x86_64.zip")
                && violation["corrective_context"]
                    .as_str()
                    .unwrap()
                    .contains("release artifact contract")
        }),
        "{report:#?}"
    );
}
