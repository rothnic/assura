use std::fs;
use std::process::Command;

use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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

fn write_project_with_extra_markdown(config_markdown: &str, markdown: &str) -> TempDir {
    let project = write_project(config_markdown, markdown);
    fs::write(project.path().join("loose.md"), "# Loose\n   \n").unwrap();
    fs::write(project.path().join("notes.txt"), "   \n").unwrap();
    project
}

fn write_project_with_combined_markdown_fixes(markdown: &str) -> TempDir {
    write_project(
        "          lint_trailing_spaces: true\n          required_sections:\n            - Usage\n            - API\n",
        markdown,
    )
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

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["success"], true);
    let violations = json["violations"].as_array().unwrap();
    let finding = violations
        .iter()
        .find(|violation| violation["rule"] == "markdown_trailing_spaces")
        .expect("markdown trailing-space lint violation");

    assert_eq!(finding["path"], "docs/note.md");
    assert_eq!(finding["blocking"], false);
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
        .arg("--apply")
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
    assert_eq!(json["mode"], "dry_run");
    assert_eq!(json["rule"], "all");
    assert_eq!(json["dry_run"], true);
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_changed"], 0);
    assert_eq!(json["fixes_applied"], 0);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 1);
    assert_eq!(json["fixes_before"], 1);
    assert_eq!(json["fixes_after"], 1);
    assert_eq!(json["changed_paths"].as_array().unwrap().len(), 0);
    assert_eq!(json["applied_fix_ids"].as_array().unwrap().len(), 0);
    assert_eq!(json["files"][0]["path"], "docs/note.md");
    assert_eq!(json["files"][0]["status"], "planned");
    assert_eq!(json["files"][0]["fixes_planned"], 1);
    assert_eq!(json["files"][0]["fixes_applied"], 0);
    assert_eq!(json["fixes"][0]["path"], "docs/note.md");
    assert_eq!(
        json["fixes"][0]["operation"],
        "remove_blank_line_trailing_spaces"
    );
    assert_eq!(json["fixes"][0]["status"], "planned");
    assert_eq!(json["fixes"][0]["line"], 4);
    assert_eq!(json["fixes"][0]["column"], 1);
    assert_eq!(json["fixes"][0]["before_trailing_whitespace"], 3);
    assert_eq!(json["fixes"][0]["after_trailing_whitespace"], 3);
    assert!(json["fixes"][0]["id"]
        .as_str()
        .unwrap()
        .starts_with("markdown.safe_fix."));
    assert_eq!(
        json["rollback"]["guidance"],
        "Use version control to inspect or revert applied safe fixes."
    );

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
        .arg("--apply")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "apply");
    assert_eq!(json["dry_run"], false);
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_changed"], 1);
    assert_eq!(json["fixes_applied"], 1);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 1);
    assert_eq!(json["fixes_before"], 1);
    assert_eq!(json["fixes_after"], 0);
    assert_eq!(json["changed_paths"][0], "docs/note.md");
    assert_eq!(json["applied_fix_ids"][0], json["fixes"][0]["id"]);
    assert_eq!(json["files"][0]["status"], "changed");
    assert_eq!(json["files"][0]["fixes_applied"], 1);
    assert_eq!(json["fixes"][0]["status"], "applied");
    assert_eq!(json["fixes"][0]["after_trailing_whitespace"], 0);

    let rerun = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--apply")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(rerun.status.code(), Some(0));
    let rerun_json: serde_json::Value = serde_json::from_slice(&rerun.stdout).unwrap();
    assert_eq!(rerun_json["files_changed"], 0);
    assert_eq!(rerun_json["fixes_applied"], 0);
    assert_eq!(rerun_json["files_would_change"], 0);
    assert_eq!(rerun_json["fixes_would_apply"], 0);
    assert_eq!(rerun_json["files"][0]["status"], "unchanged");
}

#[test]
fn fix_markdown_all_dry_run_reports_every_supported_safe_fix_without_writing() {
    let project =
        write_project_with_combined_markdown_fixes("---\ntitle: Note\n---\n   \n# Note\n\nBody\n");
    let before = fs::read_to_string(project.path().join("docs/note.md")).unwrap();

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["rule"], "all");
    assert_eq!(json["mode"], "dry_run");
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_would_change"], 1);
    assert_eq!(json["fixes_would_apply"], 3);
    assert_eq!(json["fixes_applied"], 0);
    let operations = json["fixes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|fix| fix["operation"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        operations,
        vec![
            "remove_blank_line_trailing_spaces",
            "insert_required_section_heading",
            "insert_required_section_heading"
        ]
    );

    let after = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(after, before);
}

