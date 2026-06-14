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

fn has_violation(output: &std::process::Output, rule: &str, path: &str) -> bool {
    report(output)["violations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|violation| violation["rule"] == rule && violation["path"] == path)
}

fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn native_config() -> &'static str {
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
fn check_accepts_native_config_and_rejects_extra_direct_files() {
    let project = TempDir::new().unwrap();
    write_config(&project, native_config());

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
        "expected Assura config fixture to pass:\nstdout:\n{}\nstderr:\n{}",
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
fn check_reports_missing_required_and_forbidden_native_files() {
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
fn check_combined_extension_exists_and_naming_allows_valid_matches() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: false
    .md:
      exists: 1
      naming: kebab-case
exclude:
  - ".assura/**"
"#,
    );

    fs::write(project.path().join("guide.md"), "# Guide\n").unwrap();
    assert!(check(&project).status.success());

    fs::remove_file(project.path().join("guide.md")).unwrap();
    fs::write(project.path().join("BadGuide.md"), "# Guide\n").unwrap();
    let bad_name = check(&project);
    assert!(!bad_name.status.success());
    assert!(has_rule(&bad_name, "file_naming"));
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
fn check_package_fragment_requires_multiple_files_per_package() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
rules:
  package-standard:
    README.md: exists:1
    AGENTS.md: exists:1
    package.json: exists:1
    src/: exists:1
structure:
  packages/*/:
    use: "@package-standard"
exclude:
  - ".assura/**"
"#,
    );

    let core = project.path().join("packages/core");
    fs::create_dir_all(core.join("src")).unwrap();
    fs::write(core.join("README.md"), "# Core\n").unwrap();
    fs::write(core.join("AGENTS.md"), "# Guidance\n").unwrap();
    fs::write(core.join("package.json"), "{}\n").unwrap();
    assert!(check(&project).status.success());

    let ui = project.path().join("packages/ui");
    fs::create_dir_all(ui.join("src")).unwrap();
    fs::write(ui.join("README.md"), "# UI\n").unwrap();
    fs::write(ui.join("package.json"), "{}\n").unwrap();
    let missing_agents = check(&project);
    assert!(!missing_agents.status.success());
    assert!(has_rule(&missing_agents, "exists_count"));
}

#[test]
fn check_nested_wildcard_package_scope_matches_existing_packages() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: false
    README.md: exists:1
    AGENTS.md: exists:1
    package.json: exists:1
    packages/:
      extra: false
      "*/":
        extra: false
        package.json: exists:1
        src/:
          .ts: kebab-case
exclude:
  - ".assura/**"
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
"#,
    );

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    fs::write(project.path().join("AGENTS.md"), "# Guidance\n").unwrap();
    fs::write(project.path().join("package.json"), "{}\n").unwrap();

    let core_src = project.path().join("packages/core/src");
    let ui_src = project.path().join("packages/ui/src");
    fs::create_dir_all(&core_src).unwrap();
    fs::create_dir_all(&ui_src).unwrap();
    fs::write(project.path().join("packages/core/package.json"), "{}\n").unwrap();
    fs::write(project.path().join("packages/ui/package.json"), "{}\n").unwrap();
    fs::write(core_src.join("index.ts"), "export {};\n").unwrap();
    fs::write(ui_src.join("button.ts"), "export {};\n").unwrap();

    assert!(check(&project).status.success());

    fs::remove_file(project.path().join("packages/ui/package.json")).unwrap();
    let missing_package_json = check(&project);
    assert!(!missing_package_json.status.success());
    assert!(has_rule(&missing_package_json, "exists_count"));

    fs::write(project.path().join("packages/ui/package.json"), "{}\n").unwrap();
    fs::write(ui_src.join("BadName.ts"), "export {};\n").unwrap();
    let bad_source_name = check(&project);
    assert!(!bad_source_name.status.success());
    assert!(has_rule(&bad_source_name, "file_naming"));

    fs::remove_file(ui_src.join("BadName.ts")).unwrap();
    fs::create_dir(project.path().join("packages/core/tmp")).unwrap();
    let extra_package_dir = check(&project);
    assert!(!extra_package_dir.status.success());
    assert!(has_violation(
        &extra_package_dir,
        "unexpected_directory",
        "packages/core/tmp"
    ));
}

#[test]
fn check_required_wildcard_scope_fails_when_no_directories_match() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  packages/*/:
    required: true
    package.json: exists:1
exclude:
  - ".assura/**"
"#,
    );

    let missing_packages = check(&project);
    assert!(!missing_packages.status.success());
    assert!(has_violation(
        &missing_packages,
        "required_directory",
        "packages/*"
    ));
}

#[test]
fn check_nested_required_wildcard_scope_fails_when_no_children_match() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    packages/:
      "*/":
        required: true
        package.json: exists:1
exclude:
  - ".assura/**"
"#,
    );

    let missing_parent = check(&project);
    assert!(!missing_parent.status.success());
    assert!(has_violation(
        &missing_parent,
        "required_directory",
        "packages"
    ));

    fs::create_dir(project.path().join("packages")).unwrap();
    let missing_package = check(&project);
    assert!(!missing_package.status.success());
    assert!(has_violation(
        &missing_package,
        "required_directory",
        "packages/*"
    ));

    fs::create_dir(project.path().join("packages/core")).unwrap();
    let missing_package_json = check(&project);
    assert!(!missing_package_json.status.success());
    assert!(has_violation(
        &missing_package_json,
        "exists_count",
        "packages/core"
    ));
}

#[test]
fn check_dir_rule_validates_matched_directory_names() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  packages/*/:
    .dir: kebab-case
exclude:
  - ".assura/**"
"#,
    );

    fs::create_dir_all(project.path().join("packages/core-lib")).unwrap();
    assert!(check(&project).status.success());

    fs::create_dir_all(project.path().join("packages/CoreLib")).unwrap();
    let bad_dir = check(&project);
    assert!(!bad_dir.status.success());
    assert!(has_rule(&bad_dir, "directory_naming"));
}

#[test]
fn check_rejects_removed_native_exists_shortcuts() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    README.md: exists
exclude:
  - ".assura/**"
"#,
    );
    let bare_exists = check(&project);
    assert!(!bare_exists.status.success());
    assert!(output_text(&bare_exists).contains("exists shorthand must include cardinality"));

    write_config(
        &project,
        r#"
structure:
  ./:
    README.md: 1
exclude:
  - ".assura/**"
"#,
    );
    let numeric_node = check(&project);
    assert!(!numeric_node.status.success());
    assert!(output_text(&numeric_node).contains("node directive numbers are not supported"));
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
