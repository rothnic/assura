use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_project(config_markdown: &str, markdown: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    let assura_dir = project.path().join(".assura");
    let docs_dir = project.path().join("docs");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::create_dir_all(&docs_dir).unwrap();
    fs::write(
        assura_dir.join("config.yml"),
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
    fs::write(docs_dir.join("note.md"), markdown).unwrap();
    project
}

#[test]
fn check_reports_configured_markdown_trailing_spaces() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let violations = json["violations"].as_array().unwrap();
    let finding = violations
        .iter()
        .find(|violation| violation["rule"] == "markdown_trailing_spaces")
        .expect("markdown trailing-space lint violation");

    assert_eq!(finding["path"], "docs/note.md");
    assert!(finding["message"].as_str().unwrap().contains("line 4"));
}

#[test]
fn fix_markdown_removes_blank_line_trailing_spaces_and_preserves_frontmatter() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n\nBody\n",
    );

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("changed 1 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("applied 1 fix(es)"),
        "stdout was:\n{stdout}"
    );

    let fixed = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(fixed, "---\ntitle: Note\n---\n\n# Note\n\nBody\n");

    let check = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();
    assert_eq!(check.status.code(), Some(0));
}

#[test]
fn fix_markdown_dry_run_reports_safe_fix_without_writing() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n\nBody\n",
    );
    let before = fs::read_to_string(project.path().join("docs/note.md")).unwrap();

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path())
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.safe-fix.markdown.v1");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_changed"], 0);
    assert_eq!(json["fixes_applied"], 0);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 1);

    let after = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(after, before);
}

#[test]
fn fix_markdown_json_reports_bounded_write_summary() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["dry_run"], false);
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_changed"], 1);
    assert_eq!(json["fixes_applied"], 1);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 1);
}

#[test]
fn fix_markdown_reports_noop_for_clean_configured_markdown() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n\n# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("changed 0 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("applied 0 fix(es)"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn check_leaves_markdown_lint_disabled_by_default() {
    let project = write_project(
        "          require_frontmatter: false\n",
        "---\ntitle: Note\n---\n   \n# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
}
