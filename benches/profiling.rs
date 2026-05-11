//! Profiling benchmark to identify bottlenecks in Assura validation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glob::Pattern;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::{Builder, TempDir};

use assura::cli::run_structure_check;
use assura::config::loader::ConfigLoader;
use assura::constraints::{
    CaseConvention, ConstraintConfig, ConstraintContext, ConstraintEngine, DirectoryConstraint,
    FileSizeConstraint, FileSizeLimit, FileSizeRule, NamingConstraint,
};

/// Create test project with many files
fn create_test_project(dir_count: usize, files_per_dir: usize) -> TempDir {
    let temp_dir = Builder::new().prefix("assura_profile_").tempdir().unwrap();
    let base_path = temp_dir.path();

    for d in 0..dir_count {
        let dir_path = base_path.join(format!("dir_{:04}", d));
        std::fs::create_dir(&dir_path).unwrap();

        for f in 0..files_per_dir {
            let filename = format!("file-{:04}-{:04}.txt", d, f);
            let file_path = dir_path.join(&filename);
            std::fs::write(&file_path, format!("Content {}", f)).unwrap();
        }
    }

    temp_dir
}

/// Benchmark just directory walking (no validation)
fn profile_walking(c: &mut Criterion) {
    let temp_dir = create_test_project(50, 100); // 5000 files

    c.bench_function("profile_walk_only_jwalk", |b| {
        b.iter(|| {
            let count = WalkDir::new(temp_dir.path())
                .parallelism(jwalk::Parallelism::RayonNewPool(0))
                .into_iter()
                .filter_map(|e| e.ok())
                .count();
            black_box(count);
        })
    });

    c.bench_function("profile_walk_only_sequential", |b| {
        b.iter(|| {
            let count = walkdir::WalkDir::new(temp_dir.path())
                .into_iter()
                .filter_map(|e| e.ok())
                .count();
            black_box(count);
        })
    });
}

/// Benchmark constraint engine overhead
fn profile_constraint_engine(c: &mut Criterion) {
    let temp_dir = create_test_project(50, 100);
    let entries: Vec<_> = WalkDir::new(temp_dir.path())
        .parallelism(jwalk::Parallelism::RayonNewPool(0))
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    // No constraints
    c.bench_function("profile_no_constraints", |b| {
        let engine = ConstraintEngine::new(ConstraintConfig::new());
        let context = ConstraintContext::new();
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });

    // 1 constraint
    c.bench_function("profile_1_constraint", |b| {
        let mut engine = ConstraintEngine::new(ConstraintConfig::new());
        let naming = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
        engine.register_constraint(Box::new(naming));
        let context = ConstraintContext::new();
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });

    // 2 constraints (non-recursive directory to avoid O(n²) behavior)
    c.bench_function("profile_2_constraints", |b| {
        let mut engine = ConstraintEngine::new(ConstraintConfig::new());
        let naming = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
        engine.register_constraint(Box::new(naming));
        let directory = DirectoryConstraint::new()
            .with_case_convention(CaseConvention::KebabCase)
            .with_config(assura::constraints::DirectoryValidationConfig::new().non_recursive());
        engine.register_constraint(Box::new(directory));
        let context = ConstraintContext::new();
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });

    // 3 constraints (non-recursive directory to avoid O(n²) behavior)
    c.bench_function("profile_3_constraints", |b| {
        let mut engine = ConstraintEngine::new(ConstraintConfig::new());
        let naming = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
        engine.register_constraint(Box::new(naming));
        let directory = DirectoryConstraint::new()
            .with_case_convention(CaseConvention::KebabCase)
            .with_config(assura::constraints::DirectoryValidationConfig::new().non_recursive());
        engine.register_constraint(Box::new(directory));
        let size = FileSizeConstraint::new()
            .add_rule(FileSizeRule::new("max_size").max_size(FileSizeLimit::Megabytes(1)));
        engine.register_constraint(Box::new(size));
        let context = ConstraintContext::new();
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });
}

/// Profile context creation overhead
fn profile_context_creation(c: &mut Criterion) {
    let temp_dir = create_test_project(50, 100);
    let entries: Vec<_> = WalkDir::new(temp_dir.path())
        .parallelism(jwalk::Parallelism::RayonNewPool(0))
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let mut engine = ConstraintEngine::new(ConstraintConfig::new());
    let naming = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
    engine.register_constraint(Box::new(naming));

    // Shared context
    c.bench_function("profile_shared_context", |b| {
        let context = ConstraintContext::new();
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });

    // New context per file (potential overhead)
    c.bench_function("profile_new_context_per_file", |b| {
        b.iter(|| {
            entries.par_iter().for_each(|entry| {
                let context = ConstraintContext::new();
                black_box(engine.validate(entry.path(), &context));
            });
        })
    });
}

