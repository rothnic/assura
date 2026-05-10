//! Constraint Validation Engine
//!
//! Validates individual constraints against files.
//! Follows Constitution: separate concerns, testable units.

use crate::config::ast::{Constraint, NamingConvention, Range};
use std::path::Path;

/// Result of constraint validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: Option<String>,
    pub constraint_type: String,
}

impl ValidationResult {
    pub fn pass(constraint_type: &str) -> Self {
        Self {
            passed: true,
            message: None,
            constraint_type: constraint_type.to_string(),
        }
    }

    pub fn fail(constraint_type: &str, message: String) -> Self {
        Self {
            passed: false,
            message: Some(message),
            constraint_type: constraint_type.to_string(),
        }
    }
}

/// Validates constraints against a file
pub struct ConstraintValidator;

impl ConstraintValidator {
    /// Validate a single constraint
    pub fn validate(
        constraint: &Constraint,
        file_path: &Path,
        file_content: Option<&str>,
    ) -> ValidationResult {
        match constraint {
            Constraint::Naming(convention) => Self::validate_naming(convention, file_path),
            Constraint::Lines { lines } => {
                if let Some(content) = file_content {
                    Self::validate_lines(lines, content)
                } else {
                    ValidationResult::pass("lines")
                }
            }
            Constraint::Size { size } => Self::validate_size(size, file_path),
            Constraint::Exists { exists: _ } => {
                // Exists is handled at directory level, not file level
                ValidationResult::pass("exists")
            }
            Constraint::ConstraintsArray(constraints) => {
                // Validate all constraints in array (AND logic)
                for constraint in constraints {
                    let result = Self::validate(constraint, file_path, file_content);
                    if !result.passed {
                        return result;
                    }
                }
                ValidationResult::pass("constraints")
            }
        }
    }

    /// Validate naming convention
    fn validate_naming(convention: &NamingConvention, file_path: &Path) -> ValidationResult {
        let file_name = file_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        let is_valid = match convention {
            NamingConvention::PascalCase => Self::is_pascal_case(file_name),
            NamingConvention::CamelCase => Self::is_camel_case(file_name),
            NamingConvention::SnakeCase => Self::is_snake_case(file_name),
            NamingConvention::KebabCase => Self::is_kebab_case(file_name),
            NamingConvention::ScreamingSnakeCase => Self::is_screaming_snake_case(file_name),
            NamingConvention::Lowercase => {
                !file_name.is_empty()
                    && file_name
                        .chars()
                        .all(|c| c.is_lowercase() || c.is_numeric())
            }
            NamingConvention::Uppercase => {
                !file_name.is_empty()
                    && file_name
                        .chars()
                        .all(|c| c.is_uppercase() || c.is_numeric())
            }
        };

        if is_valid {
            ValidationResult::pass("naming")
        } else {
            ValidationResult::fail(
                "naming",
                format!(
                    "File '{}' does not match {:?} naming convention",
                    file_name, convention
                ),
            )
        }
    }

    /// Validate line count
    fn validate_lines(range: &Range, content: &str) -> ValidationResult {
        let line_count = content.lines().count() as u64;

        let is_valid = match range {
            Range::Exact(expected) => line_count == *expected,
            Range::RangeString(s) => Self::check_range(line_count, s),
        };

        if is_valid {
            ValidationResult::pass("lines")
        } else {
            ValidationResult::fail(
                "lines",
                format!("File has {} lines, expected {}", line_count, range),
            )
        }
    }

    /// Validate file size
    fn validate_size(size_str: &str, file_path: &Path) -> ValidationResult {
        // Parse size string (e.g., "1MB", "500KB")
        let (value, unit) = Self::parse_size(size_str);

        // Get actual file size
        let actual_size = match std::fs::metadata(file_path) {
            Ok(meta) => meta.len(),
            Err(_) => {
                return ValidationResult::fail(
                    "size",
                    format!("Cannot read file size for {:?}", file_path),
                );
            }
        };

        let max_bytes = match unit.as_str() {
            "B" => value,
            "KB" => value * 1024,
            "MB" => value * 1024 * 1024,
            "GB" => value * 1024 * 1024 * 1024,
            _ => value, // Assume bytes if no unit
        };

        if actual_size <= max_bytes {
            ValidationResult::pass("size")
        } else {
            ValidationResult::fail(
                "size",
                format!(
                    "File size is {} bytes, max allowed is {} bytes",
                    actual_size, max_bytes
                ),
            )
        }
    }

