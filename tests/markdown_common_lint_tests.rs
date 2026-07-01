use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_project(config_markdown: &str, note: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    extra: true
    children:
      docs/:
        markdown:
{config_markdown}
exclude:
  - target/**
"#
        ),
    )
    .unwrap();
    fs::write(project.path().join("docs/note.md"), note).unwrap();
    project
}

fn check_json(project: &TempDir) -> (std::process::ExitStatus, serde_json::Value) {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    let json = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse stdout as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, json)
}

#[test]
fn markdown_common_lint_reports_heading_and_blank_line_findings() {
    let project = write_project(
        "          lint_common: true\n",
        "# Note\n\n#Title\n\n\n### Deep\n\n## Repeat\n\n## Repeat\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let rules = json["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        rules.contains(&"markdown_heading_marker_spacing"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"markdown_heading_increment"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"markdown_multiple_blank_lines"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"markdown_duplicate_heading"),
        "rules: {rules:?}"
    );
}

#[test]
fn markdown_common_lint_ignores_frontmatter_fences_and_indented_code() {
    let project = write_project(
        "          lint_common: true\n",
        "---\ntitle: Note\n# Repeat\n---\n# Repeat\n\n```markdown\n#Bad\n\n\n```\n\n    #Bad\n\n## Usage\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdown_common_lint_respects_suppressions_and_severity() {
    let project = write_project(
        "          lint_common: true\n          rules:\n            markdown_multiple_blank_lines:\n              severity: low\n",
        "# Note\n\n<!-- assura-ignore markdown_duplicate_heading: fixture duplicate -->\n## Repeat\n\n\n## Repeat\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1, "report: {json:#}");
    assert_eq!(violations[0]["rule"], "markdown_multiple_blank_lines");
    assert_eq!(violations[0]["severity"], "low");
    assert_eq!(violations[0]["blocking"], false);
}
