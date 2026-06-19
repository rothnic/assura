use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src/cli")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
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
    fs::write(root.join("src/lib.rs"), "pub mod cli;\n").unwrap();
}

#[test]
fn compiled_config_cli_supports_module_topology_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-module-topology-project");
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
        project.join("src/lib.rs"),
        "pub mod cli;\npub mod surprise;\n",
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
    assert!(stdout.contains("module_topology:public_modules"));
    assert!(stdout.contains("surprise"));
}
