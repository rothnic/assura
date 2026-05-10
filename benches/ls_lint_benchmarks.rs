//! Performance benchmarks for LS-Lint parity features
//!
//! This benchmark suite provides comprehensive performance analysis of:
//! 1. Case convention validation performance (all 12 conventions)
//! 2. Directory validation with various directory depths
//! 3. Multi-part extension pattern matching
//! 4. Multiple rule syntax parsing and validation
//! 5. Path-specific rule matching with glob patterns
//! 6. Full constraint validation scenarios
//!
//! Each benchmark measures execution time and provides comparative metrics
//! to identify performance characteristics and bottlenecks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::path::Path;
use tempfile::TempDir;

use assura::constraints::{
    CaseConvention, ComplexExtension, Constraint, ConstraintConfig, ConstraintContext,
    ConstraintEngine, DirectoryConstraint, DirectoryValidationConfig, ExtensionPattern,
    ExtensionRule, FileSizeConstraint, FileSizeLimit, FileSizeRule, MultiPartExtensionRule,
    MultipleRuleSyntax, NamingConstraint, PathRule, PathRuleConfig, Severity,
};

// ============================================================================
// 1. Case Convention Validation Performance (All 12 Conventions)
// ============================================================================

/// Benchmark all 12 case conventions with various input sizes
///
/// This benchmark compares the performance characteristics of each case convention
/// validation across different input complexities.
fn bench_case_conventions(c: &mut Criterion) {
    let mut group = c.benchmark_group("case_convention_validation");

    // Test strings of varying complexity
    let test_cases = [
        ("short_4", "file"),
        ("short_10", "filename"),
        ("medium_20", "my_test_file_name"),
        ("long_50", "very_long_file_name_with_many_parts_and_words"),
        ("complex_35", "My-Complex_Test.file.NAME123"),
    ];

    // All 12 case conventions
    let conventions = [
        ("lowercase", CaseConvention::LowerCase),
        ("UPPERCASE", CaseConvention::UpperCase),
        ("snake_case", CaseConvention::SnakeCase),
        ("camelCase", CaseConvention::CamelCase),
        ("PascalCase", CaseConvention::PascalCase),
        ("kebab-case", CaseConvention::KebabCase),
        ("SCREAMING_SNAKE", CaseConvention::ScreamingSnakeCase),
        ("dot.case", CaseConvention::DotCase),
        ("flatcase", CaseConvention::FlatCase),
        ("FLATCASE", CaseConvention::ScreamingFlatCase),
        ("COBOL-CASE", CaseConvention::CobolCase),
        ("Train-Case", CaseConvention::TrainCase),
    ];

    for (case_name, test_str) in test_cases {
        group.throughput(Throughput::Bytes(test_str.len() as u64));

        for (conv_name, convention) in conventions {
            group.bench_with_input(BenchmarkId::new(conv_name, case_name), test_str, |b, s| {
                b.iter(|| black_box(convention.validate(black_box(s))))
            });
        }
    }

    group.finish();
}

/// Benchmark batch validation of multiple names against conventions
///
/// Compares throughput when validating datasets of different sizes.
fn bench_case_convention_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("case_convention_batch");

    // Create test data sets of various sizes
    let small_set: Vec<String> = (0..10).map(|i| format!("test_file_{}", i)).collect();
    let medium_set: Vec<String> = (0..100).map(|i| format!("test_file_{}", i)).collect();
    let large_set: Vec<String> = (0..1000).map(|i| format!("test_file_{}", i)).collect();

    for (name, dataset) in [
        ("small_10", &small_set),
        ("medium_100", &medium_set),
        ("large_1000", &large_set),
    ] {
        group.throughput(Throughput::Elements(dataset.len() as u64));

        // Benchmark batch validation for most commonly used conventions
        group.bench_function(BenchmarkId::new("snake_case", name), |b| {
            b.iter(|| {
                for name in dataset.iter() {
                    black_box(CaseConvention::SnakeCase.validate(name));
                }
            })
        });

        group.bench_function(BenchmarkId::new("kebab-case", name), |b| {
            b.iter(|| {
                for name in dataset.iter() {
                    black_box(CaseConvention::KebabCase.validate(name));
                }
            })
        });

        group.bench_function(BenchmarkId::new("camelCase", name), |b| {
            b.iter(|| {
                for name in dataset.iter() {
                    black_box(CaseConvention::CamelCase.validate(name));
                }
            })
        });

        group.bench_function(BenchmarkId::new("PascalCase", name), |b| {
            b.iter(|| {
                for name in dataset.iter() {
                    black_box(CaseConvention::PascalCase.validate(name));
                }
            })
        });
    }

    group.finish();
}