    // Naming convention helpers
    fn is_pascal_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        // First char uppercase, rest alphanumeric
        s.chars().next().is_some_and(|c| c.is_uppercase()) && s.chars().all(|c| c.is_alphanumeric())
    }

    fn is_camel_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        // First char lowercase, contains uppercase
        s.chars().next().is_some_and(|c| c.is_lowercase())
            && s.chars().any(|c| c.is_uppercase())
            && s.chars().all(|c| c.is_alphanumeric())
    }

    fn is_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        s.chars()
            .all(|c| c.is_lowercase() || c == '_' || c.is_numeric())
            && !s.starts_with('_')
            && !s.ends_with('_')
            && !s.contains("__")
    }

    fn is_kebab_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        s.chars()
            .all(|c| c.is_lowercase() || c == '-' || c.is_numeric())
            && !s.starts_with('-')
            && !s.ends_with('-')
            && !s.contains("--")
    }

    fn is_screaming_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        s.chars()
            .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
            && !s.starts_with('_')
            && !s.ends_with('_')
            && !s.contains("__")
    }

    // Range checking
    fn check_range(value: u64, range_str: &str) -> bool {
        if range_str.contains("..") {
            let parts: Vec<&str> = range_str.split("..").collect();
            match parts.len() {
                2 => {
                    let min = parts[0].parse::<u64>().ok();
                    let max = parts[1].parse::<u64>().ok();
                    match (min, max) {
                        (Some(min), Some(max)) => value >= min && value <= max,
                        (None, Some(max)) => value <= max,
                        (Some(min), None) => value >= min,
                        (None, None) => true,
                    }
                }
                _ => false,
            }
        } else {
            range_str.parse::<u64>() == Ok(value)
        }
    }

    // Size parsing
    fn parse_size(s: &str) -> (u64, String) {
        let s = s.trim();

        // Find where digits end
        let digits_end = s
            .chars()
            .position(|c| !c.is_numeric() && c != '.')
            .unwrap_or(s.len());

        let value_str = &s[..digits_end];
        let unit = &s[digits_end..];

        let value = value_str.parse::<f64>().unwrap_or(0.0) as u64;
        let unit = if unit.is_empty() { "B" } else { unit };

        (value, unit.to_string())
    }
}

#[cfg(test)]
mod tests {
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
            ConstraintValidator::validate_naming(
                &NamingConvention::Lowercase,
                Path::new("button.rs")
            )
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
            ConstraintValidator::validate_naming(
                &NamingConvention::Lowercase,
                Path::new("a1b2c3.rs")
            )
            .passed
        );

        // Invalid cases
        assert!(
            !ConstraintValidator::validate_naming(&NamingConvention::Lowercase, Path::new(""))
                .passed
        ); // Empty
        assert!(
            !ConstraintValidator::validate_naming(
                &NamingConvention::Lowercase,
                Path::new("Button.rs")
            )
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
            !ConstraintValidator::validate_naming(
                &NamingConvention::Lowercase,
                Path::new("BUTTON.rs")
            )
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
            ConstraintValidator::validate_naming(
                &NamingConvention::Uppercase,
                Path::new("BUTTON.rs")
            )
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
            ConstraintValidator::validate_naming(
                &NamingConvention::Uppercase,
                Path::new("A1B2C3.rs")
            )
            .passed
        );

        // Invalid cases
        assert!(
            !ConstraintValidator::validate_naming(&NamingConvention::Uppercase, Path::new(""))
                .passed
        ); // Empty
        assert!(
            !ConstraintValidator::validate_naming(
                &NamingConvention::Uppercase,
                Path::new("button.rs")
            )
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
}
