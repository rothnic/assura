use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use assura::config::ls_compat::{convert_ls_lint_documents_to_config, convert_ls_lint_to_config};
use tempfile::TempDir;

static LS_LINT_BINARY: OnceLock<PathBuf> = OnceLock::new();

fn native_ls_lint_binary() -> &'static Path {
    LS_LINT_BINARY
        .get_or_init(|| {
            let install_dir = std::env::temp_dir().join(format!(
                "assura_lslint_rule_coverage_{}",
                std::process::id()
            ));
            fs::create_dir_all(&install_dir).unwrap();
            let status = Command::new("npm")
                .current_dir(&install_dir)
                .args([
                    "install",
                    "--silent",
                    "--no-audit",
                    "--no-fund",
                    "@ls-lint/ls-lint@2.3.1",
                ])
                .status()
                .expect("npm install for native LS-Lint should run");
            assert!(status.success(), "npm install for native LS-Lint failed");

            let binary = install_dir
                .join("node_modules")
                .join("@ls-lint")
                .join("ls-lint")
                .join("bin")
                .join(native_ls_lint_binary_name());
            assert!(binary.exists(), "missing native LS-Lint binary: {binary:?}");
            binary
        })
        .as_path()
}

fn native_ls_lint_binary_name() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "ls-lint-darwin-amd64",
        ("macos", "aarch64") => "ls-lint-darwin-arm64",
        ("linux", "x86_64") => "ls-lint-linux-amd64",
        ("linux", "aarch64") => "ls-lint-linux-arm64",
        ("linux", "s390x") => "ls-lint-linux-s390x",
        ("linux", "powerpc64") => "ls-lint-linux-ppc64le",
        ("windows", "x86_64") => "ls-lint-windows-amd64.exe",
        other => panic!("unsupported native LS-Lint platform: {other:?}"),
    }
}

fn run_native_ls_lint(project: &TempDir) -> (bool, Vec<String>) {
    let output = Command::new(native_ls_lint_binary())
        .current_dir(project.path())
        .args(["--error-output-format", "json"])
        .output()
        .expect("native LS-Lint should run");
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json_output = if stdout.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };
    let paths = if json_output.is_empty() {
        Vec::new()
    } else {
        let value: serde_json::Value = serde_json::from_str(json_output).unwrap_or_else(|error| {
            panic!(
                "failed to parse native LS-Lint JSON: {error}\nstdout:\n{}\nstderr:\n{}",
                stdout, stderr
            )
        });
        let mut paths = value
            .as_object()
            .unwrap_or_else(|| panic!("native LS-Lint JSON should be an object: {value:#}"))
            .keys()
            .map(|path| {
                if path == "." {
                    String::new()
                } else {
                    path.clone()
                }
            })
            .collect::<Vec<_>>();
        paths.sort();
        paths
    };
    (success, paths)
}

fn run_assura_paths(project: &TempDir) -> (bool, Vec<String>) {
    let report = super::run_json_check(project);
    paths_from_assura_report(&report)
}

fn paths_from_assura_report(report: &serde_json::Value) -> (bool, Vec<String>) {
    let mut paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    paths.sort();
    (report["success"].as_bool().unwrap(), paths)
}

fn write_lslint_config(project: &TempDir, config: &str) {
    fs::write(project.path().join(".ls-lint.yml"), config).unwrap();
    let assura_config = convert_ls_lint_to_config(config).unwrap();
    super::write_generated_config(project, &assura_config);
}

fn assert_native_parity(
    name: &str,
    config: &str,
    populate: impl FnOnce(&Path),
    expected_success: bool,
    expected_paths: &[&str],
) {
    let project = TempDir::new().unwrap();
    write_lslint_config(&project, config);
    populate(project.path());

    let (native_success, native_paths) = run_native_ls_lint(&project);
    let (assura_success, assura_paths) = run_assura_paths(&project);
    let expected_paths = expected_paths
        .iter()
        .map(|path| path.to_string())
        .collect::<Vec<_>>();

    assert_eq!(
        native_success, expected_success,
        "{name}: native LS-Lint status mismatch; paths={native_paths:?}"
    );
    assert_eq!(
        assura_success, expected_success,
        "{name}: Assura status mismatch; paths={assura_paths:?}"
    );
    assert_eq!(
        native_paths, expected_paths,
        "{name}: native LS-Lint paths mismatch"
    );
    assert_eq!(
        assura_paths, expected_paths,
        "{name}: Assura paths mismatch"
    );
}

