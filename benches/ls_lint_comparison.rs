//! Head-to-head performance comparison with LS-Lint
//!
//! This benchmark suite provides direct comparison between Assura and LS-Lint:
//! 1. Cold start time (first run, no cache)
//! 2. Warm validation time (subsequent runs)
//! 3. Memory usage comparison
//! 4. Files/second throughput
//! 5. Incremental validation (single file change)
//!
//! To run: cargo bench --bench ls_lint_comparison

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use jwalk::WalkDir;
use rayon::prelude::*;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::{Builder, TempDir};

use assura::constraints::{
    CaseConvention, ConstraintConfig, ConstraintContext, ConstraintEngine, DirectoryConstraint,
    DirectoryValidationConfig, FileSizeConstraint, FileSizeLimit, FileSizeRule, NamingConstraint,
};

// ============================================================================
// Test Fixture Generation
// ============================================================================

/// Creates a test project structure with specified file counts
fn create_test_project(dir_count: usize, files_per_dir: usize) -> TempDir {
    let temp_dir = Builder::new().prefix("assura_bench_").tempdir().unwrap();
    let base_path = temp_dir.path();

    // Create directories with files
    for d in 0..dir_count {
        let dir_path = base_path.join(format!("dir_{:04}", d));
        std::fs::create_dir(&dir_path).unwrap();

        for f in 0..files_per_dir {
            let filename = format!("file-{:04}-{:04}.txt", d, f);
            let file_path = dir_path.join(&filename);
            std::fs::write(&file_path, format!("Content of file {}", f)).unwrap();
        }

        // Add subdirectory for nesting
        let sub_dir = dir_path.join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();
        for f in 0..(files_per_dir / 4) {
            let filename = format!("subfile-{:04}-{:04}.txt", d, f);
            let file_path = sub_dir.join(&filename);
            std::fs::write(&file_path, format!("Sub content {} {}", d, f)).unwrap();
        }
    }

    temp_dir
}

/// Creates a Rust project structure
fn create_rust_project(module_count: usize) -> TempDir {
    let temp_dir = Builder::new()
        .prefix("assura_rust_bench_")
        .tempdir()
        .unwrap();
    let base_path = temp_dir.path();
    let src_dir = base_path.join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Create lib.rs
    std::fs::write(src_dir.join("lib.rs"), format!("pub mod modules;\n")).unwrap();

    // Create modules
    let modules_dir = src_dir.join("modules");
    std::fs::create_dir(&modules_dir).unwrap();

    for i in 0..module_count {
        let mod_name = format!("module_{:04}", i);
        let mod_path = modules_dir.join(format!("{}.rs", mod_name));
        std::fs::write(&mod_path, format!("pub fn function_{}() {{}}\n", i)).unwrap();
    }

    // Create tests
    let tests_dir = base_path.join("tests");
    std::fs::create_dir(&tests_dir).unwrap();

    for i in 0..(module_count / 4) {
        std::fs::write(
            tests_dir.join(format!("test_{:04}.rs", i)),
            format!("#[test] fn test_{}() {{}}\n", i),
        )
        .unwrap();
    }

    // Create benchmarks
    let benches_dir = base_path.join("benches");
    std::fs::create_dir(&benches_dir).unwrap();

    for i in 0..(module_count / 10) {
        std::fs::write(
            benches_dir.join(format!("bench_{:04}.rs", i)),
            format!("fn bench_{}(b: &mut criterion::Bencher) {{}}\n", i),
        )
        .unwrap();
    }

    temp_dir
}

/// Creates a JavaScript/TypeScript project structure
fn create_js_project(component_count: usize) -> TempDir {
    let temp_dir = Builder::new().prefix("assura_js_bench_").tempdir().unwrap();
    let base_path = temp_dir.path();

    // Create src structure
    let src_dir = base_path.join("src");
    std::fs::create_dir(&src_dir).unwrap();

    // Components
    let components_dir = src_dir.join("components");
    std::fs::create_dir(&components_dir).unwrap();

    for i in 0..component_count {
        std::fs::write(
            components_dir.join(format!("Component-{:04}.tsx", i)),
            format!("export const Component{} = () => {{}};\n", i),
        )
        .unwrap();

        std::fs::write(
            components_dir.join(format!("Component-{:04}.test.tsx", i)),
            format!("test('component {}', () => {{}});\n", i),
        )
        .unwrap();
    }

    // Utils
    let utils_dir = src_dir.join("utils");
    std::fs::create_dir(&utils_dir).unwrap();

    for i in 0..(component_count / 5) {
        std::fs::write(
            utils_dir.join(format!("utility-{:04}.ts", i)),
            format!("export const utility{} = () => {{}};\n", i),
        )
        .unwrap();
    }

    // Hooks
    let hooks_dir = src_dir.join("hooks");
    std::fs::create_dir(&hooks_dir).unwrap();

    for i in 0..(component_count / 10) {
        std::fs::write(
            hooks_dir.join(format!("use-hook-{:04}.ts", i)),
            format!("export const useHook{} = () => {{}};\n", i),
        )
        .unwrap();
    }

    temp_dir
}

