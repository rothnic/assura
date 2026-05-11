//! Unit tests for the parent module.
use super::*;

#[test]
fn test_case_convention_validation() {
    assert!(CaseConvention::SnakeCase.validate("hello_world"));
    assert!(!CaseConvention::SnakeCase.validate("HelloWorld"));
    assert!(!CaseConvention::SnakeCase.validate("hello-world"));

    assert!(CaseConvention::KebabCase.validate("hello-world"));
    assert!(!CaseConvention::KebabCase.validate("hello_world"));

    assert!(CaseConvention::CamelCase.validate("helloWorld"));
    assert!(!CaseConvention::CamelCase.validate("HelloWorld"));
    assert!(!CaseConvention::CamelCase.validate("hello_world"));

    assert!(CaseConvention::PascalCase.validate("HelloWorld"));
    assert!(!CaseConvention::PascalCase.validate("helloWorld"));
}

#[test]
fn test_extension_rule() {
    let rule = ExtensionRule::new()
        .allow_extension("rs")
        .allow_extension("toml");

    assert!(rule.validate("main.rs").is_none());
    assert!(rule.validate("Cargo.toml").is_none());
    assert!(rule.validate("README.md").is_some());
    assert!(rule.validate("no_extension").is_some());
}

#[test]
fn test_extension_rule_optional() {
    let rule = ExtensionRule::new().optional();

    assert!(rule.validate("no_extension").is_none());
}

#[test]
fn test_extension_rule_case() {
    let rule = ExtensionRule::new()
        .allow_extension("rs")
        .allow_mixed_case();

    assert!(rule.validate("main.RS").is_none());

    let rule_lower = ExtensionRule::new().allow_extension("rs");
    assert!(rule_lower.validate("main.RS").is_some());
}

#[test]
fn test_naming_pattern() {
    let pattern = NamingPattern::new("test_pattern", r"^test_.*\.rs$")
        .expect("Valid regex")
        .with_description("Test files must start with 'test_'");

    assert!(pattern.validate("test_something.rs").is_none());
    assert!(pattern.validate("other.rs").is_some());
}

#[test]
fn test_forbidden_pattern() {
    let pattern = NamingPattern::forbidden("no_spaces", r".*\s.*")
        .expect("Valid regex")
        .with_description("No spaces in filenames");

    assert!(pattern.validate("no_spaces").is_none());
    assert!(pattern.validate("has spaces").is_some());
}

