use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
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
      command_contracts:
        - .assura/command-surface.yml
      docs_claim_sources:
        - path: docs/support.md
      manifest_policies:
        - cargo_public
      entries:
        - surface: "command:assura check"
          status: supported
        - surface: "command:assura info"
          status: experimental
        - surface: "package:sample-tool"
          status: supported
        - surface: "binary:sample-tool"
          status: supported
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
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("docs/support.md"),
        r#"# Support

| Command | Status | Evidence |
| --- | --- | --- |
| `assura check` | Supported | fixture |
| `assura info` | Experimental | fixture |
"#,
    )
    .unwrap();
    fs::write(
        root.join("Cargo.toml"),
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
    fs::write(
        root.join(".assura/command-surface.yml"),
        r#"
commands:
  - name: "assura check"
    allow_positionals: true
  - name: "assura status"
    allow_positionals: true
"#,
    )
    .unwrap();
}

fn write_manifest_gap_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
extensions:
  manifest_semantics:
    - id: cargo_public
      manifests:
        - path: Cargo.toml
          package: sample-tool
          publish: public
          binaries:
            - sample-tool
  support_matrices:
    - id: public_surface
      manifest_policies:
        - cargo_public
      entries:
        - surface: "binary:sample-tool"
          status: supported
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
        root.join("Cargo.toml"),
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
fn compiled_config_cli_supports_support_matrix_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-support-matrix-project");
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

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("support_matrix:public_surface"));
    assert!(stdout.contains("command:assura status"));

    fs::write(
        project.join("docs/support.md"),
        r#"# Support

| Command | Status | Evidence |
| --- | --- | --- |
| `assura check` | Supported | fixture |
| `assura info` | Supported | fixture |
"#,
    )
    .unwrap();
    let docs_claim_invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(docs_claim_invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&docs_claim_invalid.stdout);
    assert!(stdout.contains("support_matrix:public_surface"));
    assert!(stdout.contains("command:assura info"));
    assert!(stdout.contains("says it is `supported`"));

    fs::write(
        project.join(".assura/command-surface.yml"),
        "commands:\n  - name: \"assura check\"\n    allow_positionals: true\n",
    )
    .unwrap();
    fs::write(
        project.join("docs/support.md"),
        r#"# Support

| Command | Status | Evidence |
| --- | --- | --- |
| `assura check` | Supported | fixture |
| `assura info` | Experimental | fixture |
"#,
    )
    .unwrap();
    let valid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        valid.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&valid.stdout),
        String::from_utf8_lossy(&valid.stderr)
    );
}

#[test]
fn compiled_config_cli_preserves_support_matrix_manifest_sources() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-support-matrix-manifest-project");
    let compiled_config = temp.path().join("manifest-check-config.bin");
    write_manifest_gap_project(&project);

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

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("support_matrix:public_surface"));
    assert!(stdout.contains("package:sample-tool"));
}
