use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_project(markdown: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  ./:
    children:
      docs/:
        markdown:
          required_sections:
            - Usage
            - API
"#,
    )
    .unwrap();
    fs::write(project.path().join("docs/note.md"), markdown).unwrap();
    project
}

fn write_project_with_config(config_markdown: &str, markdown: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    children:
      docs/:
        markdown:
{config_markdown}
"#
        ),
    )
    .unwrap();
    fs::write(project.path().join("docs/note.md"), markdown).unwrap();
    project
}

#[test]
fn fix_markdown_dry_run_reports_missing_required_sections_without_writing() {
    let project = write_project("---\ntitle: Note\n---\n# Note\n\nBody\n");
    let before = fs::read_to_string(project.path().join("docs/note.md")).unwrap();

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--rule")
        .arg("required-sections")
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["rule"], "required-sections");
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 2);
    assert_eq!(json["files"][0]["status"], "planned");
    assert_eq!(
        json["fixes"][0]["operation"],
        "insert_required_section_heading"
    );
    assert_eq!(json["fixes"][0]["inserted_text"], "## Usage");
    assert!(json["fixes"][0]["id"]
        .as_str()
        .unwrap()
        .starts_with("markdown.safe_fix."));

    let after = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(after, before);
}

#[test]
fn fix_markdown_required_sections_preserves_crlf_line_endings() {
    let project = write_project("---\r\ntitle: Note\r\n---\r\n# Note\r\n\r\nBody\r\n");

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--rule")
        .arg("required-sections")
        .arg("--apply")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let fixed = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert!(fixed.contains("Body\r\n\r\n## Usage\r\n\r\n## API\r\n"));
    assert!(!fixed.contains("Body\n\n## Usage\n"));
}

#[test]
fn markdown_required_sections_rejects_unsafe_heading_text() {
    let project = write_project_with_config(
        "          required_sections:\n            - \"\"\n",
        "# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("heading text cannot be empty"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn markdown_required_sections_rejects_duplicate_heading_text() {
    let project = write_project_with_config(
        "          required_sections:\n            - Usage\n            - Usage\n",
        "# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("duplicate heading text 'Usage'"),
        "stderr:\n{stderr}"
    );
}

#[test]
fn fix_markdown_appends_missing_required_sections_and_preserves_frontmatter() {
    let project = write_project("---\ntitle: Note\n---\n# Note\n\nBody\n");

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--rule")
        .arg("required-sections")
        .arg("--apply")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["files_changed"], 1);
    assert_eq!(json["fixes_applied"], 2);
    assert_eq!(json["fixes_after"], 0);
    assert_eq!(json["changed_paths"][0], "docs/note.md");

    let fixed = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(
        fixed,
        "---\ntitle: Note\n---\n# Note\n\nBody\n\n## Usage\n\n## API\n"
    );

    let check = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(0));
}
