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
fn check_exists_counts_ignore_excluded_children() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      max_lines: 100
      exists:
        "*.rs": "1"
    directories:
      exists:
        "tmp-*": "0"
    children:
      .assura/:
        files:
          naming: kebab-case
exclude:
  - "ignored.rs"
  - "tmp-ignored/**"
"#,
    );

    fs::write(project.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(project.path().join("ignored.rs"), "fn ignored() {}\n").unwrap();
    fs::create_dir(project.path().join("tmp-ignored")).unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
