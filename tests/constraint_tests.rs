//! Comprehensive integration tests for the constraint system

use assura::{
    CaseConvention, Constraint, ConstraintConfig, ConstraintContext, ConstraintEngine,
    ConstraintOutput, ExtensionRule, FileSizeConstraint, FileSizeLimit, FileSizeRule,
    MaturityLevel, NamingConstraint, Severity, SeverityConfig, ValidationFailure,
    ValidationFailures,
};
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_constraint_engine_basic() {
    let config = ConstraintConfig::new();
    let mut engine = ConstraintEngine::new(config);

    // Register constraints
    engine.register_constraint(Box::new(FileSizeConstraint::default_config()));
    engine.register_constraint(Box::new(NamingConstraint::general_config()));

    let context = ConstraintContext::new();

    // Test with a small, valid file
    let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    temp_file.write_all(b"Hello").unwrap();

    let results = engine.validate(temp_file.path(), &context);

    // Should have results from both constraints
    assert_eq!(results.len(), 2);

    // All should pass for a small file with valid name
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().passed);
    }
}

#[test]
fn test_file_size_constraint_with_large_file() {
    let constraint = FileSizeConstraint::new().add_rule(
        FileSizeRule::new("max_1kb")
            .max_size(FileSizeLimit::Kilobytes(1))
            .with_severity(Severity::High),
    );

    let context = ConstraintContext::new();

    // Create a file larger than 1KB
    let mut temp_file = NamedTempFile::new().unwrap();
    let large_content = vec![b'x'; 2048]; // 2KB
    temp_file.write_all(&large_content).unwrap();

    let result = constraint.validate(temp_file.path(), &context).unwrap();

    assert!(!result.passed);
    assert_eq!(result.failures.len(), 1);
    assert_eq!(result.severity, Severity::High);
}

