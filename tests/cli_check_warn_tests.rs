use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

#[test]
fn check_warn_reports_violations_but_exits_successfully() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--warn")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file_naming"), "stdout was:\n{}", stdout);
}
