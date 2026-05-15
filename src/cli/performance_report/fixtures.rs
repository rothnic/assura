//! Generated stable fixtures for performance report measurement.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy)]
pub(super) struct FixtureScenario {
    pub(super) id: &'static str,
    pub(super) source_revision: &'static str,
    pub(super) cohort: &'static str,
    pub(super) rule_cohort: &'static str,
    pub(super) dirs: usize,
    pub(super) files_per_dir: usize,
    pub(super) kind: FixtureKind,
}

#[derive(Clone, Copy)]
pub(super) enum FixtureKind {
    Sized,
    RuleHeavy,
    IgnoredGenerated,
}

pub(super) fn scenarios() -> [FixtureScenario; 5] {
    [
        FixtureScenario {
            id: "simple_small",
            source_revision: "generated-fixtures-v1",
            cohort: "stable-baseline",
            rule_cohort: "extension-dir-naming",
            dirs: 5,
            files_per_dir: 10,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "simple_medium",
            source_revision: "generated-fixtures-v1",
            cohort: "stable-baseline",
            rule_cohort: "extension-dir-naming",
            dirs: 20,
            files_per_dir: 50,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "monorepo_large",
            source_revision: "generated-fixtures-v1",
            cohort: "stable-baseline",
            rule_cohort: "extension-dir-naming",
            dirs: 50,
            files_per_dir: 100,
            kind: FixtureKind::Sized,
        },
        FixtureScenario {
            id: "rule_heavy",
            source_revision: "generated-fixtures-v1",
            cohort: "stable-baseline",
            rule_cohort: "multi-extension-patterns",
            dirs: 40,
            files_per_dir: 50,
            kind: FixtureKind::RuleHeavy,
        },
        FixtureScenario {
            id: "ignored_generated_heavy",
            source_revision: "generated-fixtures-v1",
            cohort: "stable-baseline",
            rule_cohort: "exclude-pruning",
            dirs: 50,
            files_per_dir: 20,
            kind: FixtureKind::IgnoredGenerated,
        },
    ]
}

pub(super) fn materialize_fixture(scenario: FixtureScenario) -> Result<PathBuf, String> {
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
    }

    Ok(root)
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

fn write_configs(root: &Path, assura_config: &str, ls_lint_config: &str) -> Result<(), String> {
    let assura_dir = root.join(".assura");
    fs::create_dir_all(&assura_dir)
        .map_err(|error| format!("create {}: {error}", assura_dir.display()))?;
    fs::write(assura_dir.join("config.yml"), assura_config)
        .map_err(|error| format!("write Assura config: {error}"))?;
    fs::write(root.join(".ls-lint.yml"), ls_lint_config)
        .map_err(|error| format!("write LS-Lint config: {error}"))?;
    Ok(())
}

fn monotonic_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