/// Compare validation of valid vs invalid names
///
/// Shows whether validation short-circuits on early failures.
fn bench_valid_vs_invalid(c: &mut Criterion) {
    let mut group = c.benchmark_group("case_convention_valid_vs_invalid");

    let valid_names = ["my-file", "another-test-file", "simple"];

    let invalid_names = [
        "my_file",  // Wrong separator
        "MyFile",   // Wrong case
        "my--file", // Double separator
        "-leading", // Leading separator
    ];

    for name in valid_names {
        group.bench_with_input(BenchmarkId::new("kebab_valid", name), name, |b, n| {
            b.iter(|| black_box(CaseConvention::KebabCase.validate(black_box(n))))
        });
    }

    for name in invalid_names {
        group.bench_with_input(BenchmarkId::new("kebab_invalid", name), name, |b, n| {
            b.iter(|| black_box(CaseConvention::KebabCase.validate(black_box(n))))
        });
    }

    group.finish();
}

// ============================================================================
// 2. Directory Validation with Various Directory Depths
// ============================================================================

/// Create a directory structure with specified depth and branching factor
fn create_directory_structure(depth: usize, breadth: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    fn create_level(
        path: &std::path::Path,
        current_depth: usize,
        max_depth: usize,
        breadth: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }

        for i in 0..breadth {
            let dir_name = format!("level{}-dir{}", current_depth, i);
            let dir_path = path.join(&dir_name);
            std::fs::create_dir(&dir_path).unwrap();

            // Create a file in each directory
            let file_path = dir_path.join(format!("file{}.txt", i));
            std::fs::write(&file_path, "test content").unwrap();

            // Recurse
            create_level(&dir_path, current_depth + 1, max_depth, breadth);
        }
    }

    create_level(base_path, 0, depth, breadth);
    temp_dir
}

/// Benchmark directory validation at various depths
///
/// Compares validation performance across different tree structures.
fn bench_directory_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_validation_depth");

    // Test different directory depths and branching factors
    let configs = [
        ("shallow_2x3", 2, 3), // 2 levels, 3 dirs per level = 12 dirs
        ("medium_3x5", 3, 5),  // 3 levels, 5 dirs per level = 155 dirs
        ("deep_5x3", 5, 3),    // 5 levels, 3 dirs per level = 363 dirs
        ("wide_2x10", 2, 10),  // 2 levels, 10 dirs per level = 110 dirs
    ];

    for (name, depth, breadth) in configs {
        let temp_dir = create_directory_structure(depth, breadth);
        let constraint = DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase);
        let context = ConstraintContext::new();

        group.bench_function(BenchmarkId::new("validate", name), |b| {
            b.iter(|| {
                let result = constraint.validate(black_box(temp_dir.path()), &context);
                black_box(result);
            })
        });
    }

    group.finish();
}

