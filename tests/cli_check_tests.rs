use std::fs;
use std::process::Command;

use assura::cli::run_structure_check;
use assura::config::ls_compat::convert_ls_lint_to_config;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn write_generated_config(project: &TempDir, config: &assura::config::config::Config) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(
        assura_dir.join("config.yml"),
        serde_yaml::to_string(config).unwrap(),
    )
    .unwrap();
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
fn check_prunes_excluded_directories_before_validation() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - generated/**
"#,
    );

    fs::create_dir(project.path().join("generated")).unwrap();
    fs::create_dir(project.path().join("generated/BadDir")).unwrap();
    fs::write(project.path().join("generated/BadName.rs"), "").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/good-file.rs"), "").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "report was: {report:#?}");
    assert_eq!(report.violations.len(), 0);
    assert_eq!(
        report.dirs_checked, 2,
        ".assura and src should be checked while generated descendants are pruned"
    );
}

#[test]
fn check_reports_deterministically_sorted_violations() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
"#,
    );

    fs::create_dir(project.path().join("z-dir")).unwrap();
    fs::create_dir(project.path().join("a-dir")).unwrap();
    fs::write(project.path().join("z-dir/BadName.rs"), "").unwrap();
    fs::write(project.path().join("a-dir/BadName.rs"), "").unwrap();
    fs::write(project.path().join("BadRoot.rs"), "").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    let pairs: Vec<_> = report
        .violations
        .iter()
        .map(|violation| {
            (
                violation.path.to_string_lossy().to_string(),
                violation.rule.clone(),
            )
        })
        .collect();
    let mut sorted = pairs.clone();
    sorted.sort();

    assert_eq!(pairs, sorted);
}

#[test]
fn check_fail_fast_stops_after_first_sorted_traversal_violation() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
"#,
    );

    fs::create_dir(project.path().join("a-dir")).unwrap();
    fs::create_dir(project.path().join("z-dir")).unwrap();
    fs::write(project.path().join("a-dir/BadName.rs"), "").unwrap();
    fs::write(project.path().join("z-dir/BadName.rs"), "").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, true).unwrap();

    assert!(!report.success);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(
        report.violations[0].path,
        std::path::PathBuf::from("a-dir/BadName.rs")
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

#[test]
fn check_supports_regex_naming_conventions() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      required:
        - README.md
    children:
      specs/:
        files:
          naming: "regex:^[0-9]{3}-[a-z0-9-]+$"
"#,
    );

    fs::create_dir(project.path().join("specs")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("specs/001-good-name.md"), "# Good\n").unwrap();
    fs::write(project.path().join("specs/bad-name.md"), "# Bad\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file_naming"), "stdout was:\n{}", stdout);
    assert!(stdout.contains("bad-name.md"), "stdout was:\n{}", stdout);
    assert!(
        !stdout.contains("001-good-name.md"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_rejects_unexpected_direct_files_when_closed() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
      allow_extra: false
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("notes.md"), "# Surprise\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unexpected_file"),
        "stdout was:\n{}",
        stdout
    );
    assert!(stdout.contains("notes.md"), "stdout was:\n{}", stdout);
}

#[test]
fn check_allows_direct_files_by_pattern_and_rejects_forbidden_patterns() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
      allowed_patterns:
        - "*.md"
      forbidden_patterns:
        - "draft-*"
      allow_extra: false
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("notes.md"), "# Allowed\n").unwrap();
    fs::write(project.path().join("draft-plan.md"), "# Forbidden\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forbidden_file"), "stdout was:\n{}", stdout);
    assert!(stdout.contains("draft-plan.md"), "stdout was:\n{}", stdout);
    assert!(!stdout.contains("notes.md"), "stdout was:\n{}", stdout);
}

#[test]
fn check_rejects_unexpected_direct_directories_when_closed() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
    directories:
      allowed_names:
        - src
      allow_extra: false
    children:
      .assura/:
        files:
          naming: kebab-case
      src/:
        files:
          naming: snake_case
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("scratch")).unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unexpected_directory"),
        "stdout was:\n{}",
        stdout
    );
    assert!(stdout.contains("scratch"), "stdout was:\n{}", stdout);
}

#[test]
fn check_does_not_duplicate_naming_for_unexpected_directories() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    directories:
      naming: kebab-case
      allowed_names:
        - src
      allow_extra: false
    children:
      .assura/:
        files:
          naming: kebab-case
      src/:
        files:
          naming: snake_case
"#,
    );

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("bad_dir")).unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("unexpected_directory"),
        "stdout was:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("directory_naming"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_validates_file_and_directory_exists_counts() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - README.md
      exists:
        "*.rs": "1"
    directories:
      exists:
        "tmp-*": "0"
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir(project.path().join("tmp-work")).unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("exists_count"), "stdout was:\n{}", stdout);
    assert!(stdout.contains("tmp-*"), "stdout was:\n{}", stdout);
}

#[test]
fn check_supports_wildcard_extension_rules() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      extensions:
        - ".*"
      allow_extra: false
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("LICENSE"), "MIT\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("All configured structure checks passed"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_supports_multi_part_extension_rules_without_leading_dot() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      extensions:
        - "tar.gz"
      allow_extra: false
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("archive.tar.gz"), "example\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("All configured structure checks passed"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_supports_regex_naming_with_pipe_and_or_rule() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: "regex:^(foo|bar)$ | kebab-case"
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("foo.rs"), "fn main() {}\n").unwrap();
    fs::write(project.path().join("good-name.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("All configured structure checks passed"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_markdown_headings_support_indentation_and_ignore_fences() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      allowed_names:
        - doc.md
    markdown:
      max_heading_depth: 2
      required_sections:
        - Indented
        - Deep
    children:
      .assura/:
        files:
          naming: kebab-case
"#,
    );

    fs::write(
        project.path().join("doc.md"),
        "# Title\n\n```md\n### Ignored\n```\n\n  ##   Indented  \n\n### Deep\n",
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
        stdout.contains("markdown_heading_depth"),
        "stdout was:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("markdown_required_section"),
        "stdout was:\n{}",
        stdout
    );
}

#[test]
fn check_validates_converted_ls_lint_dir_rules() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  src:
    .dir: kebab-case
    .rs: snake_case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("src/bad_dir")).unwrap();
    fs::write(project.path().join("src/bad_dir/main.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("directory_naming"),
        "stdout was:\n{}",
        stdout
    );
    assert!(stdout.contains("bad_dir"), "stdout was:\n{}", stdout);
}
