//! Integration coverage for pattern-scoped file directive attributes.

use assura::cli::{
    run_structure_check, run_structure_check_with_artifact, CompiledStructureConfigArtifact,
};
use assura::config::config::ConfigLoader;
use std::fs;
use tempfile::TempDir;

fn write_config(project: &TempDir) {
    let config_dir = project.path().join(".assura");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yml"),
        r#"
rules:
  "@source-file":
    naming: kebab-case
    max_lines: 3
    max_size: 64B
  "@test-file":
    naming: kebab-case
    max_lines: 1
structure:
  src/:
    .ts: "@source-file"
    .tsx: "@source-file"
    .test.ts: "@test-file"
    components/:
      .test.ts: "@test-file"
  "src/**/generated/":
    inherit: false
"#,
    )
    .unwrap();
}

#[test]
fn directive_limits_only_matching_patterns_and_keeps_naming_specific() {
    let project = TempDir::new().unwrap();
    write_config(&project);
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("good-file.ts"), "one\ntwo\nthree\n").unwrap();
    fs::write(src.join("good-view.tsx"), "one\ntwo\nthree\n").unwrap();
    fs::write(src.join("good-file.test.ts"), "one\n").unwrap();
    fs::write(src.join("README.md"), "not source\n".repeat(10)).unwrap();
    fs::create_dir_all(src.join("components/generated")).unwrap();
    fs::write(
        src.join("components/generated/unchecked.ts"),
        "outside inherited scope\n".repeat(10),
    )
    .unwrap();

    let passing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(
        passing.success,
        "unexpected violations: {:#?}",
        passing.violations
    );

    fs::write(src.join("BadName.ts"), "one\n").unwrap();
    fs::write(src.join("too-long.tsx"), "one\ntwo\nthree\nfour\n").unwrap();
    fs::write(src.join("too-large.ts"), "x".repeat(65)).unwrap();
    fs::write(src.join("too-long.test.ts"), "one\ntwo\n").unwrap();
    fs::create_dir_all(src.join("components")).unwrap();
    fs::write(
        src.join("components/too-long.ts"),
        "one\ntwo\nthree\nfour\n",
    )
    .unwrap();
    let failing = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("BadName.ts") && violation.rule == "file_naming"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.tsx") && violation.rule == "max_lines"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("too-large.ts") && violation.rule == "max_size"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.test.ts") && violation.rule == "max_lines"
    }));
    assert!(failing.violations.iter().any(|violation| {
        violation.path.ends_with("components/too-long.ts") && violation.rule == "max_lines"
    }));
    assert!(!failing.violations.iter().any(|violation| violation
        .path
        .ends_with("components/generated/unchecked.ts")));
    assert!(!failing
        .violations
        .iter()
        .any(|violation| violation.path.ends_with("README.md")));
}

#[test]
fn compiled_artifact_preserves_pattern_scoped_limits() {
    let project = TempDir::new().unwrap();
    write_config(&project);
    let src = project.path().join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("too-long.ts"), "one\ntwo\nthree\nfour\n").unwrap();
    fs::write(src.join("README.md"), "not source\n".repeat(10)).unwrap();

    let config_path = project.path().join(".assura/config.yml");
    let config = ConfigLoader::load_validated(&config_path).unwrap();
    let report = run_structure_check_with_artifact(
        project.path().to_path_buf(),
        config_path,
        project.path().to_path_buf(),
        CompiledStructureConfigArtifact::new(config),
        false,
    )
    .unwrap();

    assert!(report.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.ts") && violation.rule == "max_lines"
    }));
    assert!(!report
        .violations
        .iter()
        .any(|violation| violation.path.ends_with("README.md")));
}