#[test]
fn test_naming_constraint() {
    let constraint = NamingConstraint::new()
        .with_case_convention(CaseConvention::KebabCase)
        .with_extension_rule(
            ExtensionRule::new()
                .allow_extension("txt")
                .with_severity(Severity::High),
        );

    let context = ConstraintContext::new();

    // Valid kebab-case with valid extension
    let result = constraint
        .validate(Path::new("/test/my-file.txt"), &context)
        .unwrap();
    assert!(result.passed);

    // Invalid case
    let result = constraint
        .validate(Path::new("/test/my_file.txt"), &context)
        .unwrap();
    assert!(!result.passed);

    // Invalid extension
    let result = constraint
        .validate(Path::new("/test/my-file.doc"), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_rust_config() {
    let constraint = NamingConstraint::rust_config();
    let context = ConstraintContext::new();

    let result = constraint
        .validate(Path::new("/test/my_module.rs"), &context)
        .unwrap();
    assert!(result.passed);

    let result = constraint
        .validate(Path::new("/test/my-module.rs"), &context)
        .unwrap();
    assert!(!result.passed);
}

#[test]
fn test_new_case_conventions() {
    // flatcase - lowercase, no separators
    assert!(CaseConvention::FlatCase.validate("filename"));
    assert!(CaseConvention::FlatCase.validate("myfile123"));
    assert!(!CaseConvention::FlatCase.validate("file_name"));
    assert!(!CaseConvention::FlatCase.validate("file-name"));
    assert!(!CaseConvention::FlatCase.validate("FileName"));
    assert!(!CaseConvention::FlatCase.validate("FILENAME"));

    // FLATCASE - UPPERCASE, no separators
    assert!(CaseConvention::ScreamingFlatCase.validate("FILENAME"));
    assert!(CaseConvention::ScreamingFlatCase.validate("MYFILE123"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("file_name"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("file-name"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("FileName"));
    assert!(!CaseConvention::ScreamingFlatCase.validate("filename"));

    // COBOL-CASE - UPPERCASE with hyphens
    assert!(CaseConvention::CobolCase.validate("FILE-NAME"));
    assert!(CaseConvention::CobolCase.validate("MY-FILE-123"));
    assert!(!CaseConvention::CobolCase.validate("file-name"));
    assert!(!CaseConvention::CobolCase.validate("File-Name"));
    assert!(!CaseConvention::CobolCase.validate("FILE_NAME"));
    assert!(!CaseConvention::CobolCase.validate("FILE--NAME"));
    assert!(!CaseConvention::CobolCase.validate("-FILENAME"));
    assert!(!CaseConvention::CobolCase.validate("FILENAME-"));

    // Train-Case - Title-Case with hyphens
    assert!(CaseConvention::TrainCase.validate("File-Name"));
    assert!(CaseConvention::TrainCase.validate("My-File-123"));
    assert!(CaseConvention::TrainCase.validate("Hello-World"));
    assert!(!CaseConvention::TrainCase.validate("file-name"));
    assert!(!CaseConvention::TrainCase.validate("FILE-NAME"));
    assert!(!CaseConvention::TrainCase.validate("File_Name"));
    assert!(!CaseConvention::TrainCase.validate("File--Name"));
    assert!(!CaseConvention::TrainCase.validate("-FileName"));
    assert!(!CaseConvention::TrainCase.validate("FileName-"));
    assert!(!CaseConvention::TrainCase.validate("fiLe-Name"));
}

// =========================================================================
// COMPREHENSIVE EDGE CASE TESTS
// Based on LS-Lint test patterns for maximum coverage
// =========================================================================

#[test]
fn test_pascal_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::PascalCase.validate("A")); // Single uppercase
    assert!(CaseConvention::PascalCase.validate("Ab")); // Two chars
    assert!(CaseConvention::PascalCase.validate("Button"));
    assert!(CaseConvention::PascalCase.validate("MyComponent"));
    assert!(CaseConvention::PascalCase.validate("A1")); // With number
    assert!(CaseConvention::PascalCase.validate("MyComponent2"));
    assert!(CaseConvention::PascalCase.validate("A1B2C3")); // Mixed alphanumeric
                                                            // Note: "AB" is NOT valid because consecutive uppercase letters are not allowed

    // Invalid cases
    assert!(!CaseConvention::PascalCase.validate("")); // Empty
    assert!(!CaseConvention::PascalCase.validate("AB")); // Consecutive caps not allowed
    assert!(!CaseConvention::PascalCase.validate("a")); // Lowercase start
    assert!(!CaseConvention::PascalCase.validate("button")); // All lowercase
    assert!(!CaseConvention::PascalCase.validate("my_component")); // Underscore
    assert!(!CaseConvention::PascalCase.validate("my-component")); // Hyphen
    assert!(!CaseConvention::PascalCase.validate("1Button")); // Number start
    assert!(!CaseConvention::PascalCase.validate("Button_1")); // Underscore in middle
    assert!(!CaseConvention::PascalCase.validate("Button-1")); // Hyphen in middle
    assert!(!CaseConvention::PascalCase.validate("_Button")); // Underscore start
    assert!(!CaseConvention::PascalCase.validate("Button_")); // Underscore end
}

#[test]
fn test_camel_case_edge_cases() {
    // Valid cases - camelCase only requires lowercase start and no separators
    // (does NOT require uppercase letter like some other conventions)
    assert!(CaseConvention::CamelCase.validate("button")); // All lowercase valid
    assert!(CaseConvention::CamelCase.validate("a")); // Single lowercase valid
    assert!(CaseConvention::CamelCase.validate("abc")); // All lowercase
    assert!(CaseConvention::CamelCase.validate("buttonText")); // Standard case with uppercase
    assert!(CaseConvention::CamelCase.validate("myButton"));
    assert!(CaseConvention::CamelCase.validate("aB")); // With uppercase
    assert!(CaseConvention::CamelCase.validate("myComponent2")); // With number
    assert!(CaseConvention::CamelCase.validate("a1B2C3")); // Mixed alphanumeric

    // Invalid - wrong start
    assert!(!CaseConvention::CamelCase.validate("Button")); // Starts uppercase (PascalCase)
    assert!(!CaseConvention::CamelCase.validate("1button")); // Number start

    // Invalid - separators
    assert!(!CaseConvention::CamelCase.validate("my_button")); // Underscore
    assert!(!CaseConvention::CamelCase.validate("my-button")); // Hyphen

    // Invalid - empty
    assert!(!CaseConvention::CamelCase.validate(""));
}

#[test]
fn test_snake_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::SnakeCase.validate("a")); // Single lowercase
    assert!(CaseConvention::SnakeCase.validate("button")); // All lowercase
    assert!(CaseConvention::SnakeCase.validate("my_component"));
    assert!(CaseConvention::SnakeCase.validate("my_long_component_name"));
    assert!(CaseConvention::SnakeCase.validate("a_b_c")); // Multiple underscores
    assert!(CaseConvention::SnakeCase.validate("component_1")); // With number
    assert!(CaseConvention::SnakeCase.validate("component_1_v2")); // Multiple numbers
    assert!(CaseConvention::SnakeCase.validate("my_1_component")); // Number in middle
    assert!(CaseConvention::SnakeCase.validate("x_y_z_123")); // End with number

    // Invalid cases
    assert!(!CaseConvention::SnakeCase.validate("")); // Empty
    assert!(!CaseConvention::SnakeCase.validate("_private")); // Leading underscore
    assert!(!CaseConvention::SnakeCase.validate("private_")); // Trailing underscore
    assert!(!CaseConvention::SnakeCase.validate("_")); // Just underscore
    assert!(!CaseConvention::SnakeCase.validate("__")); // Double underscore
    assert!(!CaseConvention::SnakeCase.validate("my__component")); // Consecutive underscores
    assert!(!CaseConvention::SnakeCase.validate("MyComponent")); // Uppercase
    assert!(!CaseConvention::SnakeCase.validate("myComponent")); // Mixed case
    assert!(!CaseConvention::SnakeCase.validate("my-component")); // Hyphen
}

#[test]
fn test_kebab_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::KebabCase.validate("a")); // Single lowercase
    assert!(CaseConvention::KebabCase.validate("button")); // All lowercase
    assert!(CaseConvention::KebabCase.validate("my-component"));
    assert!(CaseConvention::KebabCase.validate("my-long-component-name"));
    assert!(CaseConvention::KebabCase.validate("a-b-c")); // Multiple hyphens
    assert!(CaseConvention::KebabCase.validate("component-1")); // With number
    assert!(CaseConvention::KebabCase.validate("component-1-v2")); // Multiple numbers
    assert!(CaseConvention::KebabCase.validate("my-1-component")); // Number in middle
    assert!(CaseConvention::KebabCase.validate("x-y-z-123")); // End with number

    // Invalid cases
    assert!(!CaseConvention::KebabCase.validate("")); // Empty
    assert!(!CaseConvention::KebabCase.validate("-private")); // Leading hyphen
    assert!(!CaseConvention::KebabCase.validate("private-")); // Trailing hyphen
    assert!(!CaseConvention::KebabCase.validate("-")); // Just hyphen
    assert!(!CaseConvention::KebabCase.validate("--")); // Double hyphen
    assert!(!CaseConvention::KebabCase.validate("my--component")); // Consecutive hyphens
    assert!(!CaseConvention::KebabCase.validate("MyComponent")); // Uppercase
    assert!(!CaseConvention::KebabCase.validate("myComponent")); // Mixed case
    assert!(!CaseConvention::KebabCase.validate("my_component")); // Underscore
}

