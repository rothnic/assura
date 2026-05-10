//! Validation engine for the unified config format
//!
//! Integrates policy resolution with constraint validation.

use crate::config::engine::PolicyEngine;
use crate::config::types::{Case, Config, NamingConvention, Severity};
use crate::constraints::error::ValidationFailure;
use crate::constraints::naming::CaseConvention;
use std::path::Path;

/// Validation engine for the new config format
#[derive(Debug)]
pub struct ValidationEngine {
    policy_engine: PolicyEngine,
}

/// Validation result for a single file
#[derive(Debug, Clone)]
pub struct FileValidationResult {
    /// Path that was validated
    pub path: std::path::PathBuf,
    /// Whether validation passed
    pub passed: bool,
    /// List of failures
    pub failures: Vec<ValidationFailure>,
    /// Severity level
    pub severity: Severity,
}

/// Batch validation results
#[derive(Debug, Clone)]
pub struct ValidationResults {
    /// Results for each file
    pub results: Vec<FileValidationResult>,
    /// Total number of files checked
    pub total_files: usize,
    /// Number of files with violations
    pub violation_count: usize,
    /// Number of critical violations
    pub critical_count: usize,
}

impl ValidationEngine {
    /// Create a new validation engine from a config
    pub fn new(config: Config) -> Self {
        let policy_engine = PolicyEngine::new(config.clone());
        Self { policy_engine }
    }

    /// Validate a single file
    pub fn validate_file(&self, path: &Path) -> FileValidationResult {
        let rules = self.policy_engine.resolve(path);
        let mut failures = Vec::new();

        // Validate naming convention
        if let Some(ref naming) = rules.naming {
            if let Some(failure) = self.validate_naming(path, naming) {
                failures.push(failure);
            }
        }

        // Validate max lines
        if let Some(max_lines) = rules.max_lines {
            if let Some(failure) = self.validate_max_lines(path, max_lines) {
                failures.push(failure);
            }
        }

        // Validate max size
        if let Some(ref max_size) = rules.max_size {
            if let Some(failure) = self.validate_max_size(path, max_size) {
                failures.push(failure);
            }
        }

        // Validate require docs
        if rules.require_docs == Some(true) {
            if let Some(failure) = self.validate_require_docs(path) {
                failures.push(failure);
            }
        }

        // Determine severity
        let severity = rules.severity.unwrap_or(Severity::Medium);

        FileValidationResult {
            path: path.to_path_buf(),
            passed: failures.is_empty(),
            failures,
            severity,
        }
    }

    /// Validate multiple files
    pub fn validate_files(&self, paths: &[&Path]) -> ValidationResults {
        let mut results = Vec::new();
        let mut violation_count = 0;
        let mut critical_count = 0;

        for path in paths {
            let result = self.validate_file(path);
            if !result.passed {
                violation_count += 1;
                if result.severity == Severity::Critical {
                    critical_count += 1;
                }
            }
            results.push(result);
        }

        ValidationResults {
            total_files: paths.len(),
            results,
            violation_count,
            critical_count,
        }
    }

    /// Validate naming convention
    fn validate_naming(
        &self,
        path: &Path,
        convention: &NamingConvention,
    ) -> Option<ValidationFailure> {
        let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        let valid = match convention {
            NamingConvention::Single(case) => self.validate_case(filename, case),
            NamingConvention::Multiple(cases) => {
                cases.iter().any(|case| self.validate_case(filename, case))
            }
        };

        if !valid {
            let convention_name = match convention {
                NamingConvention::Single(case) => format!("{:?}", case),
                NamingConvention::Multiple(cases) => cases
                    .iter()
                    .map(|c| format!("{:?}", c))
                    .collect::<Vec<_>>()
                    .join(" or "),
            };

            Some(ValidationFailure::new(
                "naming",
                path,
                format!(
                    "Filename '{}' does not follow {} convention",
                    filename, convention_name
                ),
            ))
        } else {
            None
        }
    }

    /// Validate a single case
    fn validate_case(&self, name: &str, case: &Case) -> bool {
        let convention = case_convention_to_legacy(case);
        convention.validate(name)
    }

    /// Validate max lines
    fn validate_max_lines(&self, path: &Path, max_lines: usize) -> Option<ValidationFailure> {
        if !path.is_file() {
            return None;
        }

        match std::fs::read_to_string(path) {
            Ok(content) => {
                let line_count = content.lines().count();
                if line_count > max_lines {
                    Some(ValidationFailure::new(
                        "max_lines",
                        path,
                        format!("File has {} lines (max: {})", line_count, max_lines),
                    ))
                } else {
                    None
                }
            }
            Err(_) => None, // Can't read file, skip
        }
    }

