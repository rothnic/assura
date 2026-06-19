use std::fs;
use std::path::PathBuf;

use assura::cli::run_structure_check;
use tempfile::TempDir;

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

#[test]
fn package_doc_relationship_composes_with_markdown_outline() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
      markdown:
        outline:
          - Overview
          - ?? Configuration:
              - Install
          - Why Assura?
exclude:
  - target/**
"#,
    );
    fs::create_dir_all(project.path().join("packages/core")).unwrap();
    fs::create_dir_all(project.path().join("docs/packages")).unwrap();
    fs::write(
        project.path().join("docs/packages/core.md"),
        "# Core\n\n## Overview\n\n## Configuration\n\n### Install\n\n## Why Assura?\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn markdown_outline_allows_absent_optional_parent() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
          - ?? Configuration:
              - Install
          - Why Assura?
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Overview\n\n## Why Assura?\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn markdown_outline_allows_headingless_document_when_every_entry_is_optional() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - ?? Troubleshooting
          - ?? Configuration:
              - Install
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "Plain text body without headings.\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn markdown_outline_skips_unmatched_sibling_sections() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Intro\n\n### Context\n\n## Overview\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn markdown_outline_reports_required_child_under_present_optional_parent() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
          - ?? Configuration:
              - Install
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Overview\n\n## Configuration\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.path, PathBuf::from("guide.md"));
    assert_eq!(violation.rule, "markdown_outline");
    assert!(violation.message.contains("Install"), "{violation:#?}");
}

#[test]
fn markdown_outline_supports_question_mark_headings_and_object_escape() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Why Assura?
          - title: "?? Debug Mode"
            optional: false
          - ?? Optional?
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Why Assura?\n\n## ?? Debug Mode\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "{:#?}", report.violations);
    assert!(report.violations.is_empty());
}

#[test]
fn markdown_outline_reports_skipped_heading_level() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview:
              - Install
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Overview\n\n#### Install\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.rule, "markdown_outline");
    assert!(
        violation.message.contains("skipped heading level H4"),
        "{violation:#?}"
    );
    assert!(violation.message.contains("line 5"), "{violation:#?}");
}

#[test]
fn markdown_outline_reports_unconfigured_skipped_heading_level() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
          - Details
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Overview\n\n#### Surprise\n\n## Details\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.rule, "markdown_outline");
    assert!(
        violation.message.contains("skipped heading level H4"),
        "{violation:#?}"
    );
}

#[test]
fn markdown_outline_reports_ambiguous_root_matching() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Overview\n\n## Detail\n\n# Overview\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.rule, "markdown_outline");
    assert!(violation.message.contains("ambiguous"), "{violation:#?}");
}

#[test]
fn markdown_outline_reports_ambiguous_title_scoped_root_matching() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    guide.md:
      markdown:
        outline:
          - Overview
exclude:
  - target/**
"#,
    );
    fs::write(
        project.path().join("guide.md"),
        "# Guide\n\n## Overview\n\n## Overview\n",
    )
    .unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    let violation = &report.violations[0];
    assert_eq!(violation.rule, "markdown_outline");
    assert!(violation.message.contains("ambiguous"), "{violation:#?}");
}