/// Benchmark directory validation with exclusions
///
/// Shows the performance impact of exclusion checking.
fn bench_directory_with_exclusions(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_validation_exclusions");

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create directories including excluded ones
    for i in 0..30 {
        let dir_name = if i % 5 == 0 {
            format!(".git{}", i) // Excluded
        } else if i % 5 == 1 {
            format!("node_modules{}", i) // Excluded
        } else if i % 5 == 2 {
            format!("target{}", i) // Excluded
        } else {
            format!("valid-dir{}", i)
        };
        let dir_path = base_path.join(&dir_name);
        std::fs::create_dir(&dir_path).unwrap();
    }

    let constraint = DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase);
    let context = ConstraintContext::new();

    group.bench_function("with_exclusions", |b| {
        b.iter(|| {
            let result = constraint.validate(black_box(base_path), &context);
            black_box(result);
        })
    });

    // Benchmark with custom config that has more exclusions
    let config = DirectoryValidationConfig::new()
        .with_excluded_dir("custom_exclude")
        .with_excluded_dir("another_exclude");
    let constraint_custom = DirectoryConstraint::new()
        .with_config(config)
        .with_case_convention(CaseConvention::KebabCase);

    group.bench_function("with_more_exclusions", |b| {
        b.iter(|| {
            let result = constraint_custom.validate(black_box(base_path), &context);
            black_box(result);
        })
    });

    group.finish();
}

/// Benchmark recursive vs non-recursive validation
fn bench_recursive_vs_flat(c: &mut Criterion) {
    let mut group = c.benchmark_group("directory_validation_recursive");

    let temp_dir = create_directory_structure(4, 4);
    let base_path = temp_dir.path();

    let recursive_constraint =
        DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase);

    let non_recursive_config = DirectoryValidationConfig::new().non_recursive();
    let flat_constraint = DirectoryConstraint::new()
        .with_config(non_recursive_config)
        .with_case_convention(CaseConvention::KebabCase);

    let context = ConstraintContext::new();

    group.bench_function("recursive", |b| {
        b.iter(|| {
            let result = recursive_constraint.validate(black_box(base_path), &context);
            black_box(result);
        })
    });

    group.bench_function("non_recursive", |b| {
        b.iter(|| {
            let result = flat_constraint.validate(black_box(base_path), &context);
            black_box(result);
        })
    });

    group.finish();
}

// ============================================================================
// 3. Multi-Part Extension Pattern Matching
// ============================================================================

/// Benchmark extension pattern parsing and matching
///
/// Compares performance across different extension complexities.
fn bench_extension_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("extension_patterns");

    // Test filenames with various extension complexities
    let test_files = [
        ("simple_js", "file.js"),
        ("component_test_js", "component.test.js"),
        ("types_d_ts", "types.d.ts"),
        ("bundle_min_css", "bundle.min.css"),
        ("deep_pattern", "deeply.nested.extension.pattern.json"),
    ];

    for (name, filename) in test_files {
        group.throughput(Throughput::Bytes(filename.len() as u64));

        // Parse extension from filename
        group.bench_with_input(BenchmarkId::new("parse", name), filename, |b, f| {
            b.iter(|| black_box(ExtensionPattern::from_filename(f)))
        });

        // Pattern matching with wildcard
        let pattern = ExtensionPattern::new("*.js");
        group.bench_with_input(
            BenchmarkId::new("match_wildcard", name),
            filename,
            |b, f| {
                let ext = ExtensionPattern::from_filename(f).unwrap();
                b.iter(|| black_box(pattern.matches(&ext)))
            },
        );
    }

    group.finish();
}

