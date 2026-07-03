use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    for dir in [".assura", "docs/requirements", "records"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".assura/config.yml"),
        r#"extensions:
  requirements_traceability:
    - id: document_traceability
      severity: high
      requirements_collection: requirements
      priority_field: priority
      high_priority_values:
        - high
      coverage_collections:
        - claims
      claim_collections:
        - claims
      evidence_collections:
        - evidence
      source_document_collections:
        - source_documents
structure:
  ./:
    required: false
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(
        root.join("docs/requirements/req-auth.md"),
        "---\nid: req-auth\npriority: high\n---\n# Auth\n",
    )
    .unwrap();
    fs::write(
        root.join("records/claim.json"),
        r#"{"id":"claim-auth","requirements":["req-auth"]}"#,
    )
    .unwrap();
}

#[test]
fn compiled_config_cli_preserves_but_does_not_enforce_requirements_traceability() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-traceability-project");
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

    let check = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(!String::from_utf8_lossy(&check.stdout).contains("requirements_traceability:"));
}