#[test]
fn explicit_file_globs_distinguish_direct_children_from_descendants() {
    let project = TempDir::new().unwrap();
    let config_dir = project.path().join(".assura");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("config.yml"),
        r#"
rules:
  "@direct-source":
    max_lines: 1
    max_size: 8B
  "@recursive-source":
    max_lines: 2
    max_size: 64B
structure:
  ./:
    "./**/*.ts": "@recursive-source"
    "./*.ts": "@direct-source"
"#,
    )
    .unwrap();
    let nested = project.path().join("src");
    fs::create_dir_all(&nested).unwrap();
    fs::write(project.path().join("root-lines.ts"), "a\nb\n").unwrap();
    fs::write(project.path().join("root-size.ts"), "123456789").unwrap();
    fs::write(nested.join("nested-lines.ts"), "a\nb\nc\n").unwrap();
    fs::write(nested.join("nested-size.ts"), "123456789").unwrap();

    let loaded = ConfigLoader::load_validated(&config_dir.join("config.yml")).unwrap();
    let files = loaded
        .structure
        .get("./")
        .and_then(|node| node.files.as_ref())
        .unwrap();
    assert_eq!(
        files
            .max_lines_patterns
            .as_ref()
            .and_then(|patterns| patterns.get("./*.ts")),
        Some(&1),
        "patterns: {:#?}",
        files.max_lines_patterns
    );
    assert_eq!(
        files
            .max_lines_patterns
            .as_ref()
            .and_then(|patterns| patterns.get("./**/*.ts")),
        Some(&2)
    );

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(
        report.violations.iter().any(|violation| {
            violation.path.ends_with("root-lines.ts")
                && violation.rule == "max_lines"
                && violation.message.contains("limit 1")
        }),
        "violations: {:#?}",
        report.violations
    );
    assert!(report.violations.iter().any(|violation| {
        violation.path.ends_with("root-size.ts")
            && violation.rule == "max_size"
            && violation.message.contains("limit 8")
    }));
    assert!(report.violations.iter().any(|violation| {
        violation.path.ends_with("nested-lines.ts")
            && violation.rule == "max_lines"
            && violation.message.contains("limit 2")
    }));
    assert!(!report
        .violations
        .iter()
        .any(|violation| violation.path.ends_with("nested-size.ts")));
}

#[test]
fn independent_hierarchy_scopes_compose_and_can_reset() {
    let project = TempDir::new().unwrap();
    let config_dir = project.path().join(".assura");
    let source = project.path().join("packages/core/src");
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("too-long.ts"), "a\nb\nc\nd\n").unwrap();
    fs::write(source.join("too-long.test.ts"), "a\nb\n").unwrap();

    let config = r#"
rules:
  "@source": { max_lines: 3 }
  "@test": { max_lines: 1 }
structure:
  ./:
    .ts: "@source"
  "packages/*/src/":
    .test.ts: "@test"
"#;
    fs::write(config_dir.join("config.yml"), config).unwrap();
    let inherited = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(inherited.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.ts")
            && violation.rule == "max_lines"
            && violation.message.contains("limit 3")
    }));
    assert!(inherited.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.test.ts")
            && violation.rule == "max_lines"
            && violation.message.contains("limit 1")
    }));

    fs::write(
        config_dir.join("config.yml"),
        config.replace(
            "  \"packages/*/src/\":\n    .test.ts",
            "  \"packages/*/src/\":\n    inherit: false\n    .test.ts",
        ),
    )
    .unwrap();
    let reset = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(!reset
        .violations
        .iter()
        .any(|violation| violation.path.ends_with("too-long.ts")));
    assert!(reset.violations.iter().any(|violation| {
        violation.path.ends_with("too-long.test.ts")
            && violation.rule == "max_lines"
            && violation.message.contains("limit 1")
    }));
}

#[test]
fn exact_file_attributes_do_not_leak_to_same_named_descendants() {
    let project = TempDir::new().unwrap();
    let config_dir = project.path().join(".assura");
    let nested = project.path().join("packages/core");
    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        config_dir.join("config.yml"),
        r#"
structure:
  ./:
    AGENTS.md:
      exists: 1
      max_lines: 1
"#,
    )
    .unwrap();
    fs::write(project.path().join("AGENTS.md"), "a\nb\n").unwrap();
    fs::write(nested.join("AGENTS.md"), "a\nb\n").unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
    assert!(report.violations.iter().any(|violation| {
        violation.path == std::path::Path::new("AGENTS.md") && violation.rule == "max_lines"
    }));
    assert!(!report
        .violations
        .iter()
        .any(|violation| violation.path.ends_with("packages/core/AGENTS.md")));
}