/// Benchmark multi-part extension rule validation
///
/// Compares simple vs complex rule validation performance.
fn bench_multi_part_extension_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_part_extension_rules");

    let test_files: Vec<String> = (0..100)
        .map(|i| match i % 6 {
            0 => format!("component{}.test.js", i),
            1 => format!("types{}.d.ts", i),
            2 => format!("bundle{}.min.css", i),
            3 => format!("module{}.rs", i),
            4 => format!("test{}.spec.ts", i),
            _ => format!("file{}.txt", i),
        })
        .collect();

    // Simple rule with few extensions
    let simple_rule = MultiPartExtensionRule::new()
        .allow_extension("js")
        .allow_extension("ts");

    // Complex rule with multi-part extensions
    let complex_rule = MultiPartExtensionRule::new()
        .allow_extension("d.ts")
        .allow_extension("test.js")
        .allow_extension("min.css")
        .allow_extension("spec.ts")
        .with_naming_convention("test.js", CaseConvention::KebabCase);

    group.throughput(Throughput::Elements(test_files.len() as u64));

    group.bench_function("simple_rule_batch_100", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(simple_rule.validate(file));
            }
        })
    });

    group.bench_function("complex_rule_batch_100", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(complex_rule.validate(file));
            }
        })
    });

    // Benchmark with naming convention checking
    let with_naming_rule = MultiPartExtensionRule::new()
        .allow_extension("test.js")
        .with_naming_convention("test.js", CaseConvention::KebabCase);

    let valid_test_files: Vec<String> = (0..50)
        .map(|i| format!("my-component-{}.test.js", i))
        .collect();

    group.bench_function("with_naming_convention", |b| {
        b.iter(|| {
            for file in &valid_test_files {
                black_box(with_naming_rule.validate(file));
            }
        })
    });

    group.finish();
}