    /// Validate max size
    fn validate_max_size(&self, path: &Path, max_size: &str) -> Option<ValidationFailure> {
        if !path.is_file() {
            return None;
        }

        let max_bytes = parse_size_string(max_size)?;

        match std::fs::metadata(path) {
            Ok(metadata) => {
                let size = metadata.len();
                if size > max_bytes {
                    Some(ValidationFailure::new(
                        "max_size",
                        path,
                        format!("File size is {} (max: {})", format_size(size), max_size),
                    ))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// Validate require docs
    fn validate_require_docs(&self, path: &Path) -> Option<ValidationFailure> {
        if !path.is_file() {
            return None;
        }

        // Check if file has documentation (Rust-specific: check for `//!` or `///`)
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let has_docs = content.contains("//!") || content.contains("///");
                    if !has_docs {
                        Some(ValidationFailure::new(
                            "require_docs",
                            path,
                            "File is missing documentation (//! or ///)",
                        ))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

/// Convert new Case enum to legacy CaseConvention
fn case_convention_to_legacy(case: &Case) -> CaseConvention {
    match case {
        Case::SnakeCase => CaseConvention::SnakeCase,
        Case::CamelCase => CaseConvention::CamelCase,
        Case::PascalCase => CaseConvention::PascalCase,
        Case::KebabCase => CaseConvention::KebabCase,
        Case::ScreamingSnakeCase => CaseConvention::ScreamingSnakeCase,
        Case::DotCase => CaseConvention::DotCase,
        Case::Flatcase => CaseConvention::FlatCase,
        Case::FlatcaseUpper => CaseConvention::ScreamingFlatCase,
        Case::CobolCase => CaseConvention::CobolCase,
        Case::TrainCase => CaseConvention::TrainCase,
        Case::Lowercase => CaseConvention::LowerCase,
        Case::Uppercase => CaseConvention::UpperCase,
        Case::Regex(_) => CaseConvention::SnakeCase, // Fallback for regex patterns
    }
}

/// Parse size string (e.g., "100KB", "1MB") to bytes
fn parse_size_string(size: &str) -> Option<u64> {
    let size = size.trim();

    // Extract number and unit
    let (num_str, unit): (&str, &str) =
        if let Some(pos) = size.find(|c: char| !c.is_ascii_digit() && c != ' ') {
            let unit_start = pos;
            let num_part: &str = &size[..unit_start];
            let unit_part: &str = size[unit_start..].trim();
            (num_part, unit_part)
        } else {
            (size, "B")
        };

    let num: u64 = num_str.trim().parse().ok()?;

    let unit_upper: String = unit.to_uppercase();
    match unit_upper.as_str() {
        "B" => Some(num),
        "KB" => Some(num * 1024),
        "MB" => Some(num * 1024 * 1024),
        "GB" => Some(num * 1024 * 1024 * 1024),
        "TB" => Some(num * 1024 * 1024 * 1024 * 1024),
        _ => Some(num), // Assume bytes if unit not recognized
    }
}

/// Format bytes as human-readable string
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{InlineRule, PolicyEntry, PolicyNode};

    #[test]
    fn test_validate_naming_snake_case() {
        let config = Config::new().with_policy(PolicyNode::new().with_entry(
            "src/",
            PolicyEntry::InlineRule(InlineRule {
                extensions: Some(vec!["rs".to_string()]),
                naming: Some(NamingConvention::Single(Case::SnakeCase)),
                max_lines: None,
                max_size: None,
                require_docs: None,
                require_test: None,
                message: None,
                severity: None,
            }),
        ));

        let engine = ValidationEngine::new(config);

        let result = engine.validate_file(Path::new("src/my_module.rs"));
        assert!(result.passed);

        let result = engine.validate_file(Path::new("src/my-module.rs"));
        assert!(!result.passed);
    }

    #[test]
    fn test_validate_max_lines() {
        // Use a glob pattern that will match any .rs file
        let config = Config::new().with_policy(PolicyNode::new().with_entry(
            "**/*.rs",
            PolicyEntry::InlineRule(InlineRule {
                extensions: Some(vec!["rs".to_string()]),
                naming: None,
                max_lines: Some(5),
                max_size: None,
                require_docs: None,
                require_test: None,
                message: None,
                severity: None,
            }),
        ));

        let engine = ValidationEngine::new(config);

        // Create a temp file with 10 lines
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_file.rs");
        std::fs::write(
            &file_path,
            "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n",
        )
        .unwrap();

        let result = engine.validate_file(&file_path);
        assert!(!result.passed);
        assert!(result.failures.iter().any(|f| f.constraint == "max_lines"));
    }

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("100"), Some(100));
        assert_eq!(parse_size_string("100B"), Some(100));
        assert_eq!(parse_size_string("10KB"), Some(10 * 1024));
        assert_eq!(parse_size_string("1MB"), Some(1024 * 1024));
        assert_eq!(parse_size_string("1 GB"), Some(1024 * 1024 * 1024));
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(100), "100.00 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
    }
}
