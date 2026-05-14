//! Current-product head-to-head comparison with LS-Lint 2.3.
//!
//! This benchmark compares Assura's public structure-first `assura check`
//! implementation with `@ls-lint/ls-lint@2.3.0` on identical temporary
//! fixtures.

use assura::cli::run_structure_check;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tempfile::{Builder, TempDir};

#[derive(Clone, Copy)]
enum ScenarioKind {
    Sized { dirs: usize, files_per_dir: usize },
    RuleHeavy { dirs: usize, files_per_dir: usize },
    Ignored { dirs: usize, files_per_dir: usize },
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

    fn rule_heavy(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: ScenarioKind::RuleHeavy {
                dirs,
                files_per_dir,
            },
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
}

fn scenarios() -> [Scenario; 5] {
    [
        Scenario::sized("small", 5, 10),
        Scenario::sized("medium", 20, 50),
        Scenario::sized("large", 50, 100),
        Scenario::rule_heavy("rule_heavy", 40, 50),
        Scenario::ignored("ignored_generated_heavy", 50, 20),
    ]
}

fn materialize_scenario(scenario: Scenario) -> TempDir {
    let project = Builder::new()
        .prefix("assura_lslint_compare_")
        .tempdir()
        .unwrap();
    match scenario.kind {
        ScenarioKind::Sized {
            dirs,
            files_per_dir,
        } => create_sized_project(&project, dirs, files_per_dir),
        ScenarioKind::RuleHeavy {
            dirs,
            files_per_dir,
        } => create_rule_heavy_project(&project, dirs, files_per_dir),
        ScenarioKind::Ignored {
            dirs,
            files_per_dir,
        } => create_ignored_project(&project, dirs, files_per_dir),
    }
    project
}

fn create_sized_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_configs(
        project,
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
    );

    for dir_index in 0..dirs {
        let dir = project.path().join(format!("pkg-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            fs::write(
                dir.join(format!("file-{dir_index:04}-{file_index:04}.ts")),
                "",
            )
            .unwrap();
        }
    }
}

fn create_rule_heavy_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    let mut assura_patterns = String::new();
    let mut ls_patterns = String::new();
    for pattern_index in 0..30 {
        assura_patterns.push_str(&format!(
            "        \"*.kind-{pattern_index:02}.ts\": kebab-case\n"
        ));
        ls_patterns.push_str(&format!("  .kind-{pattern_index:02}.ts: kebab-case\n"));
    }

    write_configs(
        project,
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
    );

    for dir_index in 0..dirs {
        let dir = project.path().join(format!("rules-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            let kind = file_index % 30;
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

fn create_ignored_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_configs(
        project,
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
    );

    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/index.ts"), "").unwrap();
    fs::create_dir(project.path().join("generated")).unwrap();
    for dir_index in 0..dirs {
        let dir = project.path().join(format!("generated/out-{dir_index:04}"));
        fs::create_dir(&dir).unwrap();
        for file_index in 0..files_per_dir {
            fs::write(dir.join(format!("BAD_{file_index:04}.ts")), "").unwrap();
        }
    }
}

fn write_configs(project: &TempDir, assura_config: &str, ls_lint_config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), assura_config).unwrap();
    fs::write(project.path().join(".ls-lint.yml"), ls_lint_config).unwrap();
}

fn count_entries(path: &Path) -> usize {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .count()
}

fn run_assura(path: &Path) {
    let report = run_structure_check(Some(path.to_path_buf()), None, false).unwrap();
    assert!(report.success, "fixture should pass Assura: {report:#?}");
    black_box(report);
}

fn ls_lint_available() -> bool {
    Command::new("npm")
        .args([
            "exec",
            "--yes",
            "--package",
            "@ls-lint/ls-lint@2.3.0",
            "--",
            "ls-lint",
            "--version",
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_ls_lint(path: &Path) {
    let output = Command::new("npm")
        .current_dir(path)
        .args([
            "exec",
            "--yes",
            "--package",
            "@ls-lint/ls-lint@2.3.0",
            "--",
            "ls-lint",
        ])
        .output()
        .expect("failed to run LS-Lint through npm");
    assert!(
        output.status.success(),
        "fixture should pass LS-Lint\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    black_box(output);
}

fn bench_current_product_comparison(c: &mut Criterion) {
    let fixtures: Vec<_> = scenarios()
        .into_iter()
        .map(|scenario| (scenario, materialize_scenario(scenario)))
        .collect();
    let has_ls_lint = ls_lint_available();

    let mut group = c.benchmark_group("current_product_lslint_2_3");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));

    for (scenario, fixture) in &fixtures {
        group.throughput(Throughput::Elements(count_entries(fixture.path()) as u64));
        group.bench_with_input(
            BenchmarkId::new("assura_check", scenario.name),
            fixture.path(),
            |b, path| b.iter(|| run_assura(path)),
        );

        if has_ls_lint {
            group.bench_with_input(
                BenchmarkId::new("ls_lint_2_3", scenario.name),
                fixture.path(),
                |b, path| b.iter(|| run_ls_lint(path)),
            );
        }
    }

    group.finish();
}

criterion_group!(comparison, bench_current_product_comparison);
criterion_main!(comparison);
