use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
extensions:
  manifest_semantics:
    - id: cargo_workspace
      severity: high
      manifests:
        - path: Cargo.toml
          package: example-root
          role: public
          version: "0.1.0"
          rust_version: "1.70.0"
          license: MIT
          publish: public
          description_required_terms: ["structure-first"]
          description_forbidden_terms: ["dependency graph validation"]
          keywords: ["structure"]
          binaries: ["example-root"]
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
name = "example-root"
version = "0.1.0"
edition = "2021"
description = "Structure-first repository validation"
license = "MIT"
rust-version = "1.70.0"
keywords = ["structure", "validation"]

[[bin]]
name = "example-root"
path = "src/main.rs"
"#,
    )
    .unwrap();
}

#[test]
fn compiled_config_cli_supports_manifest_semantics_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-manifest-semantics-project");
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
        project.join("Cargo.toml"),
        r#"
[package]
name = "example-root"
version = "0.1.0"
edition = "2021"
description = "Dependency graph validation for Rust workspaces"
license = "MIT"
rust-version = "1.70.0"
keywords = ["validation"]

[[bin]]
name = "wrong-name"
path = "src/main.rs"
"#,
    )
    .unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("manifest_semantics:cargo_workspace"));
    assert!(stdout.contains("dependency graph validation"));
}
