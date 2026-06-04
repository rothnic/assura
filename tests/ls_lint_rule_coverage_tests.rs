use std::fs;
use std::process::Command;

use assura::config::ls_compat::convert_ls_lint_to_config;
use tempfile::TempDir;

#[path = "fixtures/ls-lint/native_golden.rs"]
mod ls_lint_native_golden;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
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

fn run_json_check(project: &TempDir) -> serde_json::Value {
    run_json_check_path(project, project.path(), &[])
}

fn run_json_check_path(
    project: &TempDir,
    path: &std::path::Path,
    extra_args: &[&str],
) -> serde_json::Value {
    let mut command = Command::new(assura_bin());
    command
        .arg("check")
        .arg(path)
        .arg("--config")
        .arg(project.path().join(".assura/config.yml"))
        .arg("--format")
        .arg("json");
    command.args(extra_args);
    let output = command.output().unwrap();

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn lslint_migration_rejects_unknown_rules_and_invalid_syntax() {
    for (yaml, expected) in [
        (
            "ls:\n  .js: dot.case\n",
            "Unknown LS-Lint rule name 'dot.case'",
        ),
        (
            "ls:\n  .js: kebab-case|camelCase\n",
            "multiple rules must use ' | '",
        ),
        ("ls:\n  .js: \"kebab-case | \"\n", "empty rule around ' | '"),
        ("ls:\n  .js: \"regex:\"\n", "pattern is empty"),
        ("ls:\n  .js: regex:[\n", "Invalid LS-Lint regex rule"),
        (
            "ls:\n  src: dot.case\n",
            "Unknown LS-Lint rule name 'dot.case'",
        ),
    ] {
        let error = convert_ls_lint_to_config(yaml).unwrap_err();
        assert!(
            error.contains(expected),
            "expected {expected:?} in error {error:?}"
        );
    }
}

#[test]
fn lslint_migration_rejects_unsupported_yaml_shapes() {
    for (yaml, expected) in [
        ("[]\n", "document must be a mapping"),
        ("rules: {}\n", "unknown top-level key 'rules'"),
        ("ignore: ignored/**\n", "'ignore' must be a sequence"),
        ("ignore:\n  - 1\n", "'ignore' entries must be strings"),
        ("ls: []\n", "'ls' must be a mapping"),
        (
            "ls:\n  .js:\n    nested: nope\n",
            "rule for '.js' must be a string",
        ),
    ] {
        let error = convert_ls_lint_to_config(yaml).unwrap_err();
        assert!(
            error.contains(expected),
            "expected {expected:?} in error {error:?}"
        );
    }
}

#[test]
fn converted_lslint_directory_pattern_scopes_match_existing_directories() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .png: snake_case
  src/**/c:
    .png: PascalCase
    packages:
      .png: snake_case
  src/{a,b}/*:
    .png: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("snake_case.png"), "").unwrap();
    fs::create_dir_all(project.path().join("src/a/a")).unwrap();
    fs::write(project.path().join("src/a/a/kebab-case.png"), "").unwrap();
    fs::create_dir_all(project.path().join("src/b/b")).unwrap();
    fs::write(project.path().join("src/b/b/kebab-case.png"), "").unwrap();
    fs::create_dir_all(project.path().join("src/c/c/packages")).unwrap();
    fs::write(project.path().join("src/c/c/PascalCase.png"), "").unwrap();
    fs::write(project.path().join("src/c/c/packages/snake_case.png"), "").unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn converted_lslint_directory_pattern_scopes_report_mismatches() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .png: snake_case
  src/**/c:
    .png: PascalCase
    packages:
      .png: snake_case
  src/{a,b}/*:
    .png: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir_all(project.path().join("src/a/a")).unwrap();
    fs::write(project.path().join("src/a/a/not_kebab.png"), "").unwrap();
    fs::create_dir_all(project.path().join("src/c/c/packages")).unwrap();
    fs::write(project.path().join("src/c/c/not_pascal.png"), "").unwrap();
    fs::write(
        project.path().join("src/c/c/packages/not-snake-case.png"),
        "",
    )
    .unwrap();

    let report = run_json_check(&project);
    let paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 3, "report was:\n{report:#}");
    assert!(
        paths.contains("src/a/a/not_kebab.png"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("src/c/c/not_pascal.png"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("src/c/c/packages/not-snake-case.png"),
        "report was:\n{report:#}"
    );
}

#[test]
fn converted_lslint_regex_rules_are_full_string_matches() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .js: regex:config
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("config.js"), "export {};\n").unwrap();
    fs::write(project.path().join("configx.js"), "export {};\n").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["rule"], "file_naming");
    assert_eq!(violations[0]["path"], "configx.js");
}

#[test]
fn converted_lslint_multiple_regex_rules_preserve_or_semantics() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .js: regex:Schema(\_test)? | regex:Resolver(\_test)?
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("Schema.js"), "export {};\n").unwrap();
    fs::write(project.path().join("Resolver_test.js"), "export {};\n").unwrap();
    fs::write(project.path().join("Other.js"), "export {};\n").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "Other.js");
}

#[test]
fn converted_lslint_regex_negation_matches_upstream_semantics() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .js: regex:![0-9]+
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("abc.js"), "export {};\n").unwrap();
    fs::write(project.path().join("123.js"), "export {};\n").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "123.js");
}

#[test]
fn converted_lslint_multi_extension_rules_use_upstream_extension_combinations() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .js: kebab-case
  .num.js: regex:![0-9]+
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("BadName.test.js"), "export {};\n").unwrap();
    fs::write(project.path().join("123.num.js"), "export {};\n").unwrap();
    fs::write(project.path().join("abc.num.js"), "export {};\n").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "123.num.js");
    assert!(
        violations
            .iter()
            .all(|violation| violation["path"] != "BadName.test.js"),
        "native LS-Lint .js rules do not apply to .test.js files:\n{report:#}"
    );
}

#[test]
fn converted_lslint_regex_directory_substitutions_match_upstream_semantics() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  google:
    test:
      .js: regex:${1}_${0}
  gen:
    swu1:
      data:
        .js: regex:${1}
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir_all(project.path().join("google/test")).unwrap();
    fs::write(project.path().join("google/test/google_test.js"), "").unwrap();
    fs::write(project.path().join("google/test/test.js"), "").unwrap();
    fs::create_dir_all(project.path().join("gen/swu1/data")).unwrap();
    fs::write(project.path().join("gen/swu1/data/swu1.js"), "").unwrap();
    fs::write(project.path().join("gen/swu1/data/data.js"), "").unwrap();

    let report = run_json_check(&project);
    let paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 2, "report was:\n{report:#}");
    assert!(
        paths.contains("google/test/test.js"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("gen/swu1/data/data.js"),
        "report was:\n{report:#}"
    );
}

#[test]
fn invalid_lslint_exists_syntax_returns_clear_errors() {
    for rule in [
        "exists:",
        "exists:-1",
        "exists:1-",
        "exists:32768",
        "exists:2342323423234",
    ] {
        let ls_lint_yaml = format!(
            r#"
ls:
  .md: "{rule}"
"#
        );
        let error = convert_ls_lint_to_config(&ls_lint_yaml).unwrap_err();
        assert!(
            error.contains("Invalid LS-Lint exists rule"),
            "unexpected error for {rule}: {error}"
        );
    }
}

#[test]
fn converted_lslint_exists_extra_range_segments_match_upstream_parser() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .md: exists:1-2-3
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn converted_lslint_bare_exists_and_directory_exists_are_direct_counts() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .md: exists
  docs:
    .dir: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn converted_lslint_root_dir_exists_matches_upstream_zero_count() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .dir: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("good-dir")).unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["rule"], "exists_count");
    assert_eq!(violations[0]["path"], "");
}

#[test]
fn converted_lslint_root_dir_exists_zero_passes_with_upstream_zero_count() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .dir: exists:0
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("good-dir")).unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn converted_lslint_scalar_rules_match_default_target_behavior() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  README.md: exists:1
  src/: exists:1
  src: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/BadName.js"), "").unwrap();

    let report = run_json_check(&project);

    let paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 1, "report was:\n{report:#}");
    assert!(paths.contains(""), "report was:\n{report:#}");
}

#[test]
fn converted_lslint_non_dot_scalar_invalid_rules_are_rejected() {
    let error = convert_ls_lint_to_config(
        r#"
ls:
  src: dot.case
"#,
    )
    .unwrap_err();

    assert!(
        error.contains("Unknown LS-Lint rule name 'dot.case'"),
        "unexpected error: {error}"
    );
}

#[test]
fn converted_lslint_wildcard_extension_precedence_uses_most_specific_rule() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .*: snake_case
  .js: kebab-case
  .*.js: PascalCase
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("read_me.md"), "# Fixture\n").unwrap();
    fs::write(project.path().join("good-name.js"), "export {};\n").unwrap();
    fs::write(project.path().join("Button.test.js"), "export {};\n").unwrap();
    fs::write(project.path().join("bad_name.js"), "export {};\n").unwrap();
    fs::write(project.path().join("bad-name.test.js"), "export {};\n").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();
    let paths = violations
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 2, "report was:\n{report:#}");
    assert!(paths.contains("bad_name.js"), "report was:\n{report:#}");
    assert!(
        paths.contains("bad-name.test.js"),
        "report was:\n{report:#}"
    );
}

#[test]
fn converted_lslint_case_rule_edges_cover_upstream_examples() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .lower: lowercase
  .camel: camelCase
  .pascal: PascalCase
  .snake: snake_case
  .scream: SCREAMING_SNAKE_CASE
  .kebab: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    for filename in [
        "abc-1.lower",
        "camelVCase.camel",
        "PascalVCase.pascal",
        "snake_123_case.snake",
        "SNAKE_123_CASE.scream",
        "kebab-123-test.kebab",
    ] {
        fs::write(project.path().join(filename), "").unwrap();
    }
    for filename in ["abC.lower", "camelCASE123.camel", "PASCALCASE.pascal"] {
        fs::write(project.path().join(filename), "").unwrap();
    }

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();
    let paths = violations
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 3, "report was:\n{report:#}");
    assert!(paths.contains("abC.lower"), "report was:\n{report:#}");
    assert!(
        paths.contains("camelCASE123.camel"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("PASCALCASE.pascal"),
        "report was:\n{report:#}"
    );
}

#[test]
fn converted_lslint_dir_rule_validates_scoped_directory_itself() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  Bad-Dir:
    .dir: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("Bad-Dir")).unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["rule"], "directory_naming");
    assert_eq!(violations[0]["path"], "Bad-Dir");
}

#[test]
fn converted_lslint_dir_exists_rules_validate_scope_presence() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  missing:
    .dir: exists:1
  absent:
    .dir: exists:0
  present-zero:
    .dir: exists:0
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);
    fs::create_dir(project.path().join("present-zero")).unwrap();

    let report = run_json_check(&project);
    let paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(paths.len(), 2, "report was:\n{report:#}");
    assert!(paths.contains("missing"), "report was:\n{report:#}");
    assert!(paths.contains("present-zero"), "report was:\n{report:#}");
    assert!(!paths.contains("absent"), "report was:\n{report:#}");
}

#[test]
fn converted_lslint_file_exists_under_missing_scope_still_fails() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  src:
    .md: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["rule"], "exists_count");
    assert_eq!(violations[0]["path"], "src");
}

#[test]
fn converted_lslint_canonical_case_aliases_match_upstream_names() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .camel: camelcase
  .pascal: pascalcase
  .snake: snakecase
  .scream: screamingsnakecase
  .kebab: kebabcase
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    for filename in [
        "goodName.camel",
        "GoodName.pascal",
        "good_name.snake",
        "GOOD_NAME.scream",
        "good-name.kebab",
    ] {
        fs::write(project.path().join(filename), "").unwrap();
    }

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
}

#[test]
fn converted_lslint_raw_regex_alternation_keeps_upstream_anchor_semantics() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  .js: regex:foo|bar
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("foo.js"), "").unwrap();
    fs::write(project.path().join("bar.js"), "").unwrap();
    fs::write(project.path().join("foobar.js"), "").unwrap();
    fs::write(project.path().join("baz.js"), "").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "baz.js");
}

#[test]
fn converted_lslint_glob_and_brace_ignore_patterns_exclude_matches() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - generated/**
  - src/{a,b}/*.tmp
ls:
  .tmp: kebab-case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::create_dir(project.path().join("generated")).unwrap();
    fs::write(project.path().join("generated/BAD.tmp"), "").unwrap();
    fs::create_dir_all(project.path().join("src/a")).unwrap();
    fs::write(project.path().join("src/a/BAD.tmp"), "").unwrap();
    fs::create_dir_all(project.path().join("src/b")).unwrap();
    fs::write(project.path().join("src/b/BAD.tmp"), "").unwrap();
    fs::create_dir_all(project.path().join("src/c")).unwrap();
    fs::write(project.path().join("src/c/BAD.tmp"), "").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "src/c/BAD.tmp");
}

#[test]
fn check_can_use_explicit_lslint_target_semantics_without_losing_native_recursion() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ls:
  src:
    .ts: kebabcase
    .md: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/BadName.ts"), "").unwrap();

    let full_tree_report = run_json_check(&project);
    let native_file_report =
        run_json_check_path(&project, &project.path().join("src/BadName.ts"), &[]);
    let lslint_file_report = run_json_check_path(
        &project,
        &project.path().join("src/BadName.ts"),
        &["--ls-lint-target-semantics"],
    );
    let native_report = run_json_check_path(&project, &project.path().join("src"), &[]);
    let lslint_target_report = run_json_check_path(
        &project,
        &project.path().join("src"),
        &["--ls-lint-target-semantics"],
    );

    assert_eq!(
        full_tree_report["success"], false,
        "report was:\n{full_tree_report:#}"
    );
    assert_eq!(
        native_file_report["success"], false,
        "report was:\n{native_file_report:#}"
    );
    assert_eq!(
        lslint_file_report["success"], false,
        "report was:\n{lslint_file_report:#}"
    );
    assert_eq!(
        native_report["success"], false,
        "report was:\n{native_report:#}"
    );
    assert_eq!(
        lslint_target_report["success"], true,
        "report was:\n{lslint_target_report:#}"
    );
}

#[test]
fn cli_migrate_accepts_multiple_lslint_configs_in_merge_order() {
    let project = TempDir::new().unwrap();
    let first = project.path().join("base.yml");
    let second = project.path().join("override.yml");
    fs::write(
        &first,
        r#"
ignore:
  - ignored/**
ls:
  .js: camelcase
  .ts: snakecase
"#,
    )
    .unwrap();
    fs::write(
        &second,
        r#"
ignore:
  - generated/**
ls:
  .ts: kebabcase
"#,
    )
    .unwrap();

    let assura_dir = project.path().join(".assura");
    fs::create_dir(&assura_dir).unwrap();
    let output_path = assura_dir.join("config.yml");
    let output = Command::new(assura_bin())
        .arg("migrate")
        .arg(&first)
        .arg(&second)
        .arg("--output")
        .arg(&output_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fs::write(project.path().join("goodName.js"), "").unwrap();
    fs::write(project.path().join("good-name.ts"), "").unwrap();
    fs::write(project.path().join("bad_name.ts"), "").unwrap();
    fs::create_dir(project.path().join("ignored")).unwrap();
    fs::write(project.path().join("ignored/BAD.ts"), "").unwrap();
    fs::create_dir(project.path().join("generated")).unwrap();
    fs::write(project.path().join("generated/BAD.ts"), "").unwrap();

    let report = run_json_check(&project);
    let violations = report["violations"].as_array().unwrap();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 1, "report was:\n{report:#}");
    assert_eq!(violations[0]["path"], "bad_name.ts");
}

#[test]
fn check_with_external_migrated_config_uses_explicit_directory_as_root() {
    let project = TempDir::new().unwrap();
    let fixture = project.path().join("fixture");
    fs::create_dir(&fixture).unwrap();
    fs::write(
        project.path().join(".ls-lint.yml"),
        r#"
ignore:
  - ignored/**
ls:
  .js: kebab-case
  src:
    .ts: camelCase
"#,
    )
    .unwrap();

    let output_config = project.path().join("generated-assura.yml");
    let migrate = Command::new(assura_bin())
        .arg("migrate")
        .arg(project.path().join(".ls-lint.yml"))
        .arg("--output")
        .arg(&output_config)
        .output()
        .unwrap();
    assert!(
        migrate.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&migrate.stdout),
        String::from_utf8_lossy(&migrate.stderr)
    );

    fs::write(fixture.join("good-name.js"), "").unwrap();
    fs::create_dir(fixture.join("src")).unwrap();
    fs::write(fixture.join("src/goodName.ts"), "").unwrap();
    fs::create_dir(fixture.join("ignored")).unwrap();
    fs::write(fixture.join("ignored/BadName.js"), "").unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--config")
        .arg(&output_config)
        .arg(&fixture)
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "failed to parse check output as json: {error}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });

    assert_eq!(output.status.code(), Some(0), "report was:\n{report:#}");
    assert_eq!(report["success"], true, "report was:\n{report:#}");
}