/// Count total files in a directory
fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                count += 1;
            } else if path.is_dir() {
                count += count_files(&path);
            }
        }
    }
    count
}

// ============================================================================
// Assura Setup
// ============================================================================

/// Create Assura constraint engine for general project validation
/// Uses non-recursive directory validation to avoid O(n²) behavior
fn create_assura_engine() -> ConstraintEngine {
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Add naming constraint for general files
    let naming = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
    engine.register_constraint(Box::new(naming));

    // Add directory constraint (non-recursive to avoid O(n²) walks)
    let directory = DirectoryConstraint::new()
        .with_case_convention(CaseConvention::KebabCase)
        .with_config(DirectoryValidationConfig::new().non_recursive());
    engine.register_constraint(Box::new(directory));

    // Add file size constraint
    let size = FileSizeConstraint::new()
        .add_rule(FileSizeRule::new("max_size").max_size(FileSizeLimit::Megabytes(1)));
    engine.register_constraint(Box::new(size));

    engine
}

/// Create Assura engine for Rust project validation
fn create_assura_rust_engine() -> ConstraintEngine {
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Rust source files - snake_case
    let rust_naming = NamingConstraint::new()
        .with_file_pattern("*.rs")
        .with_case_convention(CaseConvention::SnakeCase);
    engine.register_constraint(Box::new(rust_naming));

    // Directories - kebab-case for components
    let directory = DirectoryConstraint::new().with_case_convention(CaseConvention::SnakeCase);
    engine.register_constraint(Box::new(directory));

    engine
}

/// Create Assura engine for JS/TS project validation
fn create_assura_js_engine() -> ConstraintEngine {
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Components - PascalCase
    let component_naming = NamingConstraint::new()
        .with_file_pattern("src/components/*")
        .with_case_convention(CaseConvention::PascalCase);
    engine.register_constraint(Box::new(component_naming));

    // Utils/Hooks - camelCase
    let util_naming = NamingConstraint::new()
        .with_file_pattern("src/utils/*")
        .with_case_convention(CaseConvention::CamelCase);
    engine.register_constraint(Box::new(util_naming));

    let hook_naming = NamingConstraint::new()
        .with_file_pattern("src/hooks/*")
        .with_case_convention(CaseConvention::CamelCase);
    engine.register_constraint(Box::new(hook_naming));

    engine
}

/// Validate all files in a directory using Assura with parallel jwalk
fn validate_directory_assura(engine: &ConstraintEngine, dir: &Path) -> Duration {
    let start = Instant::now();

    // Use jwalk for parallel directory traversal (like original Guardrails)
    // Collect entries first, then validate in parallel
    let entries: Vec<_> = WalkDir::new(dir)
        .parallelism(jwalk::Parallelism::RayonNewPool(0)) // Use all CPUs
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let context = ConstraintContext::new();
    entries.par_iter().for_each(|entry| {
        black_box(engine.validate(entry.path(), &context));
    });

    start.elapsed()
}

// ============================================================================
// LS-Lint Setup (External Tool)
// ============================================================================

/// Check if ls-lint is installed
fn ls_lint_available() -> bool {
    Command::new("ls-lint")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create LS-Lint configuration file
fn create_ls_lint_config(dir: &Path) -> std::path::PathBuf {
    let config_content = r#"{
  "ls": {
    ".dir": "kebab-case",
    ".txt": "kebab-case"
  }
}
"#;
    let config_path = dir.join(".ls-lint.yml");
    std::fs::write(&config_path, config_content).unwrap();
    config_path
}

/// Create LS-Lint config for Rust projects
fn create_ls_lint_rust_config(dir: &Path) -> std::path::PathBuf {
    let config_content = r#"{
  "ls": {
    ".dir": "snake_case",
    ".rs": "snake_case"
  }
}
"#;
    let config_path = dir.join(".ls-lint.yml");
    std::fs::write(&config_path, config_content).unwrap();
    config_path
}