/// Benchmark complex extension detection functions
fn bench_complex_extension_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_extension_detection");

    let test_files: Vec<String> = (0..1000)
        .map(|i| match i % 4 {
            0 => format!("test{}.test.js", i),
            1 => format!("spec{}.spec.ts", i),
            2 => format!("min{}.min.js", i),
            _ => format!("normal{}.js", i),
        })
        .collect();

    group.throughput(Throughput::Elements(test_files.len() as u64));

    group.bench_function("is_test_file", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(ComplexExtension::is_test_file(file));
            }
        })
    });

    group.bench_function("is_minified_file", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(ComplexExtension::is_minified_file(file));
            }
        })
    });

    group.bench_function("is_declaration_file", |b| {
        b.iter(|| {
            for file in &test_files {
                black_box(ComplexExtension::is_declaration_file(file));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 4. Multiple Rule Syntax Parsing and Validation
// ============================================================================

/// Benchmark OR syntax parsing performance
///
/// Shows how parsing time scales with number of alternatives.
fn bench_or_syntax_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("or_syntax_parsing");

    let rule_strings = [
        ("1_alt", "kebab-case"),
        ("2_alt", "kebab-case | snake_case"),
        ("3_alt", "kebab-case | snake_case | camelCase"),
        ("4_alt", "kebab-case | snake_case | camelCase | PascalCase"),
        ("6_alt", "kebab-case | snake_case | camelCase | PascalCase | dot.case | Train-Case"),
        ("12_alt", "lowercase | UPPERCASE | snake_case | camelCase | PascalCase | kebab-case | SCREAMING_SNAKE_CASE | dot.case | flatcase | FLATCASE | COBOL-CASE | Train-Case"),
    ];

    for (name, rule_str) in rule_strings {
        group.throughput(Throughput::Bytes(rule_str.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", name), rule_str, |b, s| {
            b.iter(|| black_box(MultipleRuleSyntax::parse(s).unwrap()))
        });
    }

    group.finish();
}

/// Benchmark OR syntax validation performance
///
/// Compares validation speed with different numbers of alternatives.
fn bench_or_syntax_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("or_syntax_validation");

    // Parse rules with varying numbers of alternatives
    let two_alt = MultipleRuleSyntax::parse("kebab-case | snake_case").unwrap();
    let four_alt =
        MultipleRuleSyntax::parse("kebab-case | snake_case | camelCase | PascalCase").unwrap();
    let six_alt = MultipleRuleSyntax::parse(
        "kebab-case | snake_case | camelCase | PascalCase | dot.case | Train-Case",
    )
    .unwrap();

    // Test names representing different conventions
    let test_names = [
        ("kebab", "my-file"),
        ("snake", "my_file"),
        ("camel", "myFile"),
        ("pascal", "MyFile"),
        ("dot", "my.file"),
        ("train", "My-File"),
    ];

    for (name, test_name) in test_names {
        group.bench_with_input(
            BenchmarkId::new("2_alternatives", name),
            test_name,
            |b, n| b.iter(|| black_box(two_alt.validate(n))),
        );

        group.bench_with_input(
            BenchmarkId::new("4_alternatives", name),
            test_name,
            |b, n| b.iter(|| black_box(four_alt.validate(n))),
        );

        group.bench_with_input(
            BenchmarkId::new("6_alternatives", name),
            test_name,
            |b, n| b.iter(|| black_box(six_alt.validate(n))),
        );
    }

    group.finish();
}

/// Benchmark batch validation with OR syntax
fn bench_or_syntax_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("or_syntax_batch");

    let rule = MultipleRuleSyntax::parse("kebab-case | snake_case | camelCase").unwrap();

    // Generate test data with mixed valid names
    let names: Vec<String> = (0..1000)
        .map(|i| match i % 4 {
            0 => format!("kebab-case-{}", i),
            1 => format!("snake_case_{}", i),
            2 => format!("camelCase{}", i),
            _ => format!("InvalidName{}", i),
        })
        .collect();

    group.throughput(Throughput::Elements(names.len() as u64));

    group.bench_function("validate_1000_names", |b| {
        b.iter(|| {
            for name in &names {
                black_box(rule.validate(name));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 5. Path-Specific Rule Matching with Glob Patterns
// ============================================================================

/// Benchmark glob pattern compilation to regex
///
/// Shows the one-time cost of pattern compilation.
fn bench_glob_to_regex(c: &mut Criterion) {
    let mut group = c.benchmark_group("glob_pattern_compilation");

    let patterns = [
        ("simple_star", "*.rs"),
        ("single_glob", "src/**/*.rs"),
        ("double_glob", "src/**/components/**/*.ts"),
        ("complex_pattern", "packages/*/{src,tests}/**/*.{ts,tsx}"),
        ("character_class", "src/[a-z]*/**/*.rs"),
    ];

    for (name, pattern) in patterns {
        group.throughput(Throughput::Bytes(pattern.len() as u64));
        group.bench_with_input(BenchmarkId::new("compile", name), pattern, |b, p| {
            b.iter(|| black_box(PathRule::new(p, CaseConvention::SnakeCase).unwrap()))
        });
    }

    group.finish();
}

/// Benchmark path rule matching performance
fn bench_path_rule_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_rule_matching");

    // Create various path rules
    let rules = [
        (
            "simple_star",
            PathRule::new("*.rs", CaseConvention::SnakeCase).unwrap(),
        ),
        (
            "src_glob",
            PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap(),
        ),
        (
            "deep_glob",
            PathRule::new("src/**/components/**/*.ts", CaseConvention::PascalCase).unwrap(),
        ),
        (
            "packages",
            PathRule::new("packages/*/src/**/*.ts", CaseConvention::CamelCase).unwrap(),
        ),
    ];

    // Test paths with varying complexity
    let paths = [
        Path::new("main.rs"),
        Path::new("src/main.rs"),
        Path::new("src/utils/helpers.rs"),
        Path::new("src/components/Button.ts"),
        Path::new("packages/core/src/index.ts"),
        Path::new("tests/test.rs"),
    ];

    for (rule_name, rule) in &rules {
        for path in paths {
            group.bench_with_input(
                BenchmarkId::new(*rule_name, path.to_str().unwrap()),
                path,
                |b, p| b.iter(|| black_box(rule.matches(p))),
            );
        }
    }

    group.finish();
}

/// Benchmark path rule configuration with multiple rules
fn bench_path_rule_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("path_rule_config");

    // Create complex path rule configuration
    let config = PathRuleConfig::new()
        .with_rule(PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap())
        .with_rule(PathRule::new("tests/**/*.rs", CaseConvention::SnakeCase).unwrap())
        .with_rule(
            PathRule::new("src/components/**/*.ts", CaseConvention::PascalCase)
                .unwrap()
                .with_severity(Severity::High),
        )
        .with_rule(PathRule::new("src/hooks/**/*.ts", CaseConvention::CamelCase).unwrap())
        .with_rule(PathRule::new("src/utils/**/*.ts", CaseConvention::SnakeCase).unwrap())
        .with_default_convention(CaseConvention::KebabCase);

    // Test paths covering different rule matches
    let paths: Vec<&Path> = vec![
        Path::new("src/my_module.rs"),
        Path::new("src/my-module.rs"), // Invalid
        Path::new("tests/my_test.rs"),
        Path::new("src/components/MyComponent.ts"),
        Path::new("src/components/my-component.ts"), // Invalid
        Path::new("src/hooks/useState.ts"),
        Path::new("src/utils/my_helper.ts"),
        Path::new("docs/my-file.md"),
        Path::new("docs/my_file.md"), // Invalid (default convention)
    ];

    group.bench_function("find_rule_9_paths", |b| {
        b.iter(|| {
            for path in &paths {
                black_box(config.find_rule(path));
            }
        })
    });

    group.bench_function("validate_9_paths", |b| {
        b.iter(|| {
            for path in &paths {
                black_box(config.validate(path));
            }
        })
    });

    group.finish();
}

/// Benchmark nested path rules (rule hierarchy)
fn bench_nested_path_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("nested_path_rules");

    // Create nested rule structure
    let parent_rule = PathRule::new("src/**", CaseConvention::SnakeCase)
        .unwrap()
        .with_child_rule(PathRule::new("src/components/**", CaseConvention::PascalCase).unwrap())
        .with_child_rule(PathRule::new("src/hooks/**", CaseConvention::CamelCase).unwrap())
        .with_child_rule(PathRule::new("src/utils/**", CaseConvention::SnakeCase).unwrap());

    let paths = [
        Path::new("src/utils.rs"),
        Path::new("src/components/Button.rs"),
        Path::new("src/components/nested/Card.rs"),
        Path::new("src/hooks/useState.rs"),
        Path::new("src/services/api.rs"),
    ];

    for path in paths {
        group.bench_with_input(
            BenchmarkId::new("find_matching_rule", path.to_str().unwrap()),
            path,
            |b, p| b.iter(|| black_box(parent_rule.find_matching_rule(p))),
        );
    }

    group.finish();
}

// ============================================================================
// 6. Full Constraint Validation Scenarios
// ============================================================================

/// Benchmark full naming constraint validation
fn bench_naming_constraint(c: &mut Criterion) {
    let mut group = c.benchmark_group("naming_constraint_full");

    // Create temp directory with test files
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create test files
    for i in 0..50 {
        let valid_file = base_path.join(format!("valid-file-{}.txt", i));
        let invalid_file = base_path.join(format!("invalid_file_{}.txt", i));
        std::fs::write(&valid_file, "content").unwrap();
        std::fs::write(&invalid_file, "content").unwrap();
    }

    // Create constraint with multiple rules
    let constraint = NamingConstraint::new()
        .with_case_convention(CaseConvention::KebabCase)
        .with_extension_rule(
            ExtensionRule::new()
                .allow_extension("txt")
                .with_severity(Severity::High),
        );

    let context = ConstraintContext::new();

    // Benchmark single file validation
    let test_file = base_path.join("test-file.txt");
    group.bench_function("single_file", |b| {
        b.iter(|| {
            let result = constraint.validate(black_box(&test_file), &context);
            black_box(result);
        })
    });

    // Benchmark batch validation
    group.bench_function("batch_50_valid_files", |b| {
        b.iter(|| {
            for i in 0..50 {
                let file = base_path.join(format!("valid-file-{}.txt", i));
                let result = constraint.validate(&file, &context);
                black_box(result);
            }
        })
    });

    group.bench_function("batch_50_mixed_files", |b| {
        b.iter(|| {
            for i in 0..25 {
                let valid_file = base_path.join(format!("valid-file-{}.txt", i));
                let invalid_file = base_path.join(format!("invalid_file_{}.txt", i));
                black_box(constraint.validate(&valid_file, &context));
                black_box(constraint.validate(&invalid_file, &context));
            }
        })
    });

    group.finish();
}

/// Benchmark constraint engine with multiple constraints
fn bench_constraint_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("constraint_engine");

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a test file
    let test_file = base_path.join("test-file.txt");
    std::fs::write(&test_file, "test content for validation").unwrap();

    // Create engine with multiple constraints
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Register naming constraint
    let naming_constraint = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
    engine.register_constraint(Box::new(naming_constraint));

    // Register file size constraint
    let size_constraint = FileSizeConstraint::new()
        .add_rule(FileSizeRule::new("max_size").max_size(FileSizeLimit::Megabytes(1)));
    engine.register_constraint(Box::new(size_constraint));

    let context = ConstraintContext::new();

    group.bench_function("validate_2_constraints", |b| {
        b.iter(|| {
            let results = engine.validate(black_box(&test_file), &context);
            black_box(results);
        })
    });

    // Create engine with more constraints
    let mut complex_engine = ConstraintEngine::new(ConstraintConfig::new());
    complex_engine.register_constraint(Box::new(
        NamingConstraint::new().with_case_convention(CaseConvention::KebabCase),
    ));
    complex_engine
        .register_constraint(Box::new(FileSizeConstraint::new().add_rule(
            FileSizeRule::new("max_size").max_size(FileSizeLimit::Megabytes(1)),
        )));
    complex_engine.register_constraint(Box::new(
        DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase),
    ));
    complex_engine.register_constraint(Box::new(
        NamingConstraint::new()
            .with_file_pattern("*.rs")
            .with_case_convention(CaseConvention::SnakeCase),
    ));

    group.bench_function("validate_4_constraints", |b| {
        b.iter(|| {
            let results = complex_engine.validate(black_box(&test_file), &context);
            black_box(results);
        })
    });

    group.finish();
}

/// Benchmark full project validation scenario
fn bench_full_project_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_project_validation");

    // Create a realistic project structure
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create src directory with files
    let src_dir = base_path.join("src");
    std::fs::create_dir(&src_dir).unwrap();
    for i in 0..20 {
        let file = src_dir.join(format!("module_{}.rs", i));
        std::fs::write(&file, format!("// Module {}", i)).unwrap();
    }

    // Create tests directory
    let tests_dir = base_path.join("tests");
    std::fs::create_dir(&tests_dir).unwrap();
    for i in 0..10 {
        let file = tests_dir.join(format!("test_{}.rs", i));
        std::fs::write(&file, format!("// Test {}", i)).unwrap();
    }

    // Create nested component directories
    let components_dir = src_dir.join("components");
    std::fs::create_dir(&components_dir).unwrap();
    for i in 0..5 {
        let comp_dir = components_dir.join(format!("component-{}", i));
        std::fs::create_dir(&comp_dir).unwrap();
        let file = comp_dir.join("mod.rs");
        std::fs::write(&file, format!("// Component {}", i)).unwrap();
    }

    // Set up constraint engine with path-specific rules
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Add naming constraint with path rules
    let naming = NamingConstraint::new()
        .with_case_convention(CaseConvention::SnakeCase)
        .with_file_pattern("*.rs");
    engine.register_constraint(Box::new(naming));

    // Add directory constraint
    let directory = DirectoryConstraint::new().with_case_convention(CaseConvention::KebabCase);
    engine.register_constraint(Box::new(directory));

    // Add file size constraint
    let size = FileSizeConstraint::new()
        .add_rule(FileSizeRule::new("max_size").max_size(FileSizeLimit::Kilobytes(10)));
    engine.register_constraint(Box::new(size));

    let context = ConstraintContext::new();

    group.bench_function("validate_project_35_files_5_dirs", |b| {
        b.iter(|| {
            let mut all_results = Vec::new();

            // Validate source files
            for i in 0..20 {
                let file = src_dir.join(format!("module_{}.rs", i));
                let results = engine.validate(&file, &context);
                all_results.extend(results);
            }

            // Validate test files
            for i in 0..10 {
                let file = tests_dir.join(format!("test_{}.rs", i));
                let results = engine.validate(&file, &context);
                all_results.extend(results);
            }

            // Validate directories
            for i in 0..5 {
                let dir = components_dir.join(format!("component-{}", i));
                let results = engine.validate(&dir, &context);
                all_results.extend(results);
            }

            black_box(all_results);
        })
    });

    group.finish();
}

// ============================================================================
// Comparative Benchmarks
// ============================================================================

/// Compare different validation approaches for the same task
fn bench_comparative_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparative");

    let test_names: Vec<String> = (0..100).map(|i| format!("test-file-{}", i)).collect();

    // Approach 1: Direct case convention validation
    group.bench_function("direct_validation", |b| {
        b.iter(|| {
            for name in &test_names {
                black_box(CaseConvention::KebabCase.validate(name));
            }
        })
    });

    // Approach 2: Multiple rule syntax with single alternative
    let single_alt = MultipleRuleSyntax::parse("kebab-case").unwrap();
    group.bench_function("single_alternative_syntax", |b| {
        b.iter(|| {
            for name in &test_names {
                black_box(single_alt.validate(name));
            }
        })
    });

    // Approach 3: Path rule validation
    let path_rule = PathRule::new("**/*", CaseConvention::KebabCase).unwrap();
    group.bench_function("path_rule_validation", |b| {
        b.iter(|| {
            for name in &test_names {
                black_box(path_rule.validate(name));
            }
        })
    });

    // Approach 4: Naming constraint
    let temp_dir = TempDir::new().unwrap();
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);
    let context = ConstraintContext::new();

    group.bench_function("naming_constraint_validation", |b| {
        b.iter(|| {
            for i in 0..100 {
                let path = temp_dir.path().join(format!("test-file-{}", i));
                black_box(constraint.validate(&path, &context));
            }
        })
    });

    group.finish();
}

