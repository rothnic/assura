//! Unit tests for the parent module.
use super::*;

// =========================================================================
// NAMING CONVENTION EDGE CASE TESTS
// Following LS-Lint test patterns - comprehensive coverage
// =========================================================================

#[test]
fn test_pascal_case_comprehensive() {
    // Valid cases
    assert!(ConstraintValidator::is_pascal_case("A")); // Single uppercase
    assert!(ConstraintValidator::is_pascal_case("Ab")); // Two chars
    assert!(ConstraintValidator::is_pascal_case("Button"));
    assert!(ConstraintValidator::is_pascal_case("MyComponent"));
    assert!(ConstraintValidator::is_pascal_case("AB")); // All caps
    assert!(ConstraintValidator::is_pascal_case("MyURLParser")); // Consecutive caps
    assert!(ConstraintValidator::is_pascal_case("A1")); // With number
    assert!(ConstraintValidator::is_pascal_case("MyComponent2"));
    assert!(ConstraintValidator::is_pascal_case("A1B2C3")); // Mixed alphanumeric

    // Invalid cases
    assert!(!ConstraintValidator::is_pascal_case("")); // Empty
    assert!(!ConstraintValidator::is_pascal_case("a")); // Lowercase start
    assert!(!ConstraintValidator::is_pascal_case("button")); // All lowercase
    assert!(!ConstraintValidator::is_pascal_case("my_component")); // Underscore
    assert!(!ConstraintValidator::is_pascal_case("my-component")); // Hyphen
    assert!(!ConstraintValidator::is_pascal_case("1Button")); // Number start
    assert!(!ConstraintValidator::is_pascal_case("Button_1")); // Underscore in middle
    assert!(!ConstraintValidator::is_pascal_case("Button-1")); // Hyphen in middle
    assert!(!ConstraintValidator::is_pascal_case("my URL")); // Space
    assert!(!ConstraintValidator::is_pascal_case("My@Component")); // Special char
    assert!(!ConstraintValidator::is_pascal_case("_Button")); // Underscore start
    assert!(!ConstraintValidator::is_pascal_case("Button_")); // Underscore end
}

#[test]
fn test_camel_case_comprehensive() {
    // Valid cases - camelCase requires lowercase start AND at least one uppercase
    assert!(ConstraintValidator::is_camel_case("buttonText")); // Standard case
    assert!(ConstraintValidator::is_camel_case("myButton"));
    assert!(ConstraintValidator::is_camel_case("aB")); // Minimal valid
    assert!(ConstraintValidator::is_camel_case("getHTTPResponse")); // Consecutive caps
    assert!(ConstraintValidator::is_camel_case("parseXMLDocument"));
    assert!(ConstraintValidator::is_camel_case("myComponent2")); // With number
    assert!(ConstraintValidator::is_camel_case("a1B2C3")); // Mixed alphanumeric

    // Invalid cases - must have at least one uppercase
    assert!(!ConstraintValidator::is_camel_case("button")); // All lowercase, no uppercase
    assert!(!ConstraintValidator::is_camel_case("a")); // Single lowercase
    assert!(!ConstraintValidator::is_camel_case("abc")); // All lowercase

    // Invalid - wrong start (camelCase must start with lowercase)
    assert!(!ConstraintValidator::is_camel_case("Button")); // Starts uppercase (this is PascalCase, not camelCase)
    assert!(!ConstraintValidator::is_camel_case("1button")); // Number start

    // Invalid - separators
    assert!(!ConstraintValidator::is_camel_case("my_button")); // Underscore
    assert!(!ConstraintValidator::is_camel_case("my-button")); // Hyphen
    assert!(!ConstraintValidator::is_camel_case("button text")); // Space

    // Invalid - special chars
    assert!(!ConstraintValidator::is_camel_case("button@text"));
    assert!(!ConstraintValidator::is_camel_case("button.text"));

    // Invalid - empty
    assert!(!ConstraintValidator::is_camel_case(""));
}