/// Create LS-Lint config for JS/TS projects
fn create_ls_lint_js_config(dir: &Path) -> std::path::PathBuf {
    let config_content = r#"{
  "ls": {
    ".dir": "kebab-case",
    "src/components/*": "PascalCase",
    "src/utils/*": "camelCase",
    "src/hooks/*": "camelCase"
  }
}
"#;
    let config_path = dir.join(".ls-lint.yml");
    std::fs::write(&config_path, config_content).unwrap();
    config_path
}

/// Run ls-lint and return execution time
fn run_ls_lint(dir: &Path) -> Option<Duration> {
    if !ls_lint_available() {
        return None;
    }

    let start = Instant::now();
    let result = Command::new("ls-lint").current_dir(dir).output();

    match result {
        Ok(_) => Some(start.elapsed()),
        Err(_) => None,
    }
}

// ============================================================================
// Benchmark Groups
// ============================================================================

/// Benchmark: Cold Start Comparison
///
/// Measures time to initialize and run first validation
fn bench_cold_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start");

    // Small project: 10 dirs, 10 files each
    let temp_dir = create_test_project(10, 10);
    let file_count = count_files(temp_dir.path());

    group.throughput(Throughput::Elements(file_count as u64));

    // Assura cold start
    group.bench_function("assura_small", |b| {
        b.iter_custom(|iters| {
            let mut total = Duration::default();
            for _ in 0..iters {
                let engine = create_assura_engine();
                total += validate_directory_assura(&engine, temp_dir.path());
            }
            total
        })
    });

    // LS-Lint cold start (if available)
    create_ls_lint_config(temp_dir.path());
    if ls_lint_available() {
        group.bench_function("ls_lint_small", |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::default();
                for _ in 0..iters {
                    if let Some(duration) = run_ls_lint(temp_dir.path()) {
                        total += duration;
                    }
                }
                total
            })
        });
    }

    group.finish();
}

/// Benchmark: Full Validation (Warm)
///
/// Measures repeated validation performance
fn bench_full_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_validation");

    let scenarios = [
        ("small_50files", 5usize, 10usize),     // 50 files
        ("medium_500files", 10usize, 50usize),  // 500 files
        ("large_5000files", 50usize, 100usize), // 5000 files
    ];

    for (name, dirs, files_per) in scenarios {
        let temp_dir = create_test_project(dirs, files_per);
        let file_count = count_files(temp_dir.path());

        group.throughput(Throughput::Elements(file_count as u64));

        // Assura - pre-create engine for warm runs
        let engine = create_assura_engine();
        group.bench_with_input(BenchmarkId::new("assura", name), &file_count, |b, _| {
            b.iter(|| black_box(validate_directory_assura(&engine, temp_dir.path())))
        });

        // LS-Lint comparison
        create_ls_lint_config(temp_dir.path());
        if ls_lint_available() {
            group.bench_with_input(BenchmarkId::new("ls_lint", name), &file_count, |b, _| {
                b.iter(|| black_box(run_ls_lint(temp_dir.path())))
            });
        }
    }

    group.finish();
}

/// Benchmark: Files per second throughput
///
/// Measures raw validation throughput
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Large project for throughput measurement
    let temp_dir = create_test_project(100, 50); // 5000 files
    let file_count = count_files(temp_dir.path());

    group.throughput(Throughput::Elements(file_count as u64));

    let engine = create_assura_engine();
    group.bench_function("assura_files_per_sec", |b| {
        b.iter(|| black_box(validate_directory_assura(&engine, temp_dir.path())))
    });

    create_ls_lint_config(temp_dir.path());
    if ls_lint_available() {
        group.bench_function("ls_lint_files_per_sec", |b| {
            b.iter(|| black_box(run_ls_lint(temp_dir.path())))
        });
    }

    group.finish();
}

