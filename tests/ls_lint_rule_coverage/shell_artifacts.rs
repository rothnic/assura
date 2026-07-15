use std::fs;

use tempfile::TempDir;

#[test]
fn native_lslint_golden_shell_artifact_filenames_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .ts: kebabcase
"#;
    super::assert_native_parity(
        "shell artifact filename family",
        config,
        |root| {
            for filename in [
                "#hash.ts",
                "$(touch pwned).ts",
                "${0}.ts",
                "$null.ts",
                "@scope.ts",
                "[abc].ts",
                "`echo hi`.ts",
                "amp&name.ts",
                "name=VALUE.ts",
                "ok-name.ts",
                "quote'name.ts",
                "semi;colon.ts",
                "two words.ts",
                "{src,tests}.ts",
            ] {
                fs::write(root.join(filename), "").unwrap();
            }
        },
        false,
        &[
            "#hash.ts",
            "$(touch pwned).ts",
            "$null.ts",
            "${0}.ts",
            "@scope.ts",
            "[abc].ts",
            "`echo hi`.ts",
            "amp&name.ts",
            "name=VALUE.ts",
            "quote'name.ts",
            "semi;colon.ts",
            "two words.ts",
            "{src,tests}.ts",
        ],
    );
}

#[test]
fn native_lslint_golden_shell_artifact_directories_under_glob_scopes_match_assura() {
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  packages/*:
    .dir: kebabcase
    .ts: kebabcase
"#;
    super::assert_native_parity(
        "shell artifact glob scope family",
        config,
        |root| {
            for dirname in ["$null", "[brackets]", "{src,tests}", "good-name"] {
                fs::create_dir_all(root.join("packages").join(dirname)).unwrap();
            }
            fs::write(root.join("packages/$null/BadName.ts"), "").unwrap();
            fs::write(root.join("packages/[brackets]/bad_name.ts"), "").unwrap();
            fs::write(root.join("packages/{src,tests}/bad_name.ts"), "").unwrap();
            fs::write(root.join("packages/good-name/good-name.ts"), "").unwrap();
        },
        false,
        &[
            "packages/$null",
            "packages/$null/BadName.ts",
            "packages/[brackets]",
            "packages/[brackets]/bad_name.ts",
            "packages/{src,tests}",
            "packages/{src,tests}/bad_name.ts",
        ],
    );
}

#[test]
fn native_lslint_golden_explicit_shell_artifact_file_target_matches_assura() {
    let project = TempDir::new().unwrap();
    let config = r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .ts: kebabcase
"#;
    super::write_lslint_config(&project, config);
    fs::write(project.path().join("$null.ts"), "").unwrap();

    let (native_success, native_paths) = super::run_native_ls_lint_target(&project, "$null.ts");
    let (assura_success, assura_paths) = super::run_assura_lslint_target(&project, "$null.ts");
    assert!(!native_success);
    assert!(!assura_success);
    assert_eq!(native_paths, vec!["$null.ts"]);
    assert_eq!(assura_paths, native_paths);
}

#[test]
fn assura_native_policy_catches_shell_artifact_files_and_glob_like_names() {
    let project = TempDir::new().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      allowed_names:
        - ok-name.ts
      allowed_patterns:
        - README.*
      forbidden_patterns:
        - "*.tmp"
        - "*.bak"
      allow_extra: false
    directories:
      allowed_names:
        - good-name
      allowed_patterns:
        - packages-*
      forbidden_patterns:
        - "*.tmp"
      allow_extra: false
"#,
    )
    .unwrap();

    for filename in [
        "#hash.ts",
        "$(touch pwned).ts",
        "$null",
        "$null.ts",
        "$null.tmp",
        "--help.ts",
        "@scope.ts",
        "[abc].bak",
        "[abc].ts",
        "`echo hi`.ts",
        "amp&name.ts",
        "name=VALUE.ts",
        "ok-name.ts",
        "quote'name.ts",
        "semi;colon.ts",
        "two words.ts",
        "{src,tests}.ts",
    ] {
        fs::write(project.path().join(filename), "").unwrap();
    }
    for dirname in [
        "$null-dir",
        "$null-dir.tmp",
        "--help",
        "[brackets]",
        "`echo hi`",
        "good-name",
        "packages-core",
        "{src,tests}",
    ] {
        fs::create_dir(project.path().join(dirname)).unwrap();
    }

    let report = super::super::run_json_check(&project);
    let paths = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<std::collections::HashSet<_>>();

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    for expected in [
        "#hash.ts",
        "$(touch pwned).ts",
        "$null",
        "$null-dir",
        "$null-dir.tmp",
        "$null.ts",
        "$null.tmp",
        "--help",
        "--help.ts",
        "@scope.ts",
        "[abc].bak",
        "[abc].ts",
        "`echo hi`",
        "`echo hi`.ts",
        "amp&name.ts",
        "name=VALUE.ts",
        "quote'name.ts",
        "semi;colon.ts",
        "two words.ts",
        "{src,tests}",
        "{src,tests}.ts",
    ] {
        assert!(
            paths.contains(expected),
            "missing {expected}; report was:\n{report:#}"
        );
    }
    for allowed in ["ok-name.ts", "good-name", "packages-core"] {
        assert!(
            !paths.contains(allowed),
            "allowed path {allowed} was rejected:\n{report:#}"
        );
    }
}
