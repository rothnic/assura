//! LS-Lint parity integration tests
//!
//! Tests to ensure Assura provides full parity with LS-Lint functionality.

use assura::{
    CaseConvention, Constraint, ConstraintContext, DirectoryConstraint, DirectoryRule,
    DirectoryValidationConfig, ExtensionPattern, MultiPartExtensionRule, MultipleRuleSyntax,
    NamingConstraint, PathRule, PathRuleConfig, Severity,
};
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

// ============================================================================
// 5.1 Case Convention Tests
// ============================================================================

#[test]
fn test_ls_lint_flatcase_convention() {
    // flatcase: lowercase, no separators (e.g., "filename")
    assert!(CaseConvention::FlatCase.validate("filename"));
    assert!(CaseConvention::FlatCase.validate("myfile"));
    assert!(CaseConvention::FlatCase.validate("file123"));
    
    // Invalid flatcase
    assert!(!CaseConvention::FlatCase.validate("file_name"));
    assert!(!CaseConvention::FlatCase.validate("file-name"));
    assert!(!CaseConvention::FlatCase.validate("FileName"));
    assert!(!CaseConvention::FlatCase.validate("FILENAME"));
}

#[test]
fn test_ls_lint_flatcase_upper_convention() {
    // FLATCASE: UPPERCASE, no separators (e.g., "FILENAME")
    assert!(CaseConvention::ScreamingFlatCase.validate("FILENAME"));
    assert!(CaseConvention::ScreamingFlatCase.validate("MYFILE"));
    assert!(CaseConvention::ScreamingFlatCase.validate("FILE123"));
    
    // Invalid FLATCASE
    assert!(!CaseConvention::ScreamingFlatCase.validate("file_name"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("file-name"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("FileName"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("filename"));
}

#[test]
fn test_ls_lint_cobol_case_convention() {
    // COBOL-CASE: UPPERCASE with hyphens (e.g., "FILE-NAME")
    assert!(CaseConvention::CobolCase.validate("FILE-NAME"));
    assert!(CaseConvention::CobolCase.validate("MY-FILE"));
    assert!(CaseConvention::CobolCase.validate("HTTP-REQUEST"));
    
    // Invalid COBOL-CASE
    assert!(!CaseConvention::CobolCase.validate("file-name"));
    assert!(!CaseConvention::CobolCase.validate("File-Name"));
    assert!(!CaseConvention::CobolCase.validate("FILE_NAME"));
    assert!(!CaseConvention::CobolCase.validate("FILE--NAME"));
    assert!(!CaseConvention::CobolCase.validate("-FILENAME"));
    assert!(!CaseConvention::CobolCase.validate("FILENAME-"));
}

#[test]
fn test_ls_lint_train_case_convention() {
    // Train-Case: Title-Case with hyphens (e.g., "File-Name")
    assert!(CaseConvention::TrainCase.validate("File-Name"));
    assert!(CaseConvention::TrainCase.validate("My-File"));
    assert!(CaseConvention::TrainCase.validate("Http-Request"));
    assert!(CaseConvention::TrainCase.validate("My-File-123"));
    
    // Invalid Train-Case
    assert!(!CaseConvention::TrainCase.validate("file-name"));
    assert!(!CaseConvention::TrainCase.validate("FILE-NAME"));
    assert!(!CaseConvention::TrainCase.validate("File_Name"));
    assert!(!CaseConvention::TrainCase.validate("File--Name"));
    assert!(!CaseConvention::TrainCase.validate("-FileName"));
    assert!(!CaseConvention::TrainCase.validate("FileName-"));
    assert!(!CaseConvention::TrainCase.validate("fiLe-Name"));
}

// ============================================================================
// 5.2 Directory Validation Tests
// ============================================================================

#[test]
fn test_ls_lint_directory_validation_kebab_case() {
    let constraint = DirectoryConstraint::new()
        .with_case_convention(CaseConvention::KebabCase);

    let context = ConstraintContext::new();
    let temp_dir = TempDir::new().unwrap();
    
    // Valid directory name
    let valid_dir = temp_dir.path().join("my-directory");
    std::fs::create_dir(&valid_dir).unwrap();
    
    let result = constraint.validate(&valid_dir, &context).unwrap();
    assert!(result.passed);
    
    // Invalid directory name (snake_case)
    let invalid_dir = temp_dir.path().join("my_directory");
    std::fs::create_dir(&invalid_dir).unwrap();
    
    let result = constraint.validate(&invalid_dir, &context).unwrap();
    assert!(!result.passed);
}

#[test]
fn test_ls_lint_directory_exclusions() {
    let config = DirectoryValidationConfig::new()
        .with_excluded_dir("custom_build");
    
    let constraint = DirectoryConstraint::new()
        .with_config(config)
        .with_case_convention(CaseConvention::KebabCase);
    
    let context = ConstraintContext::new();
    let temp_dir = TempDir::new().unwrap();
    
    // Excluded directory (custom_build would normally be invalid)
    let excluded_dir = temp_dir.path().join("custom_build");
    std::fs::create_dir(&excluded_dir).unwrap();
    
    let result = constraint.validate(&excluded_dir, &context).unwrap();
    assert!(result.passed); // Should pass because it's excluded
}

#[test]
fn test_ls_lint_directory_recursive_validation() {
    let constraint = DirectoryConstraint::new()
        .with_case_convention(CaseConvention::KebabCase);
    
    let context = ConstraintContext::new();
    let temp_dir = TempDir::new().unwrap();
    
    // Create nested directories
    let parent = temp_dir.path().join("parent-dir");
    std::fs::create_dir(&parent).unwrap();
    
    let child_valid = parent.join("child-dir");
    std::fs::create_dir(&child_valid).unwrap();
    
    let child_invalid = parent.join("child_dir");
    std::fs::create_dir(&child_invalid).unwrap();
    
    // Run recursive validation
    let result = constraint.validate(&parent, &context).unwrap();
    assert!(!result.passed); // Should fail because child_dir is invalid
    assert!(result.failures.len() >= 1);
}

// ============================================================================
// 5.3 Complex Extensions Tests
// ============================================================================

#[test]
fn test_ls_lint_multi_part_extension_d_ts() {
    // TypeScript declaration files (.d.ts)
    let rule = MultiPartExtensionRule::new()
        .allow_extension("d.ts");
    
    assert!(rule.validate("types.d.ts").is_none());
    assert!(rule.validate("utils.d.ts").is_none());
    
    // Invalid extensions
    assert!(rule.validate("types.ts").is_some());
    assert!(rule.validate("types.js").is_some());
}

#[test]
fn test_ls_lint_multi_part_extension_test_js() {
    // Test files (.test.js, .spec.js)
    let rule = MultiPartExtensionRule::new()
        .allow_extension("test.js")
        .allow_extension("spec.js")
        .allow_extension("test.ts")
        .allow_extension("spec.ts");
    
    assert!(rule.validate("component.test.js").is_none());
    assert!(rule.validate("component.spec.js").is_none());
    assert!(rule.validate("component.test.ts").is_none());
    assert!(rule.validate("component.spec.ts").is_none());
    
    assert!(rule.validate("component.js").is_some());
}

#[test]
fn test_ls_lint_multi_part_extension_min_css() {
    // Minified files (.min.css, .min.js)
    let rule = MultiPartExtensionRule::new()
        .allow_extension("min.css")
        .allow_extension("min.js");
    
    assert!(rule.validate("styles.min.css").is_none());
    assert!(rule.validate("bundle.min.js").is_none());
    
    assert!(rule.validate("styles.css").is_some());
}

#[test]
fn test_ls_lint_extension_pattern_parsing() {
    let pattern = ExtensionPattern::new("d.ts");
    assert_eq!(pattern.parts().len(), 2);
    assert_eq!(pattern.parts()[0], "d");
    assert_eq!(pattern.parts()[1], "ts");
    
    let from_file = ExtensionPattern::from_filename("types.d.ts").unwrap();
    assert_eq!(from_file.as_str(), "d.ts");
}

#[test]
fn test_ls_lint_extension_naming_convention() {
    // Test file with specific naming convention
    let rule = MultiPartExtensionRule::new()
        .allow_extension("test.js")
        .with_naming_convention("test.js", CaseConvention::KebabCase);
    
    // Valid: kebab-case stem
    assert!(rule.validate("my-component.test.js").is_none());
    
    // Invalid: snake_case stem
    assert!(rule.validate("my_component.test.js").is_some());
}

// ============================================================================
// 5.4 Multiple Rules Syntax Tests
// ============================================================================

#[test]
fn test_ls_lint_or_syntax_two_alternatives() {
    // kebab-case | snake_case
    let syntax = MultipleRuleSyntax::parse("kebab-case | snake_case").unwrap();
    
    // Both should pass
    let (passed, _) = syntax.validate("my-file");
    assert!(passed);
    
    let (passed, _) = syntax.validate("my_file");
    assert!(passed);
    
    // This should fail
    let (passed, failures) = syntax.validate("MyFile");
    assert!(!passed);
    assert_eq!(failures.len(), 2); // Both alternatives failed
}

#[test]
fn test_ls_lint_or_syntax_three_alternatives() {
    // kebab-case | snake_case | camelCase
    let syntax = MultipleRuleSyntax::parse("kebab-case | snake_case | camelCase").unwrap();
    
    assert!(syntax.validate("my-file").0);
    assert!(syntax.validate("my_file").0);
    assert!(syntax.validate("myFile").0);
    
    assert!(!syntax.validate("MyFile").0);
}

#[test]
fn test_ls_lint_or_syntax_with_new_conventions() {
    // flatcase | FLATCASE
    let syntax = MultipleRuleSyntax::new()
        .add_convention(CaseConvention::FlatCase)
        .add_convention(CaseConvention::ScreamingFlatCase);
    
    assert!(syntax.validate("filename").0);
    assert!(syntax.validate("FILENAME").0);
    
    assert!(!syntax.validate("FileName").0);
}

#[test]
fn test_ls_lint_or_syntax_error_messages() {
    let syntax = MultipleRuleSyntax::parse("kebab-case | PascalCase").unwrap();
    
    let (passed, failures) = syntax.validate("my_file");
    assert!(!passed);
    
    // Should have error messages for both alternatives
    assert!(failures.iter().any(|f| f.contains("kebab-case")));
    assert!(failures.iter().any(|f| f.contains("PascalCase")));
}

// ============================================================================
// 5.5 Path-Specific Rules Tests
// ============================================================================

#[test]
fn test_ls_lint_path_specific_rule_basic() {
    // src/**/*.rs should use snake_case
    let rule = PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap();
    
    assert!(rule.matches(PathBuf::from("src/main.rs").as_path()));
    assert!(rule.matches(PathBuf::from("src/utils/helpers.rs").as_path()));
    assert!(!rule.matches(PathBuf::from("tests/main.rs").as_path()));
    assert!(!rule.matches(PathBuf::from("src/main.js").as_path()));
}

#[test]
fn test_ls_lint_path_specific_validation() {
    let rule = PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap();
    
    // Valid: snake_case
    assert!(rule.validate("my_module.rs").is_none());
    
    // Invalid: kebab-case in Rust files
    assert!(rule.validate("my-module.rs").is_some());
}

#[test]
fn test_ls_lint_path_rule_config() {
    let config = PathRuleConfig::new()
        .with_rule(
            PathRule::new("src/**/*.rs", CaseConvention::SnakeCase)
                .unwrap()
                .with_severity(Severity::High),
        )
        .with_rule(
            PathRule::new("tests/**/*.rs", CaseConvention::SnakeCase)
                .unwrap()
                .with_severity(Severity::Low),
        )
        .with_default_convention(CaseConvention::KebabCase);
    
    // src files should use snake_case
    let error = config.validate(PathBuf::from("src/my-module.rs").as_path());
    assert!(error.is_some());
    assert!(error.unwrap().contains("snake_case"));
    
    // Other files should use default (kebab-case)
    let error = config.validate(PathBuf::from("docs/my_file.md").as_path());
    assert!(error.is_some());
    assert!(error.unwrap().contains("kebab-case"));
}

#[test]
fn test_ls_lint_nested_path_rules() {
    // Parent rule for src/**
    // Child rule for src/components/**
    let parent = PathRule::new("src/**", CaseConvention::SnakeCase)
        .unwrap()
        .with_child_rule(
            PathRule::new("src/components/**", CaseConvention::PascalCase).unwrap(),
        );
    
    // Should find child rule for components
    let child_rule = parent.find_matching_rule(PathBuf::from("src/components/MyComponent.rs").as_path());
    assert!(child_rule.is_some());
    assert_eq!(child_rule.unwrap().convention, CaseConvention::PascalCase);
    
    // Should use parent rule for other src files
    let parent_rule = parent.find_matching_rule(PathBuf::from("src/utils/helpers.rs").as_path());
    assert!(parent_rule.is_some());
    assert_eq!(parent_rule.unwrap().convention, CaseConvention::SnakeCase);
}

#[test]
fn test_ls_lint_path_specific_override() {
    let rule = PathRule::new("src/components/**", CaseConvention::PascalCase)
        .unwrap()
        .as_override();
    
    assert!(rule.is_override);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_ls_lint_full_integration() {
    // Set up a complex scenario matching LS-Lint configuration
    let temp_dir = TempDir::new().unwrap();
    
    // Create directory structure
    let src = temp_dir.path().join("src");
    let components = src.join("components");
    let utils = src.join("utils");
    std::fs::create_dir_all(&components).unwrap();
    std::fs::create_dir_all(&utils).unwrap();
    
    // Create files with different naming conventions
    std::fs::write(src.join("main.rs"), "").unwrap(); // snake_case - valid
    std::fs::write(src.join("my-module.rs"), "").unwrap(); // kebab-case - invalid
    std::fs::write(components.join("MyComponent.rs"), "").unwrap(); // PascalCase - valid for components
    std::fs::write(utils.join("helpers.rs"), "").unwrap(); // snake_case - valid
    
    // Validate with path-specific rules
    let config = PathRuleConfig::new()
        .with_rule(PathRule::new("src/**/*.rs", CaseConvention::SnakeCase).unwrap())
        .with_rule(PathRule::new("src/components/**", CaseConvention::PascalCase).unwrap());
    
    let main_result = config.validate(src.join("main.rs").as_path());
    assert!(main_result.is_none());
    
    let module_result = config.validate(src.join("my-module.rs").as_path());
    assert!(module_result.is_some());
    
    let component_result = config.validate(components.join("MyComponent.rs").as_path());
    assert!(component_result.is_none());
}

#[test]
fn test_ls_lint_naming_constraint_with_all_conventions() {
    // Test all 12 case conventions
    let conventions = vec![
        (CaseConvention::LowerCase, "filename"),
        (CaseConvention::UpperCase, "FILENAME"),
        (CaseConvention::SnakeCase, "file_name"),
        (CaseConvention::CamelCase, "fileName"),
        (CaseConvention::PascalCase, "FileName"),
        (CaseConvention::KebabCase, "file-name"),
        (CaseConvention::ScreamingSnakeCase, "FILE_NAME"),
        (CaseConvention::DotCase, "file.name"),
        (CaseConvention::FlatCase, "filename"),
        (CaseConvention::ScreamingFlatCase, "FILENAME"),
        (CaseConvention::CobolCase, "FILE-NAME"),
        (CaseConvention::TrainCase, "File-Name"),
    ];
    
    for (convention, example) in conventions {
        assert!(
            convention.validate(example),
            "{} should validate '{}'",
            convention.name(),
            example
        );
    }
}
