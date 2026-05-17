//! Generated stable fixtures for performance report measurement.

use super::fixture_io::{write_configs, write_file, write_lslint_compatible_configs};
use super::fixture_metadata::fixture_metadata;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub(super) struct FixtureScenario {
    pub(super) id: &'static str,
    pub(super) source_revision: &'static str,
    pub(super) rule_cohort: &'static str,
    pub(super) dirs: usize,
    pub(super) files_per_dir: usize,
    pub(super) kind: FixtureKind,
}

pub(in crate::cli::performance_report) struct MaterializedFixture {
    pub(super) root: PathBuf,
    pub(super) scenario: FixtureScenario,
    pub(super) metadata: FixtureMetadata,
}

pub(in crate::cli::performance_report) struct FixtureMetadata {
    pub(in crate::cli::performance_report) source_type: &'static str,
    pub(in crate::cli::performance_report) cohort: &'static str,
    pub(in crate::cli::performance_report) checked_file_count: usize,
    pub(in crate::cli::performance_report) ignored_file_count: usize,
    pub(in crate::cli::performance_report) directory_count: usize,
    pub(in crate::cli::performance_report) rule_count: usize,
    pub(in crate::cli::performance_report) rule_surface_summary: &'static str,
    pub(in crate::cli::performance_report) native_ls_lint_parity: bool,
    pub(in crate::cli::performance_report) assura_config_path: &'static str,
    pub(in crate::cli::performance_report) ls_lint_config_path: &'static str,
    pub(in crate::cli::performance_report) config_generation_method: &'static str,
    pub(in crate::cli::performance_report) shared_config_id: String,
    pub(in crate::cli::performance_report) expected_assura_exit_status: i32,
    pub(in crate::cli::performance_report) expected_ls_lint_exit_status: i32,
}

#[derive(Clone, Copy)]
pub(in crate::cli::performance_report) enum FixtureKind {
    Sized,
    RuleHeavy,
    IgnoredGenerated,
    SimpleLibrary,
    WebApp,
    MonorepoPackages,
    RuleHeavyRepo,
    IgnoredGeneratedHeavyRepo,
}

pub(super) fn scenarios() -> Vec<FixtureScenario> {
    vec![
        FixtureScenario {
            id: "simple_small",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "extension-dir-naming",
            dirs: 5,
            files_per_dir: 10,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "simple_medium",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "extension-dir-naming",
            dirs: 20,
            files_per_dir: 50,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "monorepo_large",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "extension-dir-naming",
            dirs: 50,
            files_per_dir: 100,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "rule_heavy",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "multi-extension-patterns",
            dirs: 40,
            files_per_dir: 50,
            kind: FixtureKind::RuleHeavy,
        },
        FixtureScenario {
            id: "ignored_generated_heavy",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "exclude-pruning",
            dirs: 50,
            files_per_dir: 20,
            kind: FixtureKind::IgnoredGenerated,
        },
        FixtureScenario {
            id: "simple_library",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "realistic-library",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::SimpleLibrary,
        },
        FixtureScenario {
            id: "web_app",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "realistic-frontend",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::WebApp,
        },
        FixtureScenario {
            id: "monorepo_packages",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "realistic-monorepo",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::MonorepoPackages,
        },
        FixtureScenario {
            id: "rule_heavy_repo",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "realistic-multi-extension-patterns",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::RuleHeavyRepo,
        },
        FixtureScenario {
            id: "ignored_generated_heavy_repo",
            source_revision: "generated-fixtures-v1",
            rule_cohort: "realistic-exclude-pruning",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::IgnoredGeneratedHeavyRepo,
        },
    ]
}

pub(super) fn materialize_fixture(
    scenario: FixtureScenario,
) -> Result<MaterializedFixture, String> {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "assura_perf_{}_{}_{}",
        scenario.id,
        std::process::id(),
        monotonic_nanos()
    ));
    fs::create_dir_all(&root).map_err(|error| format!("create {}: {error}", root.display()))?;

    match scenario.kind {
        FixtureKind::Sized => create_sized_project(&root, scenario.dirs, scenario.files_per_dir)?,
        FixtureKind::RuleHeavy => {
            create_rule_heavy_project(&root, scenario.dirs, scenario.files_per_dir)?
        }
        FixtureKind::IgnoredGenerated => {
            create_ignored_project(&root, scenario.dirs, scenario.files_per_dir)?
        }
        FixtureKind::SimpleLibrary => create_simple_library_project(&root)?,
        FixtureKind::WebApp => create_web_app_project(&root)?,
        FixtureKind::MonorepoPackages => create_monorepo_packages_project(&root)?,
        FixtureKind::RuleHeavyRepo => create_realistic_rule_heavy_project(&root)?,
        FixtureKind::IgnoredGeneratedHeavyRepo => create_ignored_generated_heavy_project(&root)?,
    }

    Ok(MaterializedFixture {
        scenario,
        metadata: fixture_metadata(scenario, &root)?,
        root,
    })
}

