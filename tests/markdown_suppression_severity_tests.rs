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

fn write_project_config(config: &str, note: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::write(project.path().join(".assura/config.yml"), config).unwrap();
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
fn markdown_rule_severity_can_make_link_findings_advisory() {
    let project = write_project(
        "          check_links: true\n          rule_severity:\n            markdown_link_target: low\n",
        "# Note\n\nSee [missing](missing.md).\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["success"], true);
    let violation = &json["violations"][0];
    assert_eq!(violation["rule"], "markdown_link_target");
    assert_eq!(violation["severity"], "low");
    assert_eq!(violation["blocking"], false);
}

#[test]
fn markdown_rule_severity_merges_parent_and_child_maps() {
    let project = write_project_config(
        r#"
structure:
  ./:
    extra: true
    markdown:
      check_links: true
      rule_severity:
        markdown_link_target: low
    children:
      docs/:
        markdown:
          rule_severity:
            markdown_heading_depth: high
exclude:
  - target/**
"#,
        "# Note\n\nSee [missing](missing.md).\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    let violation = &json["violations"][0];
    assert_eq!(violation["rule"], "markdown_link_target");
    assert_eq!(violation["severity"], "low");
    assert_eq!(violation["blocking"], false);
}

#[test]
fn markdown_assura_ignore_suppresses_matching_rule_when_reasoned() {
    let project = write_project(
        "          check_links: true\n",
        "# Note\n\n<!-- assura-ignore markdown_link_target: generated fixture intentionally points at future docs -->\nSee [missing](missing.md).\n",
    );

    let (status, json) = check_json(&project);

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdown_assura_ignore_suppresses_only_one_following_matching_finding() {
    let project = write_project(
        "          check_links: true\n",
        "# Note\n\n<!-- assura-ignore markdown_link_target: generated fixture intentionally points at future docs -->\nSee [first](first.md).\nSee [second](second.md).\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1, "report: {json:#}");
    assert_eq!(violations[0]["rule"], "markdown_link_target");
    assert!(violations[0]["message"]
        .as_str()
        .unwrap()
        .contains("second.md"));
}

#[test]
fn markdown_assura_ignore_does_not_suppress_prior_same_line_finding() {
    let project = write_project(
        "          check_links: true\n",
        "# Note\n\nSee [missing](missing.md). <!-- assura-ignore markdown_link_target: generated fixture intentionally points at future docs -->\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1, "report: {json:#}");
    assert_eq!(violations[0]["rule"], "markdown_link_target");
}

#[test]
fn markdown_assura_ignore_reports_reasonless_suppression() {
    let project = write_project(
        "          check_links: true\n",
        "# Note\n\n<!-- assura-ignore markdown_link_target: -->\nSee [missing](missing.md).\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let rules = json["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(rules.contains(&"markdown_suppression"), "rules: {rules:?}");
    assert!(rules.contains(&"markdown_link_target"), "rules: {rules:?}");
}

#[test]
fn markdown_assura_ignore_reports_unknown_rule() {
    let project = write_project(
        "          lint_trailing_spaces: true\n",
        "# Note\n\n<!-- assura-ignore markdown_future_rule: generated fixture -->\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    assert_eq!(json["violations"][0]["rule"], "markdown_suppression");
    assert!(json["violations"][0]["message"]
        .as_str()
        .unwrap()
        .contains("supported markdown_* rule"));
}

#[test]
fn markdown_assura_ignore_ignores_prose_and_fenced_examples() {
    let project = write_project(
        "          check_links: true\n",
        "# Note\n\nUse assura-ignore markdown_link_target: reason as prose.\n\n```markdown\n<!-- assura-ignore markdown_link_target: example only -->\n```\n\nSee [missing](missing.md).\n",
    );

    let (status, json) = check_json(&project);

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    let violations = json["violations"].as_array().unwrap();
    assert_eq!(violations.len(), 1, "report: {json:#}");
    assert_eq!(violations[0]["rule"], "markdown_link_target");
}

#[test]
fn markdown_rule_severity_rejects_unknown_rule_ids() {
    let project = write_project(
        "          rule_severity:\n            markdown_future_rule: low\n",
        "# Note\n",
    );

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("expected a supported markdown_* rule id"),
        "stderr:\n{stderr}"
    );
}
