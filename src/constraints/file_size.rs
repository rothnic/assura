//! File size constraint
//!
//! Validates file size against configurable limits with pattern matching
//! for file selection.

use std::path::Path;

use super::error::{ConstraintError, ConstraintResult, ValidationFailure, ValidationFailures};
use super::r#trait::{Constraint, ConstraintContext, ConstraintOutput};
use super::severity::Severity;

/// File size limit configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSizeLimit {
    /// No limit
    Unlimited,
    /// Exact byte count
    Bytes(u64),
    /// Kilobytes
    Kilobytes(u64),
    /// Megabytes
    Megabytes(u64),
    /// Gigabytes
    Gigabytes(u64),
}

impl FileSizeLimit {
    /// Get the limit in bytes
    pub fn as_bytes(self) -> Option<u64> {
        match self {
            FileSizeLimit::Unlimited => None,
            FileSizeLimit::Bytes(b) => Some(b),
            FileSizeLimit::Kilobytes(kb) => Some(kb * 1024),
            FileSizeLimit::Megabytes(mb) => Some(mb * 1024 * 1024),
            FileSizeLimit::Gigabytes(gb) => Some(gb * 1024 * 1024 * 1024),
        }
    }

    /// Format as human-readable string
    pub fn format(self) -> String {
        match self {
            FileSizeLimit::Unlimited => "unlimited".to_string(),
            FileSizeLimit::Bytes(b) if b < 1024 => format!("{} B", b),
            FileSizeLimit::Bytes(b) => format!("{:.2} KB", b as f64 / 1024.0),
            FileSizeLimit::Kilobytes(kb) => format!("{} KB", kb),
            FileSizeLimit::Megabytes(mb) => format!("{} MB", mb),
            FileSizeLimit::Gigabytes(gb) => format!("{} GB", gb),
        }
    }
}

impl Default for FileSizeLimit {
    fn default() -> Self {
        FileSizeLimit::Unlimited
    }
}

/// A rule for file size validation
#[derive(Debug, Clone)]
pub struct FileSizeRule {
    /// Name of the rule
    pub name: String,
    /// File patterns to match (glob patterns)
    pub patterns: Vec<String>,
    /// Maximum file size
    pub max_size: FileSizeLimit,
    /// Minimum file size (optional)
    pub min_size: Option<FileSizeLimit>,
    /// Severity for violations
    pub severity: Severity,
    /// Whether to ignore directories
    pub ignore_directories: bool,
}

impl FileSizeRule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            patterns: Vec::new(),
            max_size: FileSizeLimit::Unlimited,
            min_size: None,
            severity: Severity::Medium,
            ignore_directories: true,
        }
    }

    /// Add a file pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Set maximum size
    pub fn max_size(mut self, limit: FileSizeLimit) -> Self {
        self.max_size = limit;
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, limit: FileSizeLimit) -> Self {
        self.min_size = Some(limit);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Include directories in validation
    pub fn include_directories(mut self) -> Self {
        self.ignore_directories = false;
        self
    }

    /// Check if this rule applies to a path
    pub fn applies_to(&self, path: &Path) -> bool {
        // Check if it's a directory
        if self.ignore_directories && path.is_dir() {
            return false;
        }

        // Check patterns
        if self.patterns.is_empty() {
            return true;
        }

        let path_str = path.to_string_lossy();
        for pattern in &self.patterns {
            if matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Validate a file against this rule
    pub fn validate(&self, path: &Path) -> ConstraintResult<Option<ValidationFailure>> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            ConstraintError::io(path, format!("Failed to read file metadata: {}", e))
        })?;

        let size = metadata.len();

        // Check minimum size
        if let Some(min) = self.min_size.and_then(|l| l.as_bytes()) {
            if size < min {
                return Ok(Some(ValidationFailure::new(
                    &self.name,
                    path,
                    format!(
                        "File is too small: {} (minimum: {})",
                        format_size(size),
                        self.min_size.unwrap().format()
                    ),
                )));
            }
        }

        // Check maximum size
        if let Some(max) = self.max_size.as_bytes() {
            if size > max {
                return Ok(Some(ValidationFailure::new(
                    &self.name,
                    path,
                    format!(
                        "File is too large: {} (maximum: {})",
                        format_size(size),
                        self.max_size.format()
                    ),
                )
                .with_suggestion(format!("Consider splitting the file or using compression"))));
            }
        }

        Ok(None)
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

/// Check if a path matches a glob pattern
fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern == "*" || pattern == "**/*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[1..];
        return path.ends_with(ext);
    }

    if pattern.contains('*') || pattern.contains('?') {
        match glob::Pattern::new(pattern) {
            Ok(p) => return p.matches(path),
            Err(_) => return false,
        }
    }

    path.contains(pattern)
}

/// File size constraint that applies multiple rules
#[derive(Debug)]
pub struct FileSizeConstraint {
    name: String,
    rules: Vec<FileSizeRule>,
    default_severity: Severity,
}