/// Compare performance across different severity levels
fn bench_severity_levels(c: &mut Criterion) {
    let mut group = c.benchmark_group("severity_levels");

    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test-file.txt");
    std::fs::write(&test_file, "content").unwrap();

    let context = ConstraintContext::new();

    // Benchmark with different severity configurations
    for severity in [
        Severity::Low,
        Severity::Medium,
        Severity::High,
        Severity::Critical,
    ] {
        let constraint = NamingConstraint::new()
            .with_case_convention(CaseConvention::KebabCase)
            .with_default_severity(severity);

        group.bench_with_input(
            BenchmarkId::new("severity", format!("{:?}", severity)),
            &test_file,
            |b, path| b.iter(|| black_box(constraint.validate(path, &context))),
        );
    }

    group.finish();
}

// ============================================================================
// Criterion Groups and Main
// ============================================================================

criterion_group!(
    case_conventions,
    bench_case_conventions,
    bench_case_convention_batch,
    bench_valid_vs_invalid,
);

criterion_group!(
    directory_validation,
    bench_directory_validation,
    bench_directory_with_exclusions,
    bench_recursive_vs_flat,
);

criterion_group!(
    extension_patterns,
    bench_extension_patterns,
    bench_multi_part_extension_rules,
    bench_complex_extension_detection,
);

criterion_group!(
    rule_syntax,
    bench_or_syntax_parsing,
    bench_or_syntax_validation,
    bench_or_syntax_batch,
);

criterion_group!(
    path_rules,
    bench_glob_to_regex,
    bench_path_rule_matching,
    bench_path_rule_config,
    bench_nested_path_rules,
);

criterion_group!(
    constraint_scenarios,
    bench_naming_constraint,
    bench_constraint_engine,
    bench_full_project_validation,
);

criterion_group!(
    comparative,
    bench_comparative_validation,
    bench_severity_levels,
);

criterion_main!(
    case_conventions,
    directory_validation,
    extension_patterns,
    rule_syntax,
    path_rules,
    constraint_scenarios,
    comparative,
);
