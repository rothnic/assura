use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("docs/analysis")).unwrap();
    fs::create_dir_all(root.join("docs/evidence")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      severity: high
      active:
        - docs/**/*.md
      require_frontmatter_status:
        - docs/analysis/*.md
      allowed_statuses:
        - active
        - planned
      claim_patterns:
        - id: performance_current
          pattern: "2x"
          evidence_files:
            - docs/evidence/performance.md
structure:
  ./:
    files:
      allow_extra: true
    directories:
      allow_extra: true
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(
        root.join("docs/analysis/current.md"),
        "---\nstatus: active\n---\n# Current\n",
    )
    .unwrap();
    fs::write(
        root.join("docs/evidence/performance.md"),
        "# Evidence\n2x\n",
    )
    .unwrap();
}

#[test]
fn compiled_config_cli_supports_docs_lifecycle_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-docs-lifecycle-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project);

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    fs::write(
        project.join("docs/analysis/current.md"),
        "---\nstatus: active\n---\n# Current\nThe 2x claim drifted.\n",
    )
    .unwrap();
    fs::write(project.join("docs/evidence/performance.md"), "# Evidence\n").unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("docs_lifecycle:project_docs"));
    assert!(stdout.contains("performance_current"));
}