#[test]
fn test_snake_case_comprehensive() {
    // Valid cases
    assert!(ConstraintValidator::is_snake_case("a")); // Single lowercase
    assert!(ConstraintValidator::is_snake_case("button")); // All lowercase
    assert!(ConstraintValidator::is_snake_case("my_component"));
    assert!(ConstraintValidator::is_snake_case("my_long_component_name"));
    assert!(ConstraintValidator::is_snake_case("a_b_c")); // Multiple underscores
    assert!(ConstraintValidator::is_snake_case("component_1")); // With number
    assert!(ConstraintValidator::is_snake_case("component_1_v2")); // Multiple numbers
    assert!(ConstraintValidator::is_snake_case("my_1_component")); // Number in middle
    assert!(ConstraintValidator::is_snake_case("x_y_z_123")); // End with number

    // Invalid cases
    assert!(!ConstraintValidator::is_snake_case("")); // Empty
    assert!(!ConstraintValidator::is_snake_case("_private")); // Leading underscore
    assert!(!ConstraintValidator::is_snake_case("private_")); // Trailing underscore
    assert!(!ConstraintValidator::is_snake_case("_")); // Just underscore
    assert!(!ConstraintValidator::is_snake_case("__")); // Double underscore
    assert!(!ConstraintValidator::is_snake_case("my__component")); // Consecutive underscores
    assert!(!ConstraintValidator::is_snake_case("my_component_")); // Trailing underscore
    assert!(!ConstraintValidator::is_snake_case("_my_component")); // Leading underscore
    assert!(!ConstraintValidator::is_snake_case("MyComponent")); // Uppercase
    assert!(!ConstraintValidator::is_snake_case("myComponent")); // Mixed case
    assert!(!ConstraintValidator::is_snake_case("my-component")); // Hyphen
    assert!(!ConstraintValidator::is_snake_case("my component")); // Space
    assert!(!ConstraintValidator::is_snake_case("my@component")); // Special char
}

#[test]
fn test_kebab_case_comprehensive() {
    // Valid cases
    assert!(ConstraintValidator::is_kebab_case("a")); // Single lowercase
    assert!(ConstraintValidator::is_kebab_case("button")); // All lowercase
    assert!(ConstraintValidator::is_kebab_case("my-component"));
    assert!(ConstraintValidator::is_kebab_case("my-long-component-name"));
    assert!(ConstraintValidator::is_kebab_case("a-b-c")); // Multiple hyphens
    assert!(ConstraintValidator::is_kebab_case("component-1")); // With number
    assert!(ConstraintValidator::is_kebab_case("component-1-v2")); // Multiple numbers
    assert!(ConstraintValidator::is_kebab_case("my-1-component")); // Number in middle
    assert!(ConstraintValidator::is_kebab_case("x-y-z-123")); // End with number

    // Invalid cases
    assert!(!ConstraintValidator::is_kebab_case("")); // Empty
    assert!(!ConstraintValidator::is_kebab_case("-private")); // Leading hyphen
    assert!(!ConstraintValidator::is_kebab_case("private-")); // Trailing hyphen
    assert!(!ConstraintValidator::is_kebab_case("-")); // Just hyphen
    assert!(!ConstraintValidator::is_kebab_case("--")); // Double hyphen
    assert!(!ConstraintValidator::is_kebab_case("my--component")); // Consecutive hyphens
    assert!(!ConstraintValidator::is_kebab_case("my-component-")); // Trailing hyphen
    assert!(!ConstraintValidator::is_kebab_case("-my-component")); // Leading hyphen
    assert!(!ConstraintValidator::is_kebab_case("MyComponent")); // Uppercase
    assert!(!ConstraintValidator::is_kebab_case("myComponent")); // Mixed case
    assert!(!ConstraintValidator::is_kebab_case("my_component")); // Underscore
    assert!(!ConstraintValidator::is_kebab_case("my component")); // Space
    assert!(!ConstraintValidator::is_kebab_case("my@component")); // Special char
}

