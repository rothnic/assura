use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_project(note: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  ./:
    extra: true
    children:
      docs/:
        markdown:
          check_links: true
exclude:
  - target/**
"#,
    )
    .unwrap();
    fs::write(project.path().join("docs/note.md"), note).unwrap();
    fs::write(
        project.path().join("docs/target.md"),
        "# Target Doc\n\n## Install Steps\n\nBody\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();
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
    let json = serde_json::from_slice(&output.stdout).unwrap();
    (output.status, json)
}

#[test]
fn markdown_link_check_accepts_relative_files_headings_and_line_ranges() {
    let project = write_project(
        "# Note\n\nSee [target](target.md), [heading](target.md#install-steps), and [code](../src/lib.rs#L1-L2).\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdown_link_check_reports_missing_files_headings_and_lines() {
    let project = write_project(
        "# Note\n\nSee [missing](missing.md), [bad heading](target.md#missing-heading), and [bad line](../src/lib.rs#L9).\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let rules = json["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(rules.contains(&"markdown_link_target"), "rules: {rules:?}");
    assert!(
        rules.contains(&"markdown_link_heading_anchor"),
        "rules: {rules:?}"
    );
    assert!(
        rules.contains(&"markdown_link_line_anchor"),
        "rules: {rules:?}"
    );
}

#[test]
fn markdown_link_check_reports_root_absolute_internal_links() {
    let project = write_project("# Note\n\nSee [target](/docs/target.md).\n");

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    assert_eq!(json["violations"][0]["rule"], "markdown_link_format");
    assert!(json["violations"][0]["message"]
        .as_str()
        .unwrap()
        .contains("non-relative internal link"));
}

#[test]
fn markdown_link_check_ignores_inline_code_examples_and_images() {
    let project = write_project(
        "# Note\n\nUse `[missing](missing.md)` as a literal example.\n\n![alt text](missing.png)\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdown_link_check_uses_rendered_heading_text_for_anchor_slugs() {
    let project = write_project(
        "# Note\n\nSee [install](target.md#install-steps) and [code](target.md#api-name).\n",
    );
    fs::write(
        project.path().join("docs/target.md"),
        "# Target Doc\n\n## [Install Steps](install.md)\n\n## `API` Name\n",
    )
    .unwrap();
    fs::write(project.path().join("docs/install.md"), "# Install\n").unwrap();

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}