#[test]
fn fix_markdown_all_applies_supported_safe_fixes_once_and_is_idempotent() {
    let project =
        write_project_with_combined_markdown_fixes("---\ntitle: Note\n---\n   \n# Note\n\nBody\n");

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--apply")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["rule"], "all");
    assert_eq!(json["mode"], "apply");
    assert_eq!(json["files_checked"], 1);
    assert_eq!(json["files_changed"], 1);
    assert_eq!(json["changed_paths"].as_array().unwrap().len(), 1);
    assert_eq!(json["changed_paths"][0], "docs/note.md");
    assert_eq!(json["fixes_would_apply"], 3);
    assert_eq!(json["fixes_applied"], 3);
    assert_eq!(json["applied_fix_ids"].as_array().unwrap().len(), 3);

    let fixed = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(
        fixed,
        "---\ntitle: Note\n---\n\n# Note\n\nBody\n\n## Usage\n\n## API\n"
    );

    let rerun = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--apply")
        .arg("--format")
        .arg("json")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(rerun.status.code(), Some(0));
    let rerun_json: serde_json::Value = serde_json::from_slice(&rerun.stdout).unwrap();
    assert_eq!(rerun_json["rule"], "all");
    assert_eq!(rerun_json["files_changed"], 0);
    assert_eq!(rerun_json["fixes_applied"], 0);
    assert_eq!(rerun_json["files_would_change"], 0);
    assert_eq!(rerun_json["fixes_would_apply"], 0);
}

#[test]
fn fix_markdown_defaults_to_preview_without_writing() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n",
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
        stdout.contains("would change 1 file(s)"),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("Run again with --apply"),
        "stdout was:\n{stdout}"
    );

    let unchanged = fs::read_to_string(project.path().join("docs/note.md")).unwrap();
    assert_eq!(unchanged, "---\ntitle: Note\n---\n   \n# Note\n");
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
        .arg("--apply")
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
fn fix_markdown_reports_partial_skips_and_leaves_non_target_files_unchanged() {
    let project = write_project_with_extra_markdown(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n   \n# Note\n",
    );
    let notes_before = fs::read_to_string(project.path().join("notes.txt")).unwrap();

    let preview = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path())
        .arg("--dry-run")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(preview.status.code(), Some(0));
    let preview_json: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(preview_json["files_checked"], 1);
    assert_eq!(preview_json["files_would_change"], 1);
    assert_eq!(preview_json["skipped_fixes"][0]["path"], "loose.md");
    assert_eq!(preview_json["skipped_fixes"][0]["reason"], "not_configured");
    assert_eq!(
        preview_json["skipped_fixes"][0]["operation"],
        "remove_blank_line_trailing_spaces"
    );

    let apply = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--apply")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(apply.status.code(), Some(0));
    let apply_json: serde_json::Value = serde_json::from_slice(&apply.stdout).unwrap();
    assert_eq!(apply_json["changed_paths"].as_array().unwrap().len(), 1);
    assert_eq!(apply_json["changed_paths"][0], "docs/note.md");
    assert_eq!(apply_json["skipped_fixes"][0]["path"], "loose.md");
    let notes_after = fs::read_to_string(project.path().join("notes.txt")).unwrap();
    assert_eq!(notes_after, notes_before);
}

#[cfg(unix)]
#[test]
fn fix_markdown_partial_write_failure_emits_audit_report() {
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
          lint_trailing_spaces: true
"#,
    )
    .unwrap();
    fs::write(project.path().join("docs/a-good.md"), "# Good\n   \n").unwrap();
    let fail_path = project.path().join("docs/z-fail.md");
    fs::write(&fail_path, "# Fail\n   \n").unwrap();
    let mut permissions = fs::metadata(&fail_path).unwrap().permissions();
    permissions.set_mode(0o444);
    fs::set_permissions(&fail_path, permissions).unwrap();

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg("--apply")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["mode"], "apply");
    assert_eq!(json["changed_paths"][0], "docs/a-good.md");
    assert_eq!(json["files_changed"], 1);
    assert_eq!(json["fixes_applied"], 1);
    assert_eq!(json["failures"][0]["path"], "docs/z-fail.md");
    assert_eq!(
        json["failures"][0]["operation"],
        "remove_blank_line_trailing_spaces"
    );
    assert_eq!(json["files"][1]["path"], "docs/z-fail.md");
    assert_eq!(json["files"][1]["status"], "failed");
    assert_eq!(json["fixes"][1]["status"], "failed");
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());

    assert_eq!(
        fs::read_to_string(project.path().join("docs/a-good.md")).unwrap(),
        "# Good\n\n"
    );
    assert_eq!(fs::read_to_string(&fail_path).unwrap(), "# Fail\n   \n");
}

#[test]
fn fix_markdown_invalid_path_fails_without_report() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "---\ntitle: Note\n---\n\n# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("fix")
        .arg("markdown")
        .arg(project.path().join("missing.md"))
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("checked path does not exist"),
        "stderr was:\n{stderr}"
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