#[test]
fn test_screaming_snake_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::ScreamingSnakeCase.validate("A")); // Single uppercase
    assert!(CaseConvention::ScreamingSnakeCase.validate("BUTTON")); // All uppercase
    assert!(CaseConvention::ScreamingSnakeCase.validate("MY_COMPONENT"));
    assert!(CaseConvention::ScreamingSnakeCase.validate("MY_LONG_COMPONENT_NAME"));
    assert!(CaseConvention::ScreamingSnakeCase.validate("A_B_C")); // Multiple underscores
    assert!(CaseConvention::ScreamingSnakeCase.validate("COMPONENT_1")); // With number
    assert!(CaseConvention::ScreamingSnakeCase.validate("COMPONENT_1_V2")); // Multiple numbers
    assert!(CaseConvention::ScreamingSnakeCase.validate("MY_1_COMPONENT")); // Number in middle
    assert!(CaseConvention::ScreamingSnakeCase.validate("X_Y_Z_123")); // End with number
    assert!(CaseConvention::ScreamingSnakeCase.validate("MAX_VALUE"));
    assert!(CaseConvention::ScreamingSnakeCase.validate("HTTP_STATUS_CODE"));

    // Invalid cases
    assert!(!CaseConvention::ScreamingSnakeCase.validate("")); // Empty
    assert!(!CaseConvention::ScreamingSnakeCase.validate("_PRIVATE")); // Leading underscore
    assert!(!CaseConvention::ScreamingSnakeCase.validate("PRIVATE_")); // Trailing underscore
    assert!(!CaseConvention::ScreamingSnakeCase.validate("_")); // Just underscore
    assert!(!CaseConvention::ScreamingSnakeCase.validate("__")); // Double underscore
    assert!(!CaseConvention::ScreamingSnakeCase.validate("MY__COMPONENT")); // Consecutive underscores
    assert!(!CaseConvention::ScreamingSnakeCase.validate("my_component")); // Lowercase
    assert!(!CaseConvention::ScreamingSnakeCase.validate("MyComponent")); // Mixed case
    assert!(!CaseConvention::ScreamingSnakeCase.validate("MY-COMPONENT")); // Hyphen
}