/// Profile file size checking
fn profile_file_size_check(c: &mut Criterion) {
    let temp_dir = create_test_project(10, 10);
    let file_path = temp_dir.path().join("dir_0000/file-0000-0000.txt");

    c.bench_function("profile_file_metadata", |b| {
        b.iter(|| {
            let _ = black_box(std::fs::metadata(&file_path));
        })
    });
}

#[derive(Clone, Copy)]
enum StructureScenarioKind {
    Sized { dirs: usize, files_per_dir: usize },
    Deep { depth: usize },
    Wide { dirs: usize },
    Ignored { dirs: usize, files_per_dir: usize },
    DirectChecks { dirs: usize },
    RuleHeavy { dirs: usize, files_per_dir: usize },
}

#[derive(Clone, Copy)]
struct StructureScenario {
    name: &'static str,
    kind: StructureScenarioKind,
}

impl StructureScenario {
    fn sized(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::Sized {
                dirs,
                files_per_dir,
            },
        }
    }

    fn deep(name: &'static str, depth: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::Deep { depth },
        }
    }

    fn wide(name: &'static str, dirs: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::Wide { dirs },
        }
    }

    fn ignored(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::Ignored {
                dirs,
                files_per_dir,
            },
        }
    }

    fn direct_checks(name: &'static str, dirs: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::DirectChecks { dirs },
        }
    }

    fn rule_heavy(name: &'static str, dirs: usize, files_per_dir: usize) -> Self {
        Self {
            name,
            kind: StructureScenarioKind::RuleHeavy {
                dirs,
                files_per_dir,
            },
        }
    }

    fn materialize(self) -> TempDir {
        let temp_dir = Builder::new()
            .prefix("assura_structure_profile_")
            .tempdir()
            .unwrap();
        match self.kind {
            StructureScenarioKind::Sized {
                dirs,
                files_per_dir,
            } => create_structure_sized_project(&temp_dir, dirs, files_per_dir),
            StructureScenarioKind::Deep { depth } => {
                create_structure_deep_project(&temp_dir, depth)
            }
            StructureScenarioKind::Wide { dirs } => create_structure_wide_project(&temp_dir, dirs),
            StructureScenarioKind::Ignored {
                dirs,
                files_per_dir,
            } => create_structure_ignored_project(&temp_dir, dirs, files_per_dir),
            StructureScenarioKind::DirectChecks { dirs } => {
                create_structure_direct_checks_project(&temp_dir, dirs)
            }
            StructureScenarioKind::RuleHeavy {
                dirs,
                files_per_dir,
            } => create_structure_rule_heavy_project(&temp_dir, dirs, files_per_dir),
        }
        temp_dir
    }
}

fn structure_scenarios() -> [StructureScenario; 8] {
    [
        StructureScenario::sized("small", 8, 24),
        StructureScenario::sized("medium", 32, 80),
        StructureScenario::sized("large", 64, 160),
        StructureScenario::deep("deep_tree", 80),
        StructureScenario::wide("wide_tree", 800),
        StructureScenario::ignored("many_ignored_generated_dirs", 120, 30),
        StructureScenario::direct_checks("many_direct_content_checks", 160),
        StructureScenario::rule_heavy("many_wildcard_extension_path_rules", 120, 80),
    ]
}

