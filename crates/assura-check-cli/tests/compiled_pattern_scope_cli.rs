use std::fs;
use std::process::Command;

#[test]
fn compiled_config_preserves_direct_and_recursive_file_patterns() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("pattern-scope-project");
    let config_dir = project.join(".assura");
    let nested = project.join("src");
    let compiled_config = temp.path().join("pattern-scope.bin");
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        config_dir.join("config.yml"),
        r#"
rules:
  "@direct": { max_lines: 1 }
  "@recursive": { max_lines: 2 }
structure:
  ./:
    "./**/*.ts": "@recursive"
    "./*.ts": "@direct"
"#,
    )
    .unwrap();
    fs::write(project.join("root.ts"), "a\nb\n").unwrap();
    fs::write(nested.join("nested.ts"), "a\nb\n").unwrap();

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(config_dir.join("config.yml"))
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
    assert_eq!(check.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(stdout.contains("root.ts"), "stdout:\n{stdout}");
    assert!(!stdout.contains("nested.ts"), "stdout:\n{stdout}");
}