#[test]
fn test_lowercase_edge_cases() {
    // Valid cases
    assert!(CaseConvention::LowerCase.validate("a")); // Single lowercase
    assert!(CaseConvention::LowerCase.validate("button")); // All lowercase
    assert!(CaseConvention::LowerCase.validate("mycomponent"));
    assert!(CaseConvention::LowerCase.validate("component1")); // With number
    assert!(CaseConvention::LowerCase.validate("a1b2c3")); // Mixed alphanumeric

    // Invalid cases
    assert!(!CaseConvention::LowerCase.validate("")); // Empty
    assert!(!CaseConvention::LowerCase.validate("Button")); // Uppercase
    assert!(!CaseConvention::LowerCase.validate("myComponent")); // Mixed case
    assert!(!CaseConvention::LowerCase.validate("BUTTON")); // All uppercase
}

#[test]
fn test_uppercase_edge_cases() {
    // Valid cases
    assert!(CaseConvention::UpperCase.validate("A")); // Single uppercase
    assert!(CaseConvention::UpperCase.validate("BUTTON")); // All uppercase
    assert!(CaseConvention::UpperCase.validate("MYCOMPONENT"));
    assert!(CaseConvention::UpperCase.validate("COMPONENT1")); // With number
    assert!(CaseConvention::UpperCase.validate("A1B2C3")); // Mixed alphanumeric

    // Invalid cases
    assert!(!CaseConvention::UpperCase.validate("")); // Empty
    assert!(!CaseConvention::UpperCase.validate("button")); // Lowercase
    assert!(!CaseConvention::UpperCase.validate("myComponent")); // Mixed case
}

