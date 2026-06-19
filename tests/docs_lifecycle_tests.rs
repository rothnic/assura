use std::fs;
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

fn docs_lifecycle_config(include_historical_exception: bool) -> String {
    let exception = if include_historical_exception {
        "\n      historical_exceptions:\n        - docs/archive/**"
    } else {
        ""
    };
    format!(
        r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      severity: high
      active:
        - docs/**/*.md
      historical:
        - docs/archive/**
      require_frontmatter_status:
        - docs/goals/*.md
        - docs/analysis/*.md
      allowed_statuses:
        - active
        - planned
        - completed
        - archived
        - historical
      claim_patterns:
        - id: performance_current
          pattern: "2x"
          evidence_files:
            - docs/evidence/performance.md{exception}
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

fn write_passing_docs(project: &TempDir) {
    fs::create_dir_all(project.path().join("docs/goals")).unwrap();
    fs::create_dir_all(project.path().join("docs/archive")).unwrap();
    fs::create_dir_all(project.path().join("docs/evidence")).unwrap();
    fs::write(
        project.path().join("docs/goals/next.md"),
        "---\nstatus: planned\n---\n# Next\nSee [old](../archive/old.md).\nThe 2x claim is current.\n",
    )
    .unwrap();
    fs::write(project.path().join("docs/archive/old.md"), "# Old\n").unwrap();
    fs::write(
        project.path().join("docs/evidence/performance.md"),
        "# Performance\nThe 2x claim is checked here.\n",
    )
    .unwrap();
}

#[test]
fn check_docs_lifecycle_passes_for_current_claims_and_excepted_history() {
    let project = TempDir::new().unwrap();
    write_config(&project, &docs_lifecycle_config(true));
    write_passing_docs(&project);

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_docs_lifecycle_allows_excepted_historical_claims_without_evidence() {
    let project = TempDir::new().unwrap();
    write_config(&project, &docs_lifecycle_config(true));
    fs::create_dir_all(project.path().join("docs/goals")).unwrap();
    fs::create_dir_all(project.path().join("docs/archive")).unwrap();
    fs::write(
        project.path().join("docs/goals/next.md"),
        "---\nstatus: planned\n---\n# Next\nSee [old](../archive/old.md).\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/archive/old.md"),
        "# Old\nHistorical 2x claim preserved for context.\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_docs_lifecycle_accepts_all_configured_statuses() {
    let project = TempDir::new().unwrap();
    write_config(&project, &docs_lifecycle_config(false));
    fs::create_dir_all(project.path().join("docs/analysis")).unwrap();
    fs::create_dir_all(project.path().join("docs/evidence")).unwrap();
    for status in ["active", "completed", "archived", "historical"] {
        fs::write(
            project.path().join(format!("docs/analysis/{status}.md")),
            format!("---\nstatus: {status}\n---\n# {status}\n"),
        )
        .unwrap();
    }

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
}

#[test]
fn check_docs_lifecycle_reports_missing_status_historical_link_and_claim_drift() {
    let project = TempDir::new().unwrap();
    write_config(&project, &docs_lifecycle_config(false));
    fs::create_dir_all(project.path().join("docs/analysis")).unwrap();
    fs::create_dir_all(project.path().join("docs/archive")).unwrap();
    fs::create_dir_all(project.path().join("docs/evidence")).unwrap();
    fs::write(
        project.path().join("docs/analysis/current.md"),
        "# Current\nSee [old](../archive/old.md).\nThe 2x claim appears here.\n",
    )
    .unwrap();
    fs::write(project.path().join("docs/archive/old.md"), "# Old\n").unwrap();
    fs::write(
        project.path().join("docs/evidence/performance.md"),
        "# Evidence\n",
    )
    .unwrap();

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
            .any(|message| message.contains("requires frontmatter status")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("links to historical doc")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("claim `performance_current`")),
        "{messages:#?}"
    );
}

#[test]
fn check_docs_lifecycle_cli_json_reports_actionable_context() {
    let project = TempDir::new().unwrap();
    write_config(&project, &docs_lifecycle_config(false));
    fs::create_dir_all(project.path().join("docs/analysis")).unwrap();
    fs::create_dir_all(project.path().join("docs/evidence")).unwrap();
    fs::write(
        project.path().join("docs/evidence/performance.md"),
        "# Evidence\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/analysis/current.md"),
        "# Current\nThe 2x claim appears here.\n",
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
            normalized_json_path(&violation["path"]) == "docs/analysis/current.md"
                && violation["rule"] == "docs_lifecycle:project_docs"
                && violation["message"]
                    .as_str()
                    .unwrap()
                    .contains("performance_current")
                && violation["message"]
                    .as_str()
                    .unwrap()
                    .contains("docs/evidence/performance.md")
                && violation["corrective_context"]
                    .as_str()
                    .unwrap()
                    .contains("lifecycle metadata")
        }),
        "{report:#?}"
    );
}
