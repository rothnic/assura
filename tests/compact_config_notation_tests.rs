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

fn check(project: &TempDir) -> std::process::Output {
    Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap()
}

fn report(output: &std::process::Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn has_rule(output: &std::process::Output, rule: &str) -> bool {
    report(output)["violations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|violation| violation["rule"] == rule)
}

fn compact_config() -> &'static str {
    r#"
rules:
  readme:
    exists: 1
  agents:
    exists: 1
  project-docs:
    README.md: "@readme"
    AGENTS.md: "@agents"

structure:
  ./:
    extra: false
    use: "@project-docs"
    Cargo.toml: exists:1
    guide.md: exists:0-1
    .md: kebab-case
    src/: exists:1
    docs/: exists:0-1

exclude:
  - ".assura/**"
"#
}

#[test]
fn check_accepts_compact_config_and_rejects_extra_direct_files() {
    let project = TempDir::new().unwrap();
    write_config(&project, compact_config());

    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("AGENTS.md"), "# Project Guidance\n").unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"example\"\n",
    )
    .unwrap();
    fs::write(project.path().join("guide.md"), "# Guide\n").unwrap();

    let pass = check(&project);

    assert!(
        pass.status.success(),
        "expected compact config fixture to pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&pass.stdout),
        String::from_utf8_lossy(&pass.stderr)
    );

    fs::write(project.path().join("scratch.txt"), "temporary\n").unwrap();
    let fail = check(&project);

    assert!(
        !fail.status.success(),
        "expected extra file to fail:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&fail.stdout),
        String::from_utf8_lossy(&fail.stderr)
    );
    assert!(
        has_rule(&fail, "unexpected_file"),
        "expected unexpected_file violation:\n{}",
        String::from_utf8_lossy(&fail.stdout)
    );
}

#[test]
fn check_reports_missing_required_and_forbidden_compact_files() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    README.md: exists:1
    forbidden.log: exists:0
exclude:
  - ".assura/**"
"#,
    );

    let missing = check(&project);
    assert!(!missing.status.success());
    assert!(has_rule(&missing, "exists_count"));

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("forbidden.log"), "debug\n").unwrap();
    let forbidden = check(&project);
    assert!(!forbidden.status.success());
    assert!(has_rule(&forbidden, "exists_count"));
}

#[test]
fn check_validates_bounded_extension_counts() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    .md: exists:1-2
exclude:
  - ".assura/**"
"#,
    );

    fs::write(project.path().join("one.md"), "# One\n").unwrap();
    assert!(check(&project).status.success());

    fs::write(project.path().join("two.md"), "# Two\n").unwrap();
    assert!(check(&project).status.success());

    fs::write(project.path().join("three.md"), "# Three\n").unwrap();
    let too_many = check(&project);
    assert!(!too_many.status.success());
    assert!(has_rule(&too_many, "exists_count"));
}

#[test]
fn check_optional_pattern_scope_does_not_require_missing_matches() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  packages/*/:
    README.md: exists:1
exclude:
  - ".assura/**"
"#,
    );

    assert!(check(&project).status.success());

    fs::create_dir(project.path().join("packages")).unwrap();
    fs::create_dir(project.path().join("packages/core")).unwrap();
    let missing_nested_readme = check(&project);
    assert!(!missing_nested_readme.status.success());
    assert!(has_rule(&missing_nested_readme, "exists_count"));
}

#[test]
fn check_treats_dotfile_path_keys_as_exact_files() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    .gitignore: exists:1
exclude:
  - ".assura/**"
"#,
    );

    fs::write(project.path().join("foo.gitignore"), "target\n").unwrap();
    let missing_exact_dotfile = check(&project);
    assert!(!missing_exact_dotfile.status.success());
    assert!(has_rule(&missing_exact_dotfile, "exists_count"));

    fs::write(project.path().join(".gitignore"), "target\n").unwrap();
    assert!(check(&project).status.success());
}

#[test]
fn check_treats_long_dot_keys_as_extension_rules() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    .graphql: kebab-case
exclude:
  - ".assura/**"
"#,
    );

    fs::write(project.path().join("valid-schema.graphql"), "type Query\n").unwrap();
    assert!(check(&project).status.success());

    fs::write(project.path().join("InvalidSchema.graphql"), "type Query\n").unwrap();
    let bad_name = check(&project);
    assert!(!bad_name.status.success());
    assert!(has_rule(&bad_name, "file_naming"));
}
