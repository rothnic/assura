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
mod constraints_tests;
