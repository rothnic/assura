use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path, file_name: &str) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(root.join("src").join(file_name), "").unwrap();
}

#[test]
fn compiled_config_cli_validates_with_precompiled_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-project");
    let compiled_config = temp.path().join("check-config.json");
    write_project(&project, "valid-file.ts");

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

    fs::rename(
        project.join("src").join("valid-file.ts"),
        project.join("src").join("BadName.ts"),
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
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("BadName.ts"));
}

#[test]
fn compile_config_rejects_invalid_semantics_without_full_cli_validator() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("invalid-compiled-project");
    let compiled_config = temp.path().join("check-config.bin");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      naming: invalid_case
"#,
    )
    .unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();

    assert_eq!(compile.status.code(), Some(3));
    assert!(!compiled_config.exists());
    let stderr = String::from_utf8_lossy(&compile.stderr);
    assert!(stderr.contains("invalid_case"), "stderr:\n{stderr}");
    assert!(
        stderr.contains("not a valid naming convention"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn compiled_config_cli_uses_default_project_artifact_path() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-default-project");
    let compiled_config = project.join(".assura/check-config.bin");
    write_project(&project, "valid-file.ts");

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

    let valid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--quiet")
        .current_dir(&project)
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
fn compiled_config_cli_rejects_stale_default_project_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-default-stale-project");
    let compiled_config = project.join(".assura/check-config.bin");
    write_project(&project, "valid-file.ts");

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
        project.join(".assura/config.yml"),
        "structure:\n  ./:\n    files:\n      naming: snake_case\n",
    )
    .unwrap();

    let stale = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--quiet")
        .current_dir(&project)
        .output()
        .unwrap();

    assert_eq!(stale.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("compiled config is stale"));
}

#[test]
fn compiled_config_cli_supports_direct_policy_fast_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-direct-policy-project");
    let compiled_config = temp.path().join("check-config.bin");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
        - Cargo.toml
      forbidden_patterns:
        - "*.tmp"
      allow_extra: false
    directories:
      allowed_names:
        - .assura
        - src
      forbidden_patterns:
        - dist
      allow_extra: false
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(project.join("README.md"), "").unwrap();
    fs::write(project.join("Cargo.toml"), "").unwrap();
    fs::write(project.join("src").join("lib.rs"), "").unwrap();

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

    fs::create_dir(project.join("dist")).unwrap();
    fs::write(project.join("secret.tmp"), "").unwrap();

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    assert!(stdout.contains("forbidden_directory"));
    assert!(stdout.contains("forbidden_file"));
}

#[test]
fn compiled_config_cli_supports_custom_constraint_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-custom-project");
    let compiled_config = temp.path().join("check-config.bin");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::create_dir_all(project.join("src")).unwrap();
    fs::create_dir_all(project.join("tests")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "src/*.rs"
      target: "tests/{stem}_test.rs"
      severity: high
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
    fs::write(project.join("src").join("parser.rs"), "pub fn parse() {}\n").unwrap();

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
    assert!(
        stdout.contains("custom:source_test_pair"),
        "stdout:\n{stdout}"
    );
    assert!(stdout.contains("tests/parser_test.rs"), "stdout:\n{stdout}");

    fs::write(
        project.join("tests").join("parser_test.rs"),
        "#[test]\nfn parser() {}\n",
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
fn compiled_config_cli_rejects_stale_source_config_when_checked() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project, "valid-file.ts");
    let config_path = project.join(".assura/config.yml");

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(&config_path)
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
        &config_path,
        "structure:\n  ./:\n    files:\n      naming: snake_case\n",
    )
    .unwrap();

    let stale = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--config")
        .arg(&config_path)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert_eq!(stale.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&stale.stderr).contains("compiled config is stale"));
}

#[test]
fn compiled_config_cli_rejects_artifact_for_different_project_root() {
    let temp = tempfile::tempdir().unwrap();
    let source_project = temp.path().join("compiled-source-project");
    let other_project = temp.path().join("compiled-other-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&source_project, "valid-file.ts");
    write_project(&other_project, "valid-file.ts");

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(source_project.join(".assura/config.yml"))
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

    let stale = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&other_project)
        .output()
        .unwrap();

    assert_eq!(stale.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&stale.stderr);
    assert!(
        stderr.contains("compiled config is stale"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn compiled_config_cli_rejects_same_bytes_from_different_config_path() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-path-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project, "valid-file.ts");
    let source_config = project.join(".assura/config.yml");
    let moved_config = project.join(".assura/renamed-config.yml");

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(&source_config)
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
    fs::copy(&source_config, &moved_config).unwrap();

    let stale = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--config")
        .arg(&moved_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert_eq!(stale.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&stale.stderr);
    assert!(
        stderr.contains("compiled config is stale"),
        "stderr:\n{stderr}"
    );
}