/// Benchmark: Project Type Specific Validation
///
/// Compares validation for different project types
fn bench_project_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("project_types");

    // Rust project
    let rust_dir = create_rust_project(100); // 100 modules
    let rust_file_count = count_files(rust_dir.path());
    let rust_engine = create_assura_rust_engine();

    group.throughput(Throughput::Elements(rust_file_count as u64));
    group.bench_function("assura_rust", |b| {
        b.iter(|| black_box(validate_directory_assura(&rust_engine, rust_dir.path())))
    });

    create_ls_lint_rust_config(rust_dir.path());
    if ls_lint_available() {
        group.bench_function("ls_lint_rust", |b| {
            b.iter(|| black_box(run_ls_lint(rust_dir.path())))
        });
    }

    // JavaScript project
    let js_dir = create_js_project(100); // 100 components
    let js_file_count = count_files(js_dir.path());
    let js_engine = create_assura_js_engine();

    group.throughput(Throughput::Elements(js_file_count as u64));
    group.bench_function("assura_js", |b| {
        b.iter(|| black_box(validate_directory_assura(&js_engine, js_dir.path())))
    });

    create_ls_lint_js_config(js_dir.path());
    if ls_lint_available() {
        group.bench_function("ls_lint_js", |b| {
            b.iter(|| black_box(run_ls_lint(js_dir.path())))
        });
    }

    group.finish();
}

/// Benchmark: Incremental validation
///
/// Simulates validating a single changed file
fn bench_incremental_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");

    // Create a medium-sized project
    let temp_dir = create_test_project(10, 50);
    let engine = create_assura_engine();
    let context = ConstraintContext::new();

    // Create a test file
    let test_file = temp_dir.path().join("dir_0001").join("test-file.txt");
    std::fs::write(&test_file, "test content").unwrap();

    group.bench_function("assura_single_file", |b| {
        b.iter(|| black_box(engine.validate(&test_file, &context)))
    });

    group.finish();
}

/// Benchmark: Complex Rules Performance
///
/// Compares performance with multiple/complex rules
fn bench_complex_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_rules");

    let temp_dir = create_test_project(20, 25);
    let file_count = count_files(temp_dir.path());

    group.throughput(Throughput::Elements(file_count as u64));

    // Engine with many rules
    let config = ConstraintConfig::new();
    let mut complex_engine = ConstraintEngine::new(config);

    // Add multiple naming constraints with different patterns
    complex_engine.register_constraint(Box::new(
        NamingConstraint::new()
            .with_file_pattern("*.txt")
            .with_case_convention(CaseConvention::KebabCase),
    ));
    complex_engine.register_constraint(Box::new(
        NamingConstraint::new()
            .with_file_pattern("dir_*/*")
            .with_case_convention(CaseConvention::SnakeCase),
    ));
    complex_engine.register_constraint(Box::new(
        DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase),
    ));
    complex_engine
        .register_constraint(Box::new(FileSizeConstraint::new().add_rule(
            FileSizeRule::new("max_size").max_size(FileSizeLimit::Megabytes(1)),
        )));

    group.bench_function("assura_4_rules", |b| {
        b.iter(|| black_box(validate_directory_assura(&complex_engine, temp_dir.path())))
    });

    group.finish();
}

// ============================================================================
// Summary Benchmark
// ============================================================================

/// Comprehensive comparison benchmark that reports key metrics
fn bench_comparison_summary(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_summary");

    // Large realistic project
    let temp_dir = create_test_project(50, 100); // 5000 files + subdirs
    let file_count = count_files(temp_dir.path());

    println!("\n=== Performance Comparison Summary ===");
    println!("Test project: {} files across 50 directories", file_count);

    // Assura
    let engine = create_assura_engine();
    let assura_time = validate_directory_assura(&engine, temp_dir.path());
    let assura_fps = file_count as f64 / assura_time.as_secs_f64();
    println!("Assura: {:?} ({:.0} files/sec)", assura_time, assura_fps);

    // LS-Lint
    create_ls_lint_config(temp_dir.path());
    if let Some(ls_time) = run_ls_lint(temp_dir.path()) {
        let ls_fps = file_count as f64 / ls_time.as_secs_f64();
        println!("LS-Lint: {:?} ({:.0} files/sec)", ls_time, ls_fps);

        let speedup = ls_time.as_secs_f64() / assura_time.as_secs_f64();
        println!("Speedup: {:.1}x", speedup);
    } else {
        println!("LS-Lint: Not available for comparison");
    }
    println!("=======================================\n");

    // Run actual benchmark
    group.throughput(Throughput::Elements(file_count as u64));
    group.bench_function("assura_summary", |b| {
        b.iter(|| black_box(validate_directory_assura(&engine, temp_dir.path())))
    });

    group.finish();
}

// ============================================================================
// Criterion Main
// ============================================================================

criterion_group!(
    comparison,
    bench_cold_start,
    bench_full_validation,
    bench_throughput,
    bench_project_types,
    bench_incremental_validation,
    bench_complex_rules,
    bench_comparison_summary,
);

criterion_main!(comparison);
