use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use assura::cli::run_structure_check;
use assura::config::ls_compat::convert_ls_lint_to_config;
use tempfile::TempDir;

#[path = "realistic_lslint_fixtures.rs"]
mod realistic_lslint_fixtures;

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

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn run_json_check(project: &TempDir) -> serde_json::Value {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn violation_rules(report: &serde_json::Value) -> Vec<String> {
    report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn realistic_fixture_manifest_is_pinned_and_complete() {
    let manifest = realistic_lslint_fixtures::parse_manifest();
    assert_eq!(manifest.schema_version, 1);

    for family in realistic_lslint_fixtures::realistic_fixture_families() {
        let entry = manifest
            .fixtures
            .iter()
            .find(|entry| entry.id == family.id)
            .unwrap_or_else(|| panic!("manifest missing fixture {}", family.id));

        assert!(!entry.name.is_empty());
        assert!(!entry.purpose.is_empty());
        assert!(!entry.source.repository.is_empty());
        assert!(!entry.source.revision.is_empty());
        assert!(matches!(
            entry.source.kind.as_str(),
            "generated" | "external_git"
        ));
        assert!(
            entry.cohort == "stable_baseline" || entry.cohort.starts_with("feature_"),
            "unexpected cohort for {}: {}",
            entry.id,
            entry.cohort
        );
        assert!(!entry.ls_lint_rules.is_empty());
        assert!(!entry.assura_rules.is_empty());
    }

    let extension = manifest
        .fixtures
        .iter()
        .find(|entry| entry.id == "assura_exact_filename_exists_extension")
        .expect("exact filename exists extension fixture should be declared");
    assert!(!extension.native_lslint_parity);
    assert_eq!(extension.assura_extensions, ["exact_filename_exists"]);

    assert!(
        manifest
            .fixtures
            .iter()
            .any(|entry| entry.source.kind == "external_git"),
        "manifest must include at least one pinned external_git source"
    );
}

#[test]
fn external_git_fixture_materializer_uses_pinned_revision_and_cache() {
    let upstream = TempDir::new().unwrap();
    std::process::Command::new("git")
        .arg("init")
        .arg("--quiet")
        .arg(upstream.path())
        .status()
        .expect("git init should run");
    fs::write(upstream.path().join("README.md"), "# Fixture\n").unwrap();
    std::process::Command::new("git")
        .arg("-C")
        .arg(upstream.path())
        .arg("add")
        .arg("README.md")
        .status()
        .expect("git add should run");
    std::process::Command::new("git")
        .arg("-C")
        .arg(upstream.path())
        .args(["-c", "user.email=test@example.com", "-c", "user.name=Test"])
        .args(["commit", "--quiet", "-m", "initial"])
        .status()
        .expect("git commit should run");
    let commit = std::process::Command::new("git")
        .arg("-C")
        .arg(upstream.path())
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("git rev-parse should run");
    assert!(commit.status.success());
    let revision = String::from_utf8_lossy(&commit.stdout).trim().to_string();

    fs::write(upstream.path().join("README.md"), "# Changed\n").unwrap();

    let entry = realistic_lslint_fixtures::FixtureManifestEntry {
        id: "local_external".to_string(),
        name: "Local external".to_string(),
        source: realistic_lslint_fixtures::FixtureSource {
            kind: "external_git".to_string(),
            repository: upstream.path().to_string_lossy().to_string(),
            revision,
        },
        purpose: "prove pinned external materialization".to_string(),
        ls_lint_rules: vec!["external_fixture_source".to_string()],
        assura_rules: vec!["fixture_materialization".to_string()],
        cohort: "stable_baseline".to_string(),
        native_lslint_parity: true,
        assura_extensions: vec![],
    };
    let cache = TempDir::new().unwrap();
    let destination = TempDir::new().unwrap();
    realistic_lslint_fixtures::materialize_external_git_fixture(
        &entry,
        cache.path(),
        destination.path(),
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(destination.path().join("README.md")).unwrap(),
        "# Fixture\n"
    );
    assert!(!destination.path().join(".git").exists());
    assert!(
        fs::read_dir(cache.path()).unwrap().next().is_some(),
        "materializer should populate cache"
    );
}

#[test]
fn realistic_fixture_families_cover_valid_and_invalid_shapes() {
    for family in realistic_lslint_fixtures::realistic_fixture_families() {
        let fixture = realistic_lslint_fixtures::materialize_fixture(
            *family,
            realistic_lslint_fixtures::FixtureVariant::Valid,
        );
        let valid_report =
            run_structure_check(Some(fixture.project.path().to_path_buf()), None, false).unwrap();
        assert!(
            valid_report.success,
            "valid fixture {} should pass: {:#?}",
            family.id, valid_report.violations
        );
        assert_eq!(valid_report.violations.len(), 0);

        let fixture = realistic_lslint_fixtures::materialize_fixture(
            *family,
            realistic_lslint_fixtures::FixtureVariant::Invalid,
        );
        let invalid_report =
            run_structure_check(Some(fixture.project.path().to_path_buf()), None, false).unwrap();
        assert!(
            !invalid_report.success,
            "invalid fixture {} should fail",
            family.id
        );
        let rules: std::collections::HashSet<_> = invalid_report
            .violations
            .iter()
            .map(|violation| violation.rule.as_str())
            .collect();
        for expected in fixture.expected_rules {
            assert!(
                rules.contains(expected),
                "fixture {} missing expected rule {expected}; saw {:#?}",
                family.id,
                invalid_report.violations
            );
        }
    }
}

#[test]
fn converted_ls_lint_rules_cover_core_parity_surface() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
  - ignored/**
ls:
  .dir: kebab-case
  .ts: kebab-case | snake_case
  .log: exists:0
  .md: exists:1-2
  src:
    .dir: snake_case
    .rs: snake_case
  packages:
    core:
      .ts: camelCase
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    fs::write(project.path().join("also_good.ts"), "export {};\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("src/good_dir")).unwrap();
    fs::write(project.path().join("src/main_file.rs"), "fn main() {}\n").unwrap();
    fs::create_dir_all(project.path().join("packages/core")).unwrap();
    fs::write(
        project.path().join("packages/core/indexFile.ts"),
        "export {};\n",
    )
    .unwrap();
    fs::create_dir(project.path().join("ignored")).unwrap();
    fs::write(project.path().join("ignored/BadName.ts"), "export {};\n").unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn converted_ls_lint_rules_report_expected_failures() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
ls:
  .dir: kebab-case
  .ts: kebab-case | snake_case
  .log: exists:0
  .md: exists:1-2
  src:
    .dir: snake_case
    .rs: snake_case
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();
    fs::write(project.path().join("notes.md"), "# Notes\n").unwrap();
    fs::write(project.path().join("extra.md"), "# Extra\n").unwrap();
    fs::write(project.path().join("BadName.ts"), "export {};\n").unwrap();
    fs::write(project.path().join("debug.log"), "debug\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("src/bad-dir")).unwrap();
    fs::write(project.path().join("src/main-file.rs"), "fn main() {}\n").unwrap();

    let report = run_json_check(&project);
    let rules = violation_rules(&report);

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert!(rules.contains(&"file_naming".to_string()));
    assert!(rules.contains(&"directory_naming".to_string()));
    assert!(rules.contains(&"exists_count".to_string()));
}

#[test]
fn converted_exact_file_exists_is_a_file_count_not_required_directory() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
ls:
  README.md: exists:1
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
fn converted_missing_exact_file_exists_reports_count_not_required_directory() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
ls:
  README.md: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    let report = run_json_check(&project);
    let rules = violation_rules(&report);

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert!(rules.contains(&"exists_count".to_string()));
    assert!(!rules.contains(&"required_directory".to_string()));
}

#[test]
fn direct_child_count_constraints_do_not_recurse() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        "*.rs": "1"
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
      src/:
        files:
          naming: snake_case
"#,
    );

    fs::write(project.path().join("main.rs"), "fn main() {}\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/lib.rs"), "pub fn lib() {}\n").unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
}

#[test]
fn direct_content_policy_is_not_inherited_recursively() {
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
    directories:
      allowed_names:
        - src
      allow_extra: false
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
      src/:
        files:
          naming: kebab-case
"#,
    );

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/extra-file.rs"), "fn main() {}\n").unwrap();

    let report = run_json_check(&project);

    assert_eq!(report["success"], true, "report was:\n{report:#}");
}

#[test]
fn converted_scopes_do_not_require_directories_without_exists_rules() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
ls:
  src:
    .rs: snake_case
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
fn explicit_directory_exists_rule_still_requires_matching_directory() {
    let project = TempDir::new().unwrap();
    let config = convert_ls_lint_to_config(
        r#"
ignore:
  - .assura/**
ls:
  docs/: exists:1
"#,
    )
    .unwrap();
    write_generated_config(&project, &config);

    let report = run_json_check(&project);
    let rules = violation_rules(&report);

    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert!(rules.contains(&"exists_count".to_string()));
    assert!(!rules.contains(&"required_directory".to_string()));
}

#[test]
fn unsupported_lslint_directory_pattern_scopes_return_clear_errors() {
    for scope in ["packages/*", "**", "{src,tests}"] {
        let ls_lint_yaml = format!(
            r#"
ls:
  "{scope}":
    .ts: kebab-case
"#
        );

        let error = convert_ls_lint_to_config(&ls_lint_yaml).unwrap_err();
        assert!(
            error.contains("Unsupported LS-Lint directory scope"),
            "unexpected error for {scope}: {error}"
        );
        assert!(
            error.contains(scope),
            "error should name the unsupported scope {scope}: {error}"
        );
    }
}

#[test]
#[ignore = "manual performance audit fixture; run with --ignored --nocapture"]
fn ls_lint_parity_audit_performance_shapes() {
    let scenarios = [
        Scenario::sized("small", 8, 24),
        Scenario::sized("medium", 32, 80),
        Scenario::sized("large", 64, 160),
        Scenario::deep("deep_tree", 80),
        Scenario::wide("wide_tree", 800),
        Scenario::ignored("many_ignored_generated_dirs", 120, 30),
        Scenario::direct_checks("many_direct_content_checks", 160),
        Scenario::rule_heavy("many_wildcard_extension_path_rules", 120, 80),
    ];

    println!("scenario,files,dirs,elapsed_ms,violations");
    for scenario in scenarios {
        let project = TempDir::new().unwrap();
        scenario.materialize(&project);
        let start = Instant::now();
        let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();
        let elapsed = start.elapsed();
        println!(
            "{},{},{},{:.3},{}",
            scenario.name,
            report.files_checked,
            report.dirs_checked,
            elapsed.as_secs_f64() * 1000.0,
            report.violations.len()
        );
        assert!(
            report.success,
            "scenario {} should be a clean measurement fixture: {:#?}",
            scenario.name, report.violations
        );
    }
}

#[derive(Clone, Copy)]
enum ScenarioKind {
    Sized { dirs: usize, files_per_dir: usize },
    Deep { depth: usize },
    Wide { dirs: usize },
    Ignored { dirs: usize, files_per_dir: usize },
    DirectChecks { dirs: usize },
    RuleHeavy { dirs: usize, files_per_dir: usize },
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    kind: ScenarioKind,
}

impl Scenario {
    fn sized(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::Sized {
                dirs,
                files_per_dir,
            },
        }
    }

    fn deep(name: &'static str, depth: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::Deep { depth },
        }
    }

    fn wide(name: &'static str, dirs: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::Wide { dirs },
        }
    }

    fn ignored(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::Ignored {
                dirs,
                files_per_dir,
            },
        }
    }

    fn direct_checks(name: &'static str, dirs: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::DirectChecks { dirs },
        }
    }

    fn rule_heavy(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::RuleHeavy {
                dirs,
                files_per_dir,
            },
        }
    }

    fn materialize(self, project: &TempDir) {
        match self.kind {
            ScenarioKind::Sized {
                dirs,
                files_per_dir,
            } => create_sized_project(project, dirs, files_per_dir),
            ScenarioKind::Deep { depth } => create_deep_project(project, depth),
            ScenarioKind::Wide { dirs } => create_wide_project(project, dirs),
            ScenarioKind::Ignored {
                dirs,
                files_per_dir,
            } => create_ignored_project(project, dirs, files_per_dir),
            ScenarioKind::DirectChecks { dirs } => create_direct_checks_project(project, dirs),
            ScenarioKind::RuleHeavy {
                dirs,
                files_per_dir,
            } => create_rule_heavy_project(project, dirs, files_per_dir),
        }
    }
}

fn base_config(files: &str, directories: &str, children: &str, exclude: &str) -> String {
    format!(
        r#"
structure:
  ./:
    files:
{files}
    directories:
{directories}
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
{children}
exclude:
  - ".assura/**"
{exclude}
"#
    )
}

fn create_sized_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_config(
        project,
        &base_config(
            "      naming: kebab-case",
            "      naming: kebab-case",
            "",
            "",
        ),
    );
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("dir-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            fs::write(
                dir.join(format!("file-{dir_index:04}-{file_index:04}.rs")),
                "",
            )
            .unwrap();
        }
    }
}

fn create_deep_project(project: &TempDir, depth: usize) {
    write_config(
        project,
        &base_config(
            "      naming: kebab-case",
            "      naming: kebab-case",
            "",
            "",
        ),
    );
    let mut current = project.path().to_path_buf();
    for depth_index in 0..depth {
        current = current.join(format!("level-{depth_index:04}"));
        fs::create_dir(&current).unwrap();
        fs::write(current.join(format!("file-{depth_index:04}.rs")), "").unwrap();
    }
}

fn create_wide_project(project: &TempDir, dirs: usize) {
    write_config(
        project,
        &base_config(
            "      naming: kebab-case",
            "      naming: kebab-case",
            "",
            "",
        ),
    );
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("wide-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("index.rs"), "").unwrap();
    }
}

fn create_ignored_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_config(
        project,
        &base_config(
            "      naming: kebab-case",
            "      naming: kebab-case",
            "",
            "  - \"generated/**\"",
        ),
    );
    fs::create_dir(project.path().join("generated")).unwrap();
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("generated/out_{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            fs::write(dir.join(format!("BAD_{file_index:04}.TMP")), "").unwrap();
        }
    }
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/index.rs"), "").unwrap();
}

fn create_direct_checks_project(project: &TempDir, dirs: usize) {
    let mut children = String::new();
    for dir_index in 0..dirs {
        children.push_str(&format!(
            r#"
      dir-{dir_index:04}/:
        files:
          exists:
            "*.rs": "1"
          allowed_patterns:
            - "*.rs"
          allow_extra: false
"#
        ));
    }
    write_config(
        project,
        &base_config(
            "      naming: kebab-case",
            "      naming: kebab-case",
            &children,
            "",
        ),
    );
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("dir-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("index.rs"), "").unwrap();
    }
}

fn create_rule_heavy_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    let mut patterns = String::new();
    for pattern_index in 0..80 {
        patterns.push_str(&format!(
            "        \"*.kind-{pattern_index:02}.ts\": kebab-case\n"
        ));
    }
    write_config(
        project,
        &base_config(
            &format!("      naming_patterns:\n{patterns}"),
            "      naming: kebab-case",
            "",
            "",
        ),
    );
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("rules-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            let kind = file_index % 80;
            fs::write(
                dir.join(format!(
                    "file-{dir_index:04}-{file_index:04}.kind-{kind:02}.ts"
                )),
                "",
            )
            .unwrap();
        }
    }
}

#[allow(dead_code)]
fn count_files(path: &Path) -> usize {
    fs::read_dir(path)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                count_files(&path)
            } else {
                1
            }
        })
        .sum()
}
