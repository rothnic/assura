use std::fs;
use std::process::Command;

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

#[test]
fn check_passes_realistic_multi_package_policy_matrix() {
    let project = TempDir::new().unwrap();
    write_config(&project, realistic_policy_matrix_config());
    write_valid_realistic_policy_project(&project);

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn check_policy_diagnostics_include_corrective_context() {
    let project = TempDir::new().unwrap();
    write_config(&project, realistic_policy_matrix_config());
    write_valid_realistic_policy_project(&project);

    fs::remove_file(project.path().join("AGENTS.md")).unwrap();
    fs::remove_dir_all(project.path().join("package-core")).unwrap();
    fs::write(project.path().join("scratch.txt"), "temporary\n").unwrap();
    fs::write(project.path().join("draft-plan.md"), "# Draft\n").unwrap();
    fs::create_dir(project.path().join("tmp-cache")).unwrap();
    fs::create_dir(project.path().join("scratch")).unwrap();
    fs::create_dir(project.path().join("src/bad-dir")).unwrap();
    fs::write(
        project.path().join("src/BadName.rs"),
        "//! Bad name fixture.\n\npub fn bad_name() {}\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/decision.md"),
        "# Decision\n\n## Details\n",
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let violations = report["violations"].as_array().unwrap();
    let rules = violations
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    for expected in [
        "required_file",
        "unexpected_file",
        "forbidden_file",
        "file_naming",
        "unexpected_directory",
        "forbidden_directory",
        "directory_naming",
        "exists_count",
        "markdown_frontmatter",
        "markdown_frontmatter_field",
        "markdown_required_section",
    ] {
        assert!(
            rules.contains(expected),
            "missing {expected}; report was:\n{report:#}"
        );
    }

    for violation in violations {
        assert!(
            violation["path"].as_str().is_some(),
            "violation path should be stable: {violation:#}"
        );
        assert!(
            violation["rule"]
                .as_str()
                .is_some_and(|rule| !rule.is_empty()),
            "violation rule should be stable: {violation:#}"
        );
        assert!(
            violation["severity"]
                .as_str()
                .is_some_and(|severity| !severity.is_empty()),
            "violation severity should be present: {violation:#}"
        );
        assert!(
            violation["message"]
                .as_str()
                .is_some_and(|message| !message.is_empty()),
            "violation message should be present: {violation:#}"
        );
        assert!(
            violation["corrective_context"]
                .as_str()
                .is_some_and(|context| context.contains(".assura/config.yml")
                    || context.contains("files.")
                    || context.contains("directories.")
                    || context.contains("markdown.")),
            "violation corrective context should point to a policy fix: {violation:#}"
        );
    }
}

#[test]
fn converted_lslint_dir_rule_diagnostic_mentions_self_directory_context() {
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
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let violations = report["violations"].as_array().unwrap();
    let directory_naming = violations
        .iter()
        .find(|violation| violation["rule"] == "directory_naming")
        .unwrap_or_else(|| panic!("missing directory_naming; report was:\n{report:#}"));

    assert_eq!(directory_naming["path"], "src/bad_dir");
    assert!(
        directory_naming["corrective_context"]
            .as_str()
            .is_some_and(|context| context.contains("self_directory.naming")),
        "directory naming context should mention self_directory.naming: {directory_naming:#}"
    );
}

fn realistic_policy_matrix_config() -> &'static str {
    r#"
structure:
  ./:
    files:
      required:
        - README.md
        - AGENTS.md
      allowed_names:
        - README.md
        - AGENTS.md
        - Cargo.toml
      allowed_patterns:
        - "*.lock"
      forbidden_patterns:
        - "draft-*"
      allow_extra: false
      exists:
        "*.rs": "0"
      severity: high
    directories:
      required:
        - src
        - docs
      allowed_names:
        - src
        - docs
      allowed_patterns:
        - "package-*"
      forbidden_patterns:
        - "tmp-*"
      allow_extra: false
      exists:
        "package-*": "1-2"
      severity: critical
    exists:
      files:
        - Cargo.toml
      directories:
        - src
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
      src/:
        files:
          naming: snake_case
          max_lines: 20
          max_size: 2KB
          require_docs: true
          extensions:
            - rs
        directories:
          naming: snake_case
      docs/:
        files:
          naming_patterns:
            "*.md": kebab-case
        markdown:
          require_frontmatter: true
          required_fields:
            - title
          max_heading_depth: 2
          required_sections:
            - Summary
      package-core/:
        files:
          allowed_names:
            - README.md
          exists:
            "README.md": "1"
          allow_extra: false
        markdown:
          require_frontmatter: false
exclude:
  - "generated/**"
"#
}

fn write_valid_realistic_policy_project(project: &TempDir) {
    fs::write(project.path().join("README.md"), "# Project\n").unwrap();
    fs::write(project.path().join("AGENTS.md"), "# Agents\n").unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();
    fs::write(project.path().join("Cargo.lock"), "# lock\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "//! Demo library.\n\npub fn demo() {}\n",
    )
    .unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/decision.md"),
        "---\ntitle: Decision\n---\n# Decision\n\n## Summary\nAccepted.\n",
    )
    .unwrap();
    fs::create_dir(project.path().join("package-core")).unwrap();
    fs::write(project.path().join("package-core/README.md"), "# Core\n").unwrap();
    fs::create_dir_all(project.path().join("generated/tmp")).unwrap();
    fs::write(project.path().join("generated/tmp/BAD.TMP"), "ignored\n").unwrap();
}