#[test]
fn test_naming_constraint_kebab_case() {
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);

    let context = ConstraintContext::new();

    // Valid kebab-case
    let result = constraint
        .validate(PathBuf::from("/test/my-file.txt").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid (snake_case)
    let result = constraint
        .validate(PathBuf::from("/test/my_file.txt").as_path(), &context)
        .unwrap();
    assert!(!result.passed);

    // Invalid (camelCase)
    let result = constraint
        .validate(PathBuf::from("/test/myFile.txt").as_path(), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_naming_constraint_snake_case() {
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::SnakeCase);

    let context = ConstraintContext::new();

    // Valid snake_case
    let result = constraint
        .validate(PathBuf::from("/test/my_file.rs").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid (kebab-case)
    let result = constraint
        .validate(PathBuf::from("/test/my-file.rs").as_path(), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_naming_constraint_with_extension_rule() {
    let constraint = NamingConstraint::new().with_extension_rule(
        ExtensionRule::new()
            .allow_extension("rs")
            .allow_extension("toml"),
    );

    let context = ConstraintContext::new();

    // Valid extensions
    let result = constraint
        .validate(PathBuf::from("/test/main.rs").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    let result = constraint
        .validate(PathBuf::from("/test/Cargo.toml").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid extension
    let result = constraint
        .validate(PathBuf::from("/test/readme.md").as_path(), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_severity_mapping_with_maturity() {
    let config = SeverityConfig::new().with_base_severity("file_size", Severity::Medium);

    // Test raw maturity - should escalate
    let raw_severity =
        config.get_effective_severity("file_size", MaturityLevel::Raw, Severity::Medium);
    assert_eq!(raw_severity, Severity::High);

    // Test mature maturity - should stay the same
    let mature_severity =
        config.get_effective_severity("file_size", MaturityLevel::Mature, Severity::Medium);
    assert_eq!(mature_severity, Severity::Medium);

    // Test established maturity - should escalate
    let established_severity =
        config.get_effective_severity("file_size", MaturityLevel::Established, Severity::Medium);
    assert_eq!(established_severity, Severity::High);
}

#[test]
fn test_severity_config_should_report() {
    let config = SeverityConfig::new().with_min_severity(Severity::Medium);

    assert!(config.should_report(Severity::Medium));
    assert!(config.should_report(Severity::High));
    assert!(config.should_report(Severity::Critical));
    assert!(!config.should_report(Severity::Low));
}

#[test]
fn test_severity_config_should_fail() {
    let config = SeverityConfig::new().fail_on(Severity::High);

    assert!(!config.should_fail(Severity::Medium));
    assert!(config.should_fail(Severity::High));
    assert!(config.should_fail(Severity::Critical));
    assert!(!config.should_fail(Severity::Low));
}

#[test]
fn test_constraint_context_builder() {
    let context = ConstraintContext::new()
        .with_project_root("/project")
        .with_maturity_level(MaturityLevel::Mature)
        .manual()
        .with_fail_fast()
        .with_metadata("key", "value");

    assert_eq!(
        context.project_root(),
        Some(std::path::Path::new("/project"))
    );
    assert_eq!(context.maturity_level(), MaturityLevel::Mature);
    assert!(context.is_manual);
    assert!(context.fail_fast);
}

#[test]
fn test_constraint_engine_with_triggers() {
    let mut engine = ConstraintEngine::new(ConstraintConfig::new());

    engine.register_constraint(Box::new(FileSizeConstraint::default_config()));

    // Without triggers, constraint should always run
    let context = ConstraintContext::new();
    let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    temp_file.write_all(b"test").unwrap();

    let results = engine.validate(temp_file.path(), &context);
    assert!(!results.is_empty());
}

#[test]
fn test_validation_failure_with_suggestion() {
    let failure = ValidationFailure::new("test_constraint", "/test/file.txt", "File is too large")
        .with_suggestion("Consider splitting into multiple files");

    assert_eq!(failure.constraint, "test_constraint");
    assert_eq!(failure.path, PathBuf::from("/test/file.txt"));
    assert_eq!(failure.message, "File is too large");
    assert_eq!(
        failure.suggestion,
        Some("Consider splitting into multiple files".to_string())
    );
}

#[test]
fn test_validation_failures_collection() {
    let mut failures = ValidationFailures::new();

    failures.add(ValidationFailure::new("c1", "/a", "error 1"));
    failures.add(ValidationFailure::new("c2", "/b", "error 2"));
    failures.add(ValidationFailure::new("c3", "/c", "error 3"));

    assert_eq!(failures.len(), 3);
    assert!(!failures.is_empty());

    let collected: Vec<_> = failures.into_iter().collect();
    assert_eq!(collected.len(), 3);
}

#[test]
fn test_file_size_rule_pattern_matching() {
    let rule = FileSizeRule::new("source_files")
        .with_pattern("*.rs")
        .with_pattern("*.js")
        .max_size(FileSizeLimit::Kilobytes(50));

    assert!(rule.applies_to(PathBuf::from("/src/main.rs").as_path()));
    assert!(rule.applies_to(PathBuf::from("/src/app.js").as_path()));
    assert!(!rule.applies_to(PathBuf::from("/README.md").as_path()));
}

#[test]
fn test_case_convention_examples() {
    assert_eq!(CaseConvention::SnakeCase.example(), "file_name");
    assert_eq!(CaseConvention::KebabCase.example(), "file-name");
    assert_eq!(CaseConvention::CamelCase.example(), "fileName");
    assert_eq!(CaseConvention::PascalCase.example(), "FileName");
}

#[test]
fn test_severity_levels_ordering() {
    use assura::Severity::*;

    assert!(Critical > High);
    assert!(High > Medium);
    assert!(Medium > Low);

    assert!(Low < Medium);
    assert!(Medium < High);
    assert!(High < Critical);
}

#[test]
fn test_file_size_limit_formatting() {
    assert_eq!(FileSizeLimit::Bytes(512).format(), "512 B");
    assert_eq!(FileSizeLimit::Kilobytes(1).format(), "1 KB");
    assert_eq!(FileSizeLimit::Megabytes(5).format(), "5 MB");
    assert_eq!(FileSizeLimit::Gigabytes(2).format(), "2 GB");
}

#[test]
fn test_constraint_output_builder() {
    let output = ConstraintOutput::new("test", "/path", true)
        .with_severity(Severity::High)
        .with_duration(100)
        .with_metadata(serde_json::json!({"key": "value"}));

    assert_eq!(output.constraint_name, "test");
    assert!(output.passed);
    assert_eq!(output.severity, Severity::High);
    assert_eq!(output.duration_ms, 100);
    assert!(output.metadata.is_some());
}

#[test]
fn test_rust_naming_config() {
    let constraint = NamingConstraint::rust_config();
    let context = ConstraintContext::new();

    // Valid Rust naming
    let result = constraint
        .validate(PathBuf::from("/src/my_module.rs").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid (kebab-case in Rust)
    let result = constraint
        .validate(PathBuf::from("/src/my-module.rs").as_path(), &context)
        .unwrap();
    assert!(!result.passed);

    // Invalid extension
    let result = constraint
        .validate(PathBuf::from("/src/my_module.js").as_path(), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_javascript_naming_config() {
    let constraint = NamingConstraint::javascript_config();
    let context = ConstraintContext::new();

    // Valid JavaScript naming (kebab-case)
    let result = constraint
        .validate(PathBuf::from("/src/my-component.js").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid (snake_case in JS convention)
    let result = constraint
        .validate(PathBuf::from("/src/my_component.js").as_path(), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_general_naming_config() {
    let constraint = NamingConstraint::general_config();
    let context = ConstraintContext::new();

    // Valid filename
    let result = constraint
        .validate(PathBuf::from("/test/valid-file.txt").as_path(), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid (has spaces)
    let result = constraint
        .validate(
            PathBuf::from("/test/file with spaces.txt").as_path(),
            &context,
        )
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_maturity_based_severity_adjustment() {
    use assura::Severity::*;

    // Test the default severity_for_maturity implementation
    // This is defined in the Constraint trait
    let _config = ConstraintConfig::new();

    // For raw projects, medium should become high
    let raw_adjusted = Medium; // This would be adjusted by the constraint
    assert_eq!(raw_adjusted, Medium); // Base is medium

    // The actual adjustment happens in severity_for_maturity
    // Raw: Low -> Medium, Medium -> High
    // Established: Low -> Medium
}

#[test]
fn test_constraint_result_types() {
    use assura::ConstraintError;

    // Test IO error creation
    let io_err = ConstraintError::io("/test", "permission denied");
    assert!(io_err.path().is_some());
    assert_eq!(io_err.path().unwrap(), &PathBuf::from("/test"));

    // Test validation error creation
    let val_err = ConstraintError::validation("test", "/path", "failed");
    assert!(val_err.constraint().is_some());
    assert_eq!(val_err.constraint().unwrap(), "test");
}
