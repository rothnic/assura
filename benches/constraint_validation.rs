//! Constraint Validation Performance Benchmarks
//!
//! Measures the performance of constraint validation operations
//! to ensure we meet the 2x target over LS-Lint.
//!
//! To run: cargo bench --bench constraint_validation

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use tempfile::{Builder, TempDir};

use assura::constraints::{
    CaseConvention, ConstraintContext, FileSizeConstraint, FileSizeLimit, FileSizeRule,
    NamingConstraint, Severity,
};
use assura::Constraint;

/// Creates a temp directory with files for benchmarking
fn setup_test_files(count: usize) -> TempDir {
    let temp_dir = Builder::new().prefix("assura_bench_").tempdir().unwrap();
    let base_path = temp_dir.path();

    for i in 0..count {
        // Mix of valid and invalid naming
        let filename = if i % 2 == 0 {
            format!("valid_file_{}.rs", i)
        } else {
            format!("InvalidFile{}.rs", i)
        };
        std::fs::write(base_path.join(&filename), format!("content {}", i)).unwrap();
    }

    temp_dir
}

/// Benchmark naming convention validation
fn bench_naming_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("naming_validation");

    for file_count in [10, 100, 1000].iter() {
        let temp_dir = setup_test_files(*file_count);
        let base_path = temp_dir.path();

        group.throughput(Throughput::Elements(*file_count as u64));

        // Benchmark PascalCase validation
        group.bench_with_input(
            BenchmarkId::new("pascal_case", file_count),
            file_count,
            |b, _| {
                let constraint =
                    NamingConstraint::new().with_case_convention(CaseConvention::PascalCase);
                let context = ConstraintContext::new();

                b.iter(|| {
                    for i in 0..*file_count {
                        let filename = if i % 2 == 0 {
                            format!("valid_file_{}.rs", i)
                        } else {
                            format!("InvalidFile{}.rs", i)
                        };
                        let path = base_path.join(&filename);
                        let _ = black_box(constraint.validate(&path, &context));
                    }
                });
            },
        );

        // Benchmark SnakeCase validation
        group.bench_with_input(
            BenchmarkId::new("snake_case", file_count),
            file_count,
            |b, _| {
                let constraint =
                    NamingConstraint::new().with_case_convention(CaseConvention::SnakeCase);
                let context = ConstraintContext::new();

                b.iter(|| {
                    for i in 0..*file_count {
                        let filename = if i % 2 == 0 {
                            format!("valid_file_{}.rs", i)
                        } else {
                            format!("InvalidFile{}.rs", i)
                        };
                        let path = base_path.join(&filename);
                        let _ = black_box(constraint.validate(&path, &context));
                    }
                });
            },
        );

        // Benchmark CamelCase validation
        group.bench_with_input(
            BenchmarkId::new("camel_case", file_count),
            file_count,
            |b, _| {
                let constraint =
                    NamingConstraint::new().with_case_convention(CaseConvention::CamelCase);
                let context = ConstraintContext::new();

                b.iter(|| {
                    for i in 0..*file_count {
                        let filename = if i % 2 == 0 {
                            format!("validFile{}.rs", i)
                        } else {
                            format!("InvalidFile{}.rs", i)
                        };
                        let path = base_path.join(&filename);
                        let _ = black_box(constraint.validate(&path, &context));
                    }
                });
            },
        );

        // Benchmark KebabCase validation
        group.bench_with_input(
            BenchmarkId::new("kebab_case", file_count),
            file_count,
            |b, _| {
                let constraint =
                    NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
                let context = ConstraintContext::new();

                b.iter(|| {
                    for i in 0..*file_count {
                        let filename = if i % 2 == 0 {
                            format!("valid-file-{}.rs", i)
                        } else {
                            format!("InvalidFile{}.rs", i)
                        };
                        let path = base_path.join(&filename);
                        let _ = black_box(constraint.validate(&path, &context));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark file size validation
fn bench_file_size_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_size_validation");

    for file_count in [10, 100].iter() {
        let temp_dir = Builder::new().prefix("assura_bench_").tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create files of varying sizes
        for i in 0..*file_count {
            let size = if i % 2 == 0 { 100 } else { 2000 }; // 100B or 2KB
            let content = vec![0u8; size];
            std::fs::write(base_path.join(format!("file_{}.txt", i)), content).unwrap();
        }

        group.throughput(Throughput::Elements(*file_count as u64));

        group.bench_with_input(
            BenchmarkId::new("max_1kb", file_count),
            file_count,
            |b, _| {
                let constraint = FileSizeConstraint::new().add_rule(
                    FileSizeRule::new("max_size")
                        .max_size(FileSizeLimit::Kilobytes(1))
                        .with_severity(Severity::High),
                );
                let context = ConstraintContext::new();

                b.iter(|| {
                    for i in 0..*file_count {
                        let path = base_path.join(format!("file_{}.txt", i));
                        let _ = black_box(constraint.validate(&path, &context));
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark all case conventions comparison
fn bench_all_conventions(c: &mut Criterion) {
    let mut group = c.benchmark_group("all_conventions");

    let conventions = vec![
        ("PascalCase", CaseConvention::PascalCase),
        ("camelCase", CaseConvention::CamelCase),
        ("snake_case", CaseConvention::SnakeCase),
        ("kebab-case", CaseConvention::KebabCase),
        ("SCREAMING_SNAKE", CaseConvention::ScreamingSnakeCase),
        ("lowercase", CaseConvention::LowerCase),
        ("UPPERCASE", CaseConvention::UpperCase),
    ];

    // Test each convention with 100 files
    let file_count = 100;
    let temp_dir = setup_test_files(file_count);
    let base_path = temp_dir.path();

    group.throughput(Throughput::Elements(file_count as u64));

    for (name, convention) in conventions {
        group.bench_with_input(BenchmarkId::new(name, file_count), &file_count, |b, _| {
            let constraint = NamingConstraint::new().with_case_convention(convention);
            let context = ConstraintContext::new();

            b.iter(|| {
                for i in 0..file_count {
                    let filename = format!("test_file_{}.rs", i);
                    let path = base_path.join(&filename);
                    let _ = black_box(constraint.validate(&path, &context));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark cold start vs warm validation
fn bench_cold_vs_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_vs_warm");

    let file_count = 100;
    let temp_dir = setup_test_files(file_count);
    let base_path = temp_dir.path();

    // Cold start: create new constraint each time
    group.bench_function("cold_start", |b| {
        b.iter(|| {
            let constraint =
                NamingConstraint::new().with_case_convention(CaseConvention::SnakeCase);
            let context = ConstraintContext::new();

            for i in 0..file_count {
                let filename = format!("test_file_{}.rs", i);
                let path = base_path.join(&filename);
                let _ = black_box(constraint.validate(&path, &context));
            }
        });
    });

    // Warm validation: reuse constraint
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::SnakeCase);
    let context = ConstraintContext::new();

    group.bench_function("warm_validation", |b| {
        b.iter(|| {
            for i in 0..file_count {
                let filename = format!("test_file_{}.rs", i);
                let path = base_path.join(&filename);
                let _ = black_box(constraint.validate(&path, &context));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_naming_validation,
    bench_file_size_validation,
    bench_all_conventions,
    bench_cold_vs_warm
);
criterion_main!(benches);