fn create_sized_project(root: &Path, dirs: usize, files_per_dir: usize) -> Result<(), String> {
    write_configs(
        root,
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
"#,
        r#"
ls:
  .dir: kebab-case
  .ts: kebab-case
ignore:
  - .assura
"#,
    )?;

    for dir_index in 0..dirs {
        let dir = root.join(format!("pkg-{dir_index:04}"));
        fs::create_dir(&dir).map_err(|error| format!("create {}: {error}", dir.display()))?;
        for file_index in 0..files_per_dir {
            fs::write(
                dir.join(format!("file-{dir_index:04}-{file_index:04}.ts")),
                "",
            )
            .map_err(|error| format!("write sized fixture file: {error}"))?;
        }
    }
    Ok(())
}

fn create_rule_heavy_project(root: &Path, dirs: usize, files_per_dir: usize) -> Result<(), String> {
    let mut assura_patterns = String::new();
    let mut ls_patterns = String::new();
    for pattern_index in 0..30 {
        assura_patterns.push_str(&format!(
            "        \"*.kind-{pattern_index:02}.ts\": kebab-case\n"
        ));
        ls_patterns.push_str(&format!("  .kind-{pattern_index:02}.ts: kebab-case\n"));
    }

    write_configs(
        root,
        &format!(
            r#"
structure:
  ./:
    files:
      naming_patterns:
{assura_patterns}
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
"#
        ),
        &format!(
            r#"
ls:
  .dir: kebab-case
{ls_patterns}
ignore:
  - .assura
"#
        ),
    )?;

    for dir_index in 0..dirs {
        let dir = root.join(format!("rules-{dir_index:04}"));
        fs::create_dir(&dir).map_err(|error| format!("create {}: {error}", dir.display()))?;
        for file_index in 0..files_per_dir {
            let kind = file_index % 30;
            fs::write(
                dir.join(format!(
                    "file-{dir_index:04}-{file_index:04}.kind-{kind:02}.ts"
                )),
                "",
            )
            .map_err(|error| format!("write rule-heavy fixture file: {error}"))?;
        }
    }
    Ok(())
}

fn create_ignored_project(root: &Path, dirs: usize, files_per_dir: usize) -> Result<(), String> {
    write_configs(
        root,
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
  - "generated/**"
"#,
        r#"
ls:
  .dir: kebab-case
  .ts: kebab-case
ignore:
  - .assura
  - generated
"#,
    )?;

    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    fs::write(root.join("src/index.ts"), "").map_err(|error| format!("write src: {error}"))?;
    fs::create_dir(root.join("generated")).map_err(|error| format!("create generated: {error}"))?;
    for dir_index in 0..dirs {
        let dir = root.join(format!("generated/out-{dir_index:04}"));
        fs::create_dir(&dir).map_err(|error| format!("create {}: {error}", dir.display()))?;
        for file_index in 0..files_per_dir {
            fs::write(dir.join(format!("BAD_{file_index:04}.ts")), "")
                .map_err(|error| format!("write generated fixture file: {error}"))?;
        }
    }
    Ok(())
}

