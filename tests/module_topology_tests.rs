use std::fs;
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

fn module_topology_config() -> &'static str {
    r#"
extensions:
  module_topologies:
    - id: public_modules
      severity: high
      rust_exports:
        - src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: validation-tests
          purpose: supported CLI implementation
          roots:
            - src/cli
          public_exports:
            - cli
        - family: experimental_graph
          status: experimental
          owner: validation-tests
          purpose: contained experimental graph implementation
          roots:
            - src/experimental_graph
          public_exports:
            - experimental_graph
        - family: internal_only
          status: internal
          owner: validation-tests
          purpose: internal implementation detail
          roots:
            - src/internal_only
          visibility: internal
        - family: removed
          status: unsupported
          owner: validation-tests
          purpose: removed public module
          roots:
            - src/removed
          public_exports:
            - removed
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

fn write_passing_module_topology_files(project: &TempDir) {
    fs::create_dir_all(project.path().join("src/cli")).unwrap();
    fs::create_dir_all(project.path().join("src/experimental_graph")).unwrap();
    fs::create_dir_all(project.path().join("src/internal_only")).unwrap();
    fs::create_dir_all(project.path().join("src/removed")).unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "pub mod cli;\npub mod experimental_graph;\nmod internal_only;\n",
    )
    .unwrap();
}

#[test]
fn check_module_topology_passes_when_exports_and_roots_match() {
    let project = TempDir::new().unwrap();
    write_config(&project, module_topology_config());
    write_passing_module_topology_files(&project);

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn check_module_topology_reports_unclassified_export_missing_root_and_visibility_conflict() {
    let project = TempDir::new().unwrap();
    write_config(&project, module_topology_config());
    fs::create_dir_all(project.path().join("src/cli")).unwrap();
    fs::create_dir_all(project.path().join("src/experimental_graph")).unwrap();
    fs::create_dir_all(project.path().join("src/removed")).unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        concat!(
            "pub mod cli;\n",
            "pub mod experimental_graph;\n",
            "pub mod undocumented;\n",
            "pub mod internal_only;\n",
            "pub mod removed;\n"
        ),
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
            .any(|message| message.contains("root `src/internal_only`")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("public module/export `undocumented`")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("internal-only") && message.contains("internal_only")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("unsupported") && message.contains("removed")),
        "{messages:#?}"
    );
}

#[test]
fn check_module_topology_cli_json_reports_actionable_context() {
    let project = TempDir::new().unwrap();
    write_config(&project, module_topology_config());
    write_passing_module_topology_files(&project);
    fs::write(
        project.path().join("src/lib.rs"),
        "pub mod cli;\npub mod experimental_graph;\npub mod unclassified;\n",
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
            violation["path"] == "src/lib.rs"
                && violation["rule"] == "module_topology:public_modules"
                && violation["message"]
                    .as_str()
                    .unwrap()
                    .contains("unclassified")
                && violation["corrective_context"]
                    .as_str()
                    .unwrap()
                    .contains("module/export")
        }),
        "{report:#?}"
    );
}