impl FileSizeConstraint {
    pub fn new() -> Self {
        Self {
            name: "file_size".to_string(),
            rules: Vec::new(),
            default_severity: Severity::Medium,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn add_rule(mut self, rule: FileSizeRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_default_severity(mut self, severity: Severity) -> Self {
        self.default_severity = severity;
        self
    }

    /// Create a default configuration with sensible limits
    pub fn default_config() -> Self {
        Self::new()
            .add_rule(
                FileSizeRule::new("max_source_file")
                    .with_pattern("*.rs")
                    .with_pattern("*.js")
                    .with_pattern("*.ts")
                    .with_pattern("*.py")
                    .max_size(FileSizeLimit::Kilobytes(100))
                    .with_severity(Severity::Medium),
            )
            .add_rule(
                FileSizeRule::new("max_binary")
                    .with_pattern("*.bin")
                    .with_pattern("*.exe")
                    .max_size(FileSizeLimit::Megabytes(50))
                    .with_severity(Severity::High),
            )
            .add_rule(
                FileSizeRule::new("max_asset")
                    .with_pattern("*.png")
                    .with_pattern("*.jpg")
                    .with_pattern("*.gif")
                    .with_pattern("*.svg")
                    .max_size(FileSizeLimit::Megabytes(5))
                    .with_severity(Severity::Low),
            )
    }
}

impl Default for FileSizeConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint for FileSizeConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates file sizes against configured limits"
    }

    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let mut failures = ValidationFailures::new();
        let mut matched = false;

        // Find applicable rules
        for rule in &self.rules {
            if rule.applies_to(path) {
                matched = true;
                match rule.validate(path) {
                    Ok(Some(failure)) => {
                        failures.add(failure);
                        if context.fail_fast {
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        // If we can't read metadata, report as failure
                        failures.add(ValidationFailure::new(
                            &self.name,
                            path,
                            format!("Failed to validate: {}", e),
                        ));
                        if context.fail_fast {
                            break;
                        }
                    }
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;

        // If no rules matched, check if this is a file and apply default rule
        if !matched && path.is_file() {
            // Default: check for very large files (> 10MB)
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                let ten_mb = 10 * 1024 * 1024;
                if size > ten_mb {
                    failures.add(ValidationFailure::new(
                        &self.name,
                        path,
                        format!(
                            "File exceeds default size limit: {} (> 10 MB)",
                            format_size(size)
                        ),
                    ));
                }
            }
        }

        let passed = failures.is_empty();
        let severity = if passed {
            self.default_severity
        } else {
            // Use highest severity from failures
            self.rules
                .iter()
                .filter(|r| r.applies_to(path))
                .map(|r| r.severity)
                .max()
                .unwrap_or(self.default_severity)
        };

        let adjusted_severity = self.severity_for_maturity(context.maturity_level());
        let effective_severity = severity.max(adjusted_severity);

        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_severity(effective_severity)
            .with_duration(duration)
            .with_failures(failures))
    }

    fn applies_to(&self, path: &Path) -> bool {
        // Applies to all files (directories don't have meaningful size)
        path.is_file()
    }

    fn default_severity(&self) -> Severity {
        self.default_severity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_file_size_limit() {
        assert_eq!(FileSizeLimit::Bytes(1024).as_bytes(), Some(1024));
        assert_eq!(FileSizeLimit::Kilobytes(1).as_bytes(), Some(1024));
        assert_eq!(FileSizeLimit::Megabytes(1).as_bytes(), Some(1024 * 1024));
        assert_eq!(FileSizeLimit::Unlimited.as_bytes(), None);
    }

    #[test]
    fn test_file_size_limit_format() {
        assert_eq!(FileSizeLimit::Bytes(512).format(), "512 B");
        assert_eq!(FileSizeLimit::Kilobytes(1).format(), "1 KB");
        assert_eq!(FileSizeLimit::Megabytes(5).format(), "5 MB");
    }

    #[test]
    fn test_file_size_rule_pattern() {
        let rule = FileSizeRule::new("test")
            .with_pattern("*.rs")
            .with_pattern("*.toml");

        assert!(rule.applies_to(Path::new("/test/main.rs")));
        assert!(rule.applies_to(Path::new("/test/Cargo.toml")));
        assert!(!rule.applies_to(Path::new("/test/main.txt")));
        assert!(!rule.applies_to(Path::new("/test/dir/")));
    }

    #[test]
    fn test_file_size_rule_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        let path = temp_file.path();

        // Rule with max size larger than file
        let rule = FileSizeRule::new("test").max_size(FileSizeLimit::Kilobytes(10));
        let result = rule.validate(path).unwrap();
        assert!(result.is_none());

        // Rule with max size smaller than file (should fail)
        let rule = FileSizeRule::new("test").max_size(FileSizeLimit::Bytes(5));
        let result = rule.validate(path).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512.00 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn test_file_size_constraint() {
        let constraint = FileSizeConstraint::new()
            .add_rule(
                FileSizeRule::new("test_rule")
                    .with_pattern("*.txt")
                    .max_size(FileSizeLimit::Kilobytes(1))
                    .with_severity(Severity::High),
            );

        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        temp_file.write_all(b"Small content").unwrap();

        let context = ConstraintContext::new();
        let result = constraint.validate(temp_file.path(), &context).unwrap();

        assert!(result.passed);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_file_size_constraint_too_large() {
        let constraint = FileSizeConstraint::new()
            .add_rule(
                FileSizeRule::new("test_rule")
                    .with_pattern("*.txt")
                    .max_size(FileSizeLimit::Bytes(5))
                    .with_severity(Severity::High),
            );

        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        temp_file.write_all(b"This is more than 5 bytes").unwrap();

        let context = ConstraintContext::new();
        let result = constraint.validate(temp_file.path(), &context).unwrap();

        assert!(!result.passed);
        assert_eq!(result.failures.len(), 1);
    }
}