fn create_simple_library_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura/**
  - target/**
ls:
  .dir: kebab-case
  .rs: snake_case
  .md: exists:1-3
  .test.ts: kebab-case
  src:
    .rs: snake_case
  tests:
    .rs: snake_case
"#,
    )?;
    write_file(root.join("README.md"), "# Library\n")?;
    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    fs::create_dir(root.join("tests")).map_err(|error| format!("create tests: {error}"))?;
    fs::create_dir(root.join("target")).map_err(|error| format!("create target: {error}"))?;
    write_file(root.join("src/lib.rs"), "pub fn library() {}\n")?;
    write_file(root.join("tests/smoke_test.rs"), "#[test] fn ok() {}\n")?;
    write_file(root.join("client-api.test.ts"), "export {};\n")?;
    write_file(root.join("target/BadName.rs"), "")?;
    Ok(())
}

fn create_web_app_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura/**
  - dist/**
ls:
  .dir: kebab-case
  .tsx: PascalCase
  .test.tsx: kebab-case
  .module.css: kebab-case
  .png: kebab-case
  src:
    .tsx: PascalCase
    .test.tsx: kebab-case
  public:
    .png: kebab-case
"#,
    )?;
    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    fs::create_dir(root.join("public")).map_err(|error| format!("create public: {error}"))?;
    fs::create_dir(root.join("dist")).map_err(|error| format!("create dist: {error}"))?;
    write_file(root.join("src/App.tsx"), "export function App() {}\n")?;
    write_file(root.join("src/app.test.tsx"), "export {};\n")?;
    write_file(root.join("src/theme.module.css"), ".root {}\n")?;
    write_file(root.join("public/logo-icon.png"), "")?;
    write_file(root.join("dist/BadName.tsx"), "")?;
    Ok(())
}

fn create_monorepo_packages_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura/**
  - packages/core/dist/**
  - packages/ui/dist/**
ls:
  .dir: kebab-case
  packages:
    core:
      .dir: kebab-case
      .ts: camelCase
      .md: exists:1-2
      src:
        .ts: camelCase
      tests:
        .test.ts: kebab-case
    ui:
      .dir: kebab-case
      .tsx: PascalCase
      .md: exists:1-2
      src:
        .tsx: PascalCase
      tests:
        .test.tsx: kebab-case
"#,
    )?;
    for package in ["core", "ui"] {
        fs::create_dir_all(root.join(format!("packages/{package}/src")))
            .map_err(|error| format!("create package src: {error}"))?;
        fs::create_dir(root.join(format!("packages/{package}/tests")))
            .map_err(|error| format!("create package tests: {error}"))?;
        fs::create_dir(root.join(format!("packages/{package}/dist")))
            .map_err(|error| format!("create package dist: {error}"))?;
        write_file(
            root.join(format!("packages/{package}/README.md")),
            "# Package\n",
        )?;
    }
    write_file(root.join("packages/core/src/indexFile.ts"), "")?;
    write_file(root.join("packages/core/tests/index-file.test.ts"), "")?;
    write_file(root.join("packages/core/dist/BadName.ts"), "")?;
    write_file(root.join("packages/ui/src/Button.tsx"), "")?;
    write_file(root.join("packages/ui/tests/button.test.tsx"), "")?;
    write_file(root.join("packages/ui/dist/bad-name.tsx"), "")?;
    Ok(())
}

fn create_realistic_rule_heavy_project(root: &Path) -> Result<(), String> {
    let mut rules = String::new();
    for index in 0..36 {
        rules.push_str(&format!("  .kind-{index:02}.ts: kebab-case\n"));
    }
    write_lslint_compatible_configs(
        root,
        &format!(
            r#"
ignore:
  - .assura/**
  - .ls-lint.yml
ls:
  .dir: kebab-case
  .*: kebab-case | snake_case
{rules}"#
        ),
    )?;
    for dir_index in 0..8 {
        let dir = root.join(format!("feature-{dir_index:02}"));
        fs::create_dir(&dir).map_err(|error| format!("create feature dir: {error}"))?;
        for file_index in 0..24 {
            let kind = file_index % 36;
            write_file(
                dir.join(format!(
                    "feature-{dir_index:02}-{file_index:02}.kind-{kind:02}.ts"
                )),
                "",
            )?;
        }
    }
    Ok(())
}

fn create_ignored_generated_heavy_project(root: &Path) -> Result<(), String> {
    write_lslint_compatible_configs(
        root,
        r#"
ignore:
  - .assura/**
  - generated/**
  - coverage/**
ls:
  .dir: kebab-case
  .ts: kebab-case
"#,
    )?;
    fs::create_dir(root.join("src")).map_err(|error| format!("create src: {error}"))?;
    write_file(root.join("src/index-file.ts"), "")?;
    for generated_root in ["generated", "coverage"] {
        for dir_index in 0..24 {
            let dir = root.join(format!("{generated_root}/out-{dir_index:02}"));
            fs::create_dir_all(&dir).map_err(|error| format!("create generated dir: {error}"))?;
            for file_index in 0..16 {
                write_file(dir.join(format!("BAD_{file_index:02}.ts")), "")?;
            }
        }
    }
    Ok(())
}

fn monotonic_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
