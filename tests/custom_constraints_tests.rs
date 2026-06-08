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