#[test]
fn test_dot_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::DotCase.validate("file.name"));
    assert!(CaseConvention::DotCase.validate("my.file.name"));
    assert!(CaseConvention::DotCase.validate("component.v2"));

    // Invalid cases
    assert!(!CaseConvention::DotCase.validate("")); // Empty
    assert!(!CaseConvention::DotCase.validate(".file")); // Leading dot
    assert!(!CaseConvention::DotCase.validate("file.")); // Trailing dot
    assert!(!CaseConvention::DotCase.validate("file..name")); // Consecutive dots
    assert!(!CaseConvention::DotCase.validate("File.Name")); // Uppercase
    assert!(!CaseConvention::DotCase.validate("file_name")); // Underscore
}

#[test]
fn test_flat_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::FlatCase.validate("a"));
    assert!(CaseConvention::FlatCase.validate("filename"));
    assert!(CaseConvention::FlatCase.validate("myfile123"));

    // Invalid cases - any separator
    assert!(!CaseConvention::FlatCase.validate("")); // Empty
    assert!(!CaseConvention::FlatCase.validate("file_name")); // Underscore
    assert!(!CaseConvention::FlatCase.validate("file-name")); // Hyphen
    assert!(!CaseConvention::FlatCase.validate("file.name")); // Dot
    assert!(!CaseConvention::FlatCase.validate("FileName")); // Uppercase
    assert!(!CaseConvention::FlatCase.validate("FILENAME")); // All uppercase
}

#[test]
fn test_screaming_flat_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::ScreamingFlatCase.validate("A"));
    assert!(CaseConvention::ScreamingFlatCase.validate("FILENAME"));
    assert!(CaseConvention::ScreamingFlatCase.validate("MYFILE123"));

    // Valid - numbers are allowed
    assert!(CaseConvention::ScreamingFlatCase.validate("FILENAME1"));
    assert!(CaseConvention::ScreamingFlatCase.validate("MYFILE123"));

    // Invalid cases
    assert!(!CaseConvention::ScreamingFlatCase.validate("")); // Empty
    assert!(!CaseConvention::ScreamingFlatCase.validate("file_name")); // Underscore
    assert!(!CaseConvention::ScreamingFlatCase.validate("file-name")); // Hyphen
    assert!(!CaseConvention::ScreamingFlatCase.validate("FileName")); // Mixed case
    assert!(!CaseConvention::ScreamingFlatCase.validate("filename")); // All lowercase
}

#[test]
fn test_cobol_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::CobolCase.validate("A")); // Single uppercase
    assert!(CaseConvention::CobolCase.validate("FILE-NAME"));
    assert!(CaseConvention::CobolCase.validate("MY-FILE-123"));
    assert!(CaseConvention::CobolCase.validate("A-B-C")); // Multiple hyphens

    // Invalid cases
    assert!(!CaseConvention::CobolCase.validate("")); // Empty
    assert!(!CaseConvention::CobolCase.validate("-FILE")); // Leading hyphen
    assert!(!CaseConvention::CobolCase.validate("FILE-")); // Trailing hyphen
    assert!(!CaseConvention::CobolCase.validate("FILE--NAME")); // Consecutive hyphens
    assert!(!CaseConvention::CobolCase.validate("file-name")); // Lowercase
    assert!(!CaseConvention::CobolCase.validate("File-Name")); // Mixed case
    assert!(!CaseConvention::CobolCase.validate("FILE_NAME")); // Underscore
}

