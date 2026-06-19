use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src/cli/check")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
extensions:
  test_relationships:
    - id: supported_tests
      severity: high
      relationships:
        - source: "src/cli/check/*.rs"
          required_tests:
            - "tests/custom_constraints_tests.rs"
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
        root.join("src/cli/check/test_relationship.rs"),
        "pub fn validate() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/custom_constraints_tests.rs"),
        "#[test]\nfn test_relationship_rule() {}\n",
    )
    .unwrap();
}

#[test]
fn compiled_config_cli_supports_test_relationship_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-test-relationship-project");
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

    fs::remove_file(project.join("tests/custom_constraints_tests.rs")).unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("test_relationship:supported_tests"));
    assert!(stdout.contains("tests/custom_constraints_tests.rs"));
}