fn write_structure_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn structure_base_config(files: &str, directories: &str, children: &str, exclude: &str) -> String {
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

fn create_structure_sized_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_structure_config(
        project,
        &structure_base_config(
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

fn create_structure_deep_project(project: &TempDir, depth: usize) {
    write_structure_config(
        project,
        &structure_base_config(
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

fn create_structure_wide_project(project: &TempDir, dirs: usize) {
    write_structure_config(
        project,
        &structure_base_config(
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

fn create_structure_ignored_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    write_structure_config(
        project,
        &structure_base_config(
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

fn create_structure_direct_checks_project(project: &TempDir, dirs: usize) {
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
    write_structure_config(
        project,
        &structure_base_config(
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

fn create_structure_rule_heavy_project(project: &TempDir, dirs: usize, files_per_dir: usize) {
    let mut patterns = String::new();
    for pattern_index in 0..80 {
        patterns.push_str(&format!(
            "        \"*.kind-{pattern_index:02}.ts\": kebab-case\n"
        ));
    }
    write_structure_config(
        project,
        &structure_base_config(
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

fn collect_structure_paths(project: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(project)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn count_structure_entries(project: &Path) -> usize {
    walkdir::WalkDir::new(project)
        .into_iter()
        .filter_map(Result::ok)
        .count()
}

fn count_structure_entries_pruned(project: &Path, excluded: &[Pattern]) -> usize {
    walkdir::WalkDir::new(project)
        .into_iter()
        .filter_entry(|entry| {
            let rel = entry.path().strip_prefix(project).unwrap_or(entry.path());
            let rel = rel.to_string_lossy();
            !excluded.iter().any(|pattern| pattern.matches(&rel))
        })
        .filter_map(Result::ok)
        .count()
}

fn count_direct_children_matching(project: &Path) -> usize {
    walkdir::WalkDir::new(project)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            fs::read_dir(entry.path())
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .filter(|child| child.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                        .filter(|child| child.file_name().to_string_lossy().ends_with(".rs"))
                        .count()
                })
                .unwrap_or(0)
        })
        .sum()
}

fn count_rule_heavy_pattern_matches(paths: &[PathBuf], patterns: &[Pattern]) -> usize {
    paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .filter(|name| patterns.iter().any(|pattern| pattern.matches(name)))
        .count()
}

fn count_rule_heavy_compile_each_match(paths: &[PathBuf], pattern_names: &[String]) -> usize {
    paths
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .filter(|name| {
            pattern_names.iter().any(|pattern| {
                Pattern::new(pattern)
                    .map(|compiled| compiled.matches(name))
                    .unwrap_or(false)
            })
        })
        .count()
}

/// Benchmark the current structure-first `assura check` path.
fn profile_structure_check_full(c: &mut Criterion) {
    let scenarios = structure_scenarios().map(|scenario| (scenario, scenario.materialize()));
    let mut group = c.benchmark_group("structure_check/full/run_structure_check");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));

    for (scenario, project) in &scenarios {
        let entries = count_structure_entries(project.path());
        group.throughput(Throughput::Elements(entries as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(scenario.name),
            project.path(),
            |b, path| {
                b.iter(|| {
                    let report =
                        run_structure_check(Some(path.to_path_buf()), None, false).unwrap();
                    black_box((report.files_checked, report.dirs_checked, report.success));
                })
            },
        );
    }

    group.finish();
}

/// Benchmark isolated costs that feed the structure-first check path.
fn profile_structure_check_attribution(c: &mut Criterion) {
    let large = StructureScenario::sized("large", 64, 160).materialize();
    let ignored = StructureScenario::ignored("many_ignored_generated_dirs", 120, 30).materialize();
    let direct = StructureScenario::direct_checks("many_direct_content_checks", 160).materialize();
    let rule_heavy =
        StructureScenario::rule_heavy("many_wildcard_extension_path_rules", 120, 80).materialize();

    let excluded = [
        Pattern::new(".assura").unwrap(),
        Pattern::new(".assura/**").unwrap(),
        Pattern::new("generated").unwrap(),
        Pattern::new("generated/**").unwrap(),
    ];
    let rule_heavy_paths = collect_structure_paths(rule_heavy.path());
    let rule_heavy_pattern_names: Vec<_> = (0..80)
        .map(|index| format!("*.kind-{index:02}.ts"))
        .collect();
    let rule_heavy_patterns: Vec<_> = (0..80)
        .map(|index| Pattern::new(&format!("*.kind-{index:02}.ts")).unwrap())
        .collect();

    let mut group = c.benchmark_group("structure_check/attribution");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(500));

    group.bench_function("config_load/large", |b| {
        let config_path = large.path().join(".assura/config.yml");
        b.iter(|| black_box(ConfigLoader::load(&config_path).unwrap()))
    });

    group.bench_function("traversal/walkdir_large", |b| {
        b.iter(|| black_box(count_structure_entries(large.path())))
    });

    group.bench_function("traversal_pruned/walkdir_ignored_generated", |b| {
        b.iter(|| black_box(count_structure_entries_pruned(ignored.path(), &excluded)))
    });

    group.bench_function("direct_count_reads/many_direct_content_checks", |b| {
        b.iter(|| black_box(count_direct_children_matching(direct.path())))
    });

    group.bench_function(
        "pattern_compile_each/many_wildcard_extension_path_rules",
        |b| {
            b.iter(|| {
                black_box(count_rule_heavy_compile_each_match(
                    &rule_heavy_paths,
                    &rule_heavy_pattern_names,
                ))
            })
        },
    );

    group.bench_function(
        "pattern_precompiled/many_wildcard_extension_path_rules",
        |b| {
            b.iter(|| {
                black_box(count_rule_heavy_pattern_matches(
                    &rule_heavy_paths,
                    &rule_heavy_patterns,
                ))
            })
        },
    );

    group.finish();
}

criterion_group!(
    profiling,
    profile_walking,
    profile_constraint_engine,
    profile_context_creation,
    profile_file_size_check,
    profile_structure_check_full,
    profile_structure_check_attribution,
);
criterion_main!(profiling);
