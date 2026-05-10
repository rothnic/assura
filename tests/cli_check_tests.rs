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

fn baseline_config() -> &'static str {
    r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
      required:
        - README.md
      naming: kebab-case
    children:
      .assura/:
        files:
          naming: kebab-case
      src/:
        files:
          naming: snake_case
          max_lines: 50
      docs/:
        files:
          naming: kebab-case
        markdown:
          require_frontmatter: true
exclude:
  - target/**
"#
}

#[test]
fn check_passes_valid_structure() {
    let project = TempDir::new().unwrap();
    write_config(&project, baseline_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("src/main_file.rs"), "fn main() {}\n").unwrap();
    fs::write(
        project.path().join("docs/project-note.md"),
        "---\ntitle: Project Note\n---\n# Project Note\n",
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected valid project to pass:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn check_fails_bad_file_naming() {
    let project = TempDir::new().unwrap();
    write_config(&project, baseline_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("src/BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file_naming"), "stdout was:\n{}", stdout);
    assert!(stdout.contains("BadName.rs"), "stdout was:\n{}", stdout);
}

#[test]
fn check_fails_missing_required_file() {
    let project = TempDir::new().unwrap();
    write_config(&project, baseline_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(project.path().join("src/main_file.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("required_file"), "stdout was:\n{}", stdout);
    assert!(stdout.contains("README.md"), "stdout was:\n{}", stdout);
}

#[test]
fn check_ignores_excluded_paths() {
    let project = TempDir::new().unwrap();
    write_config(&project, baseline_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::create_dir(project.path().join("target")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("src/main_file.rs"), "fn main() {}\n").unwrap();
    fs::write(project.path().join("target/BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "excluded target file should not fail:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_fails_missing_markdown_frontmatter() {
    let project = TempDir::new().unwrap();
    write_config(&project, baseline_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("src/main_file.rs"), "fn main() {}\n").unwrap();
    fs::write(
        project.path().join("docs/project-note.md"),
        "# Project Note\n",
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("markdown_frontmatter"),
        "stdout was:\n{}",
        stdout
    );
}