fn run_native_ls_lint_target(project: &TempDir, target: &str) -> (bool, Vec<String>) {
    let output = Command::new(native_ls_lint_binary())
        .current_dir(project.path())
        .arg("--error-output-format")
        .arg("json")
        .arg(target)
        .output()
        .expect("native LS-Lint should run");
    let success = output.status.success();
    let output = if output.stdout.is_empty() {
        output.stderr.as_slice()
    } else {
        output.stdout.as_slice()
    };
    let mut paths = if output.is_empty() {
        Vec::new()
    } else {
        let native: serde_json::Value =
            serde_json::from_slice(output).expect("native output parses");
        native
            .as_object()
            .unwrap()
            .keys()
            .map(|path| {
                if path == "." {
                    String::new()
                } else {
                    path.clone()
                }
            })
            .collect::<Vec<_>>()
    };
    paths.sort();
    (success, paths)
}

fn run_assura_lslint_target(project: &TempDir, target: &str) -> (bool, Vec<String>) {
    let report = super::run_json_check_path(
        project,
        &project.path().join(target),
        &["--ls-lint-target-semantics"],
    );
    paths_from_assura_report(&report)
}

#[test]
fn native_lslint_golden_extension_subextension_and_wildcard_rules_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .*: snake_case
  .jsx: kebab-case
"#;
    assert_native_parity(
        "extension family",
        config,
        |root| {
            fs::write(root.join("read_me.md"), "").unwrap();
            fs::write(root.join("good-name.jsx"), "").unwrap();
            fs::write(root.join("badName.md"), "").unwrap();
            fs::write(root.join("bad_name.jsx"), "").unwrap();
        },
        false,
        &["badName.md", "bad_name.jsx"],
    );
}

#[test]
fn native_lslint_golden_subextension_rules_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .test.js: PascalCase
  .d.ts: PascalCase
"#;
    assert_native_parity(
        "subextension family",
        config,
        |root| {
            fs::write(root.join("GoodWidget.test.js"), "").unwrap();
            fs::write(root.join("GoodTypes.d.ts"), "").unwrap();
            fs::write(root.join("widget.test.js"), "").unwrap();
            fs::write(root.join("badtypes.d.ts"), "").unwrap();
        },
        false,
        &["badtypes.d.ts", "widget.test.js"],
    );
}