#[test]
fn test_train_case_edge_cases() {
    // Valid cases
    assert!(CaseConvention::TrainCase.validate("File-Name"));
    assert!(CaseConvention::TrainCase.validate("My-File-123"));
    assert!(CaseConvention::TrainCase.validate("Hello-World"));
    assert!(CaseConvention::TrainCase.validate("A-B")); // Minimal
    assert!(CaseConvention::TrainCase.validate("A")); // Single char - wait, should fail?

    // Invalid cases
    assert!(!CaseConvention::TrainCase.validate("")); // Empty
    assert!(!CaseConvention::TrainCase.validate("-File-Name")); // Leading hyphen
    assert!(!CaseConvention::TrainCase.validate("File-Name-")); // Trailing hyphen
    assert!(!CaseConvention::TrainCase.validate("File--Name")); // Consecutive hyphens
    assert!(!CaseConvention::TrainCase.validate("file-name")); // Lowercase
    assert!(!CaseConvention::TrainCase.validate("FILE-NAME")); // All uppercase
    assert!(!CaseConvention::TrainCase.validate("File_Name")); // Underscore
    assert!(!CaseConvention::TrainCase.validate("fiLe-Name")); // Wrong case in middle
}

#[test]
fn test_multi_part_extension_naming() {
    // Test that naming conventions work correctly with multi-part extensions
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::PascalCase);
    let context = ConstraintContext::new();

    // Multi-part extensions should still validate the file stem
    let result = constraint
        .validate(Path::new("MyComponent.d.ts"), &context)
        .unwrap();
    assert!(result.passed);

    let result = constraint
        .validate(Path::new("myComponent.d.ts"), &context)
        .unwrap();
    assert!(!result.passed); // Should fail - not PascalCase

    // Snake case with simple extension
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::SnakeCase);

    let result = constraint
        .validate(Path::new("my_component.ts"), &context)
        .unwrap();
    assert!(result.passed);

    let result = constraint
        .validate(Path::new("myComponent.ts"), &context)
        .unwrap();
    assert!(!result.passed);

    // Kebab case with simple extension
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::KebabCase);

    let result = constraint
        .validate(Path::new("my-component.js"), &context)
        .unwrap();
    assert!(result.passed);

    // Camel case with simple extension
    let constraint = NamingConstraint::new().with_case_convention(CaseConvention::CamelCase);

    let result = constraint
        .validate(Path::new("myComponent.tsx"), &context)
        .unwrap();
    assert!(result.passed);

    // Note: Multi-part extensions (.spec.ts, .d.ts) currently only strip the last extension.
    // This is a known limitation - "my_component.spec.ts" validates "my_component.spec"
    // which fails snake_case due to the dot. Full multi-part extension support requires
    // ExtensionRule configuration to recognize compound extensions.
}

#[test]
fn test_empty_and_special_cases() {
    // All conventions should reject empty strings
    assert!(!CaseConvention::PascalCase.validate(""));
    assert!(!CaseConvention::CamelCase.validate(""));
    assert!(!CaseConvention::SnakeCase.validate(""));
    assert!(!CaseConvention::KebabCase.validate(""));
    assert!(!CaseConvention::ScreamingSnakeCase.validate(""));
    assert!(!CaseConvention::LowerCase.validate(""));
    assert!(!CaseConvention::UpperCase.validate(""));
    assert!(!CaseConvention::DotCase.validate(""));
    assert!(!CaseConvention::FlatCase.validate(""));
    assert!(!CaseConvention::ScreamingFlatCase.validate(""));
    assert!(!CaseConvention::CobolCase.validate(""));
    assert!(!CaseConvention::TrainCase.validate(""));
}

#[test]
fn test_numbers_only() {
    // Numbers only - behavior varies by convention
    assert!(CaseConvention::LowerCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::UpperCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::SnakeCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::KebabCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::ScreamingSnakeCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::FlatCase.validate("123")); // Numbers allowed
    assert!(CaseConvention::ScreamingFlatCase.validate("123")); // Numbers allowed

    // These require letters (start with letter)
    assert!(!CaseConvention::PascalCase.validate("123")); // Needs uppercase start
    assert!(!CaseConvention::CamelCase.validate("123")); // Needs lowercase start
    assert!(!CaseConvention::TrainCase.validate("123")); // Needs specific case pattern
                                                         // Note: CobolCase("123") and DotCase("123") are actually valid - numbers allowed
}
