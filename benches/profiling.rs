//! Profiling benchmark to identify bottlenecks in Assura validation

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::path::Path;
use std::time::{Duration, Instant};
use tempfile::{TempDir, Builder};

use assura::constraints::{
    CaseConvention, ConstraintContext, ConstraintEngine, ConstraintConfig,
    DirectoryConstraint, NamingConstraint, FileSizeConstraint, FileSizeRule, FileSizeLimit,
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
        let naming = NamingConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);
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
        let naming = NamingConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);
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
        let naming = NamingConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);
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
    let naming = NamingConstraint::new()
        .with_case_convention(CaseConvention::KebabCase);
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
            black_box(std::fs::metadata(&file_path));
        })
    });
}

criterion_group!(
    profiling,
    profile_walking,
    profile_constraint_engine,
    profile_context_creation,
    profile_file_size_check,
);
criterion_main!(profiling);