#[test]
fn test_screaming_snake_case_comprehensive() {
    // Valid cases
    assert!(ConstraintValidator::is_screaming_snake_case("A")); // Single uppercase
    assert!(ConstraintValidator::is_screaming_snake_case("BUTTON")); // All uppercase
    assert!(ConstraintValidator::is_screaming_snake_case("MY_COMPONENT"));
    assert!(ConstraintValidator::is_screaming_snake_case(
        "MY_LONG_COMPONENT_NAME"
    ));
    assert!(ConstraintValidator::is_screaming_snake_case("A_B_C")); // Multiple underscores
    assert!(ConstraintValidator::is_screaming_snake_case("COMPONENT_1")); // With number
    assert!(ConstraintValidator::is_screaming_snake_case(
        "COMPONENT_1_V2"
    )); // Multiple numbers
    assert!(ConstraintValidator::is_screaming_snake_case(
        "MY_1_COMPONENT"
    )); // Number in middle
    assert!(ConstraintValidator::is_screaming_snake_case("X_Y_Z_123")); // End with number
    assert!(ConstraintValidator::is_screaming_snake_case("MAX_VALUE"));
    assert!(ConstraintValidator::is_screaming_snake_case(
        "HTTP_STATUS_CODE"
    ));

    // Invalid cases
    assert!(!ConstraintValidator::is_screaming_snake_case("")); // Empty
    assert!(!ConstraintValidator::is_screaming_snake_case("_PRIVATE")); // Leading underscore
    assert!(!ConstraintValidator::is_screaming_snake_case("PRIVATE_")); // Trailing underscore
    assert!(!ConstraintValidator::is_screaming_snake_case("_")); // Just underscore
    assert!(!ConstraintValidator::is_screaming_snake_case("__")); // Double underscore
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "MY__COMPONENT"
    )); // Consecutive underscores
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "MY_COMPONENT_"
    )); // Trailing underscore
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "_MY_COMPONENT"
    )); // Leading underscore
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "my_component"
    )); // Lowercase
    assert!(!ConstraintValidator::is_screaming_snake_case("MyComponent")); // Mixed case
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "MY-COMPONENT"
    )); // Hyphen
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "MY COMPONENT"
    )); // Space
    assert!(!ConstraintValidator::is_screaming_snake_case(
        "MY@COMPONENT"
    )); // Special char
}

#[test]
fn test_lowercase_comprehensive() {
    // Valid cases
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("a.rs"))
            .passed
    );
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("button.rs"))
            .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::Lowercase,
            Path::new("mycomponent.rs")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::Lowercase,
            Path::new("component1.rs")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("a1b2c3.rs"))
            .passed
    );

    // Invalid cases
    assert!(
        !ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("")).passed
    ); // Empty
    assert!(
        !ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("Button.rs"))
            .passed
    ); // Uppercase
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Lowercase,
            Path::new("myComponent.rs")
        )
        .passed
    ); // Mixed case
    assert!(
        !ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new("BUTTON.rs"))
            .passed
    ); // All uppercase
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Lowercase,
            Path::new("my_component.rs")
        )
        .passed
    ); // Underscore
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Lowercase,
            Path::new("my-component.rs")
        )
        .passed
    ); // Hyphen
}

#[test]
fn test_uppercase_comprehensive() {
    // Valid cases
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new("A.rs"))
            .passed
    );
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new("BUTTON.rs"))
            .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::Uppercase,
            Path::new("MYCOMPONENT.rs")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::Uppercase,
            Path::new("COMPONENT1.rs")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new("A1B2C3.rs"))
            .passed
    );

    // Invalid cases
    assert!(
        !ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new("")).passed
    ); // Empty
    assert!(
        !ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new("button.rs"))
            .passed
    ); // Lowercase
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Uppercase,
            Path::new("myComponent.rs")
        )
        .passed
    ); // Mixed case
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Uppercase,
            Path::new("my_component.rs")
        )
        .passed
    ); // Underscore
    assert!(
        !ConstraintValidator::validate_naming(
            &NamingConvention::Uppercase,
            Path::new("MY-COMPONENT.rs")
        )
        .passed
    ); // Hyphen
}

#[test]
fn test_naming_with_multi_part_extensions() {
    // Note: Multi-part extensions (.d.ts, .spec.ts) are handled by treating the entire
    // filename stem (everything before the last extension) as the name to validate.
    // So "MyComponent.d.ts" validates "MyComponent.d", which contains a dot.
    // This is a known limitation - multi-part extension awareness requires additional
    // ExtensionRule configuration.

    // For now, test with simple extensions
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::PascalCase,
            Path::new("MyComponent.ts")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::SnakeCase,
            Path::new("my_component.js")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::KebabCase,
            Path::new("my-component.rs")
        )
        .passed
    );
    assert!(
        ConstraintValidator::validate_naming(
            &NamingConvention::CamelCase,
            Path::new("myComponent.go")
        )
        .passed
    );
}

#[test]
fn test_range_checking() {
    assert!(ConstraintValidator::check_range(100, "..400"));
    assert!(ConstraintValidator::check_range(100, "100.."));
    assert!(ConstraintValidator::check_range(100, "50..200"));
    assert!(!ConstraintValidator::check_range(500, "..400"));
    assert!(!ConstraintValidator::check_range(50, "100.."));
}

#[test]
fn test_size_parsing() {
    assert_eq!(
        ConstraintValidator::parse_size("1MB"),
        (1, "MB".to_string())
    );
    assert_eq!(
        ConstraintValidator::parse_size("500KB"),
        (500, "KB".to_string())
    );
    assert_eq!(
        ConstraintValidator::parse_size("100"),
        (100, "B".to_string())
    );
}