#[test]
fn native_lslint_golden_directory_scopes_globs_braces_and_dir_rules_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .dir: kebab-case
  packages/*/{src,tests}:
    .ts: camelCase | PascalCase
    .dir: kebab-case
  packages/*:
    "*":
      .js: kebab-case
  BadDir:
    .dir: kebab-case
"#;
    assert_native_parity(
        "directory scope family",
        config,
        |root| {
            fs::create_dir_all(root.join("packages/core/src")).unwrap();
            fs::create_dir_all(root.join("packages/core/tests")).unwrap();
            fs::create_dir_all(root.join("packages/core/nested")).unwrap();
            fs::create_dir_all(root.join("BadDir")).unwrap();
            fs::write(root.join("packages/core/src/goodName.ts"), "").unwrap();
            fs::write(root.join("packages/core/src/BadName.ts"), "").unwrap();
            fs::write(root.join("packages/core/nested/good-name.js"), "").unwrap();
            fs::write(root.join("packages/core/nested/bad_name.js"), "").unwrap();
        },
        false,
        &["BadDir", "packages/core/nested/bad_name.js"],
    );
}

#[test]
fn native_lslint_golden_regex_negation_substitutions_and_exists_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  components/*:
    .ts: regex:${0}
    .js: regex:![0-9]+
    tests:
      .*: exists:0
      .test.ts: regex:${1} | exists:1
  docs:
    .md: regex:README|AGENTS
  no-logs:
    .log: exists:0
  single:
    .ts: exists:1
  range:
    .md: exists:1-2
"#;
    assert_native_parity(
        "regex and exists family",
        config,
        |root| {
            fs::create_dir_all(root.join("components/button/tests")).unwrap();
            fs::create_dir_all(root.join("docs")).unwrap();
            fs::create_dir_all(root.join("no-logs")).unwrap();
            fs::create_dir_all(root.join("single")).unwrap();
            fs::create_dir_all(root.join("range")).unwrap();
            fs::write(root.join("components/button/button.ts"), "").unwrap();
            fs::write(root.join("components/button/wrong.ts"), "").unwrap();
            fs::write(root.join("components/button/name.js"), "").unwrap();
            fs::write(root.join("components/button/123.js"), "").unwrap();
            fs::write(root.join("components/button/tests/button.test.ts"), "").unwrap();
            fs::write(root.join("components/button/tests/extra.md"), "").unwrap();
            fs::write(root.join("docs/README.md"), "").unwrap();
            fs::write(root.join("docs/bad.md"), "").unwrap();
            fs::write(root.join("no-logs/debug.log"), "").unwrap();
            fs::write(root.join("range/one.md"), "").unwrap();
            fs::write(root.join("range/two.md"), "").unwrap();
            fs::write(root.join("range/three.md"), "").unwrap();
        },
        false,
        &[
            "components/button/123.js",
            "components/button/tests",
            "components/button/wrong.ts",
            "docs/bad.md",
            "no-logs",
            "range",
            "single",
        ],
    );
}

#[test]
fn native_lslint_golden_multi_config_merge_and_ignore_match_assura() {
    let project = TempDir::new().unwrap();
    let first = r#"
ignore:
  - ignored/**
ls:
  .js: camelcase
  .ts: snakecase
"#;
    let second = r#"
ignore:
  - generated/**
ls:
  .ts: kebabcase
"#;
    let first_path = project.path().join("base.yml");
    let second_path = project.path().join("override.yml");
    fs::write(&first_path, first).unwrap();
    fs::write(&second_path, second).unwrap();
    let assura_config = convert_ls_lint_documents_to_config(&[first, second]).unwrap();
    super::write_generated_config(&project, &assura_config);

    fs::write(project.path().join("goodName.js"), "").unwrap();
    fs::write(project.path().join("bad-name.js"), "").unwrap();
    fs::write(project.path().join("good-name.ts"), "").unwrap();
    fs::write(project.path().join("bad_name.ts"), "").unwrap();
    fs::create_dir(project.path().join("ignored")).unwrap();
    fs::write(project.path().join("ignored/BadName.ts"), "").unwrap();
    fs::create_dir(project.path().join("generated")).unwrap();
    fs::write(project.path().join("generated/BadName.ts"), "").unwrap();

    let output = Command::new(native_ls_lint_binary())
        .current_dir(project.path())
        .arg("--error-output-format")
        .arg("json")
        .arg("--config")
        .arg(&first_path)
        .arg("--config")
        .arg(&second_path)
        .output()
        .expect("native LS-Lint should run");
    assert!(!output.status.success());
    let native_output = if output.stdout.is_empty() {
        output.stderr.as_slice()
    } else {
        output.stdout.as_slice()
    };
    let native: serde_json::Value =
        serde_json::from_slice(native_output).expect("native output parses");
    let mut native_paths = native
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    native_paths.sort();

    let (assura_success, assura_paths) = run_assura_paths(&project);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["bad-name.js", "bad_name.ts"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn native_lslint_golden_multi_config_top_level_keys_replace_previous_rules() {
    let project = TempDir::new().unwrap();
    let first = r#"
ls:
  src:
    .js: camelcase
"#;
    let second = r#"
ls:
  src:
    .ts: kebabcase
"#;
    let first_path = project.path().join("base.yml");
    let second_path = project.path().join("override.yml");
    fs::write(&first_path, first).unwrap();
    fs::write(&second_path, second).unwrap();
    let assura_config = convert_ls_lint_documents_to_config(&[first, second]).unwrap();
    super::write_generated_config(&project, &assura_config);

    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/bad_name.js"), "").unwrap();
    fs::write(project.path().join("src/bad_name.ts"), "").unwrap();

    let output = Command::new(native_ls_lint_binary())
        .current_dir(project.path())
        .arg("--error-output-format")
        .arg("json")
        .arg("--config")
        .arg(&first_path)
        .arg("--config")
        .arg(&second_path)
        .output()
        .expect("native LS-Lint should run");
    assert!(!output.status.success());
    let native_output = if output.stdout.is_empty() {
        output.stderr.as_slice()
    } else {
        output.stdout.as_slice()
    };
    let native: serde_json::Value =
        serde_json::from_slice(native_output).expect("native output parses");
    let mut native_paths = native
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    native_paths.sort();

    let (assura_success, assura_paths) = run_assura_paths(&project);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["src/bad_name.ts"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn native_lslint_golden_scalar_rules_match_default_target_semantics() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  src: kebab-case
  README.md: exists:1
  src/: exists:1
"#;
    assert_native_parity(
        "scalar path no-op family",
        config,
        |root| {
            fs::create_dir(root.join("src")).unwrap();
            fs::write(root.join("src/BadName.js"), "").unwrap();
        },
        false,
        &[""],
    );
}

#[test]
fn native_lslint_golden_explicit_file_target_finalizes_matching_exists_count() {
    let project = TempDir::new().unwrap();
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  src:
    .ts: kebabcase | exists:2
"#;
    write_lslint_config(&project, config);
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/good-name.ts"), "").unwrap();

    let (native_success, native_paths) = run_native_ls_lint_target(&project, "src/good-name.ts");
    let (assura_success, assura_paths) = run_assura_lslint_target(&project, "src/good-name.ts");
    assert!(!native_success);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["src"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn native_lslint_golden_explicit_descendant_file_target_finalizes_index_exists_count() {
    let project = TempDir::new().unwrap();
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  src:
    .ts: kebabcase | exists:1
"#;
    write_lslint_config(&project, config);
    fs::create_dir_all(project.path().join("src/nested")).unwrap();
    fs::write(project.path().join("src/nested/good-name.ts"), "").unwrap();

    let (native_success, native_paths) =
        run_native_ls_lint_target(&project, "src/nested/good-name.ts");
    let (assura_success, assura_paths) =
        run_assura_lslint_target(&project, "src/nested/good-name.ts");
    assert!(!native_success);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["src"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn native_lslint_golden_explicit_glob_descendant_target_finalizes_concrete_index_count() {
    let project = TempDir::new().unwrap();
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  packages/*:
    .ts: kebabcase | exists:1
"#;
    write_lslint_config(&project, config);
    fs::create_dir_all(project.path().join("packages/core/nested")).unwrap();
    fs::write(project.path().join("packages/core/nested/good-name.ts"), "").unwrap();

    let (native_success, native_paths) =
        run_native_ls_lint_target(&project, "packages/core/nested/good-name.ts");
    let (assura_success, assura_paths) =
        run_assura_lslint_target(&project, "packages/core/nested/good-name.ts");
    assert!(!native_success);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["packages/core"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn native_lslint_golden_explicit_directory_target_finalizes_dir_exists_count() {
    let project = TempDir::new().unwrap();
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  src:
    .dir: kebabcase | exists:2
"#;
    write_lslint_config(&project, config);
    fs::create_dir_all(project.path().join("src/one")).unwrap();

    let (native_success, native_paths) = run_native_ls_lint_target(&project, "src/one");
    let (assura_success, assura_paths) = run_assura_lslint_target(&project, "src/one");
    assert!(!native_success);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["src"]);
    assert_eq!(assura_paths, native_paths);
}
