//! Directory validation for LS-Lint parity
//!
//! Provides directory-only naming rules, directory vs file distinction,
//! and special directory handling (exclusions for .git, node_modules, etc.)

use std::collections::HashSet;
use std::path::Path;

use crate::constraints::error::{ConstraintResult, ValidationFailure};
use crate::constraints::naming::CaseConvention;
use crate::constraints::r#trait::{Constraint, ConstraintContext, ConstraintOutput};
use crate::constraints::severity::Severity;

/// Configuration for directory validation
#[derive(Debug, Clone)]
pub struct DirectoryValidationConfig {
    /// Directories to exclude from validation (e.g., .git, node_modules)
    pub excluded_dirs: HashSet<String>,
    /// Whether to validate recursively
    pub recursive: bool,
    /// Whether to distinguish between directory and file rules
    pub distinguish_types: bool,
    /// Severity for directory violations
    pub severity: Severity,
}

impl DirectoryValidationConfig {
    pub fn new() -> Self {
        let mut excluded = HashSet::new();
        excluded.insert(".git".to_string());
        excluded.insert("node_modules".to_string());
        excluded.insert("target".to_string());
        excluded.insert(".cache".to_string());
        excluded.insert("dist".to_string());
        excluded.insert("build".to_string());

        Self {
            excluded_dirs: excluded,
            recursive: true,
            distinguish_types: true,
            severity: Severity::Medium,
        }
    }

    pub fn with_excluded_dir(mut self, dir: impl Into<String>) -> Self {
        self.excluded_dirs.insert(dir.into());
        self
    }

    pub fn without_excluded_dir(mut self, dir: &str) -> Self {
        self.excluded_dirs.remove(dir);
        self
    }

    pub fn non_recursive(mut self) -> Self {
        self.recursive = false;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Check if a directory should be excluded
    pub fn is_excluded(&self, dir_name: &str) -> bool {
        self.excluded_dirs.contains(dir_name)
    }
}

impl Default for DirectoryValidationConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// A rule specific to directory naming
#[derive(Debug, Clone)]
pub struct DirectoryRule {
    /// The naming convention to enforce
    pub convention: CaseConvention,
    /// Severity for violations
    pub severity: Severity,
    /// Whether this rule applies only to directories
    pub directory_only: bool,
    /// Pattern for matching directory names (optional)
    pub pattern: Option<String>,
}

impl DirectoryRule {
    pub fn new(convention: CaseConvention) -> Self {
        Self {
            convention,
            severity: Severity::Medium,
            directory_only: true,
            pattern: None,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Check if this rule applies to a directory
    pub fn applies_to(&self, dir_name: &str, _is_directory: bool) -> bool {
        if let Some(ref pattern) = self.pattern {
            // Simple glob matching
            if !glob_matches(dir_name, pattern) {
                return false;
            }
        }
        true
    }

    /// Validate a directory name
    pub fn validate(&self, dir_name: &str) -> Option<String> {
        if !self.convention.validate(dir_name) {
            Some(format!(
                "Directory name '{}' does not follow {} convention (e.g., {})",
                dir_name,
                self.convention.name(),
                self.convention.example()
            ))
        } else {
            None
        }
    }
}

/// Constraint for directory validation
#[derive(Debug)]
pub struct DirectoryConstraint {
    name: String,
    config: DirectoryValidationConfig,
    rules: Vec<DirectoryRule>,
}

impl DirectoryConstraint {
    pub fn new() -> Self {
        Self {
            name: "directory".to_string(),
            config: DirectoryValidationConfig::new(),
            rules: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_config(mut self, config: DirectoryValidationConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_rule(mut self, rule: DirectoryRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_case_convention(mut self, convention: CaseConvention) -> Self {
        self.rules.push(DirectoryRule::new(convention));
        self
    }

    /// Validate a single directory
    fn validate_directory(&self, path: &Path) -> Option<ValidationFailure> {
        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if dir_name.is_empty() {
            return Some(ValidationFailure::new(
                &self.name,
                path,
                "Directory has no name",
            ));
        }

        // Check exclusions
        if self.config.is_excluded(dir_name) {
            return None;
        }

        // Apply all rules
        for rule in &self.rules {
            if rule.applies_to(dir_name, true) {
                if let Some(msg) = rule.validate(dir_name) {
                    return Some(ValidationFailure::new(&self.name, path, msg));
                }
            }
        }

        None
    }

    /// Recursively validate directories
    fn validate_recursive(&self, path: &Path, failures: &mut Vec<ValidationFailure>) {
        // Validate current directory
        if let Some(failure) = self.validate_directory(path) {
            failures.push(failure);
        }

        // Recurse into subdirectories
        if self.config.recursive {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let dir_name = entry_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        // Skip excluded directories
                        if !self.config.is_excluded(dir_name) {
                            self.validate_recursive(&entry_path, failures);
                        }
                    }
                }
            }
        }
    }
}

impl Default for DirectoryConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint for DirectoryConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates directory naming conventions"
    }

    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let mut failures = Vec::new();

        if path.is_dir() {
            if self.config.recursive && context.recursive_validation {
                self.validate_recursive(path, &mut failures);
            } else {
                if let Some(failure) = self.validate_directory(path) {
                    failures.push(failure);
                }
            }
        } else {
            // Not a directory - check if we should report this
            if self.config.distinguish_types {
                failures.push(ValidationFailure::new(
                    &self.name,
                    path,
                    "Path is not a directory",
                ));
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let passed = failures.is_empty();

        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_severity(self.config.severity)
            .with_duration(duration)
            .with_failures(failures.into()))
    }

    fn applies_to(&self, path: &Path) -> bool {
        // This constraint applies to directories
        path.is_dir()
    }

    fn default_severity(&self) -> Severity {
        self.config.severity
    }
}

fn glob_matches(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[1..];
        return name.ends_with(ext);
    }

    if pattern.contains('*') || pattern.contains('?') {
        match glob::Pattern::new(pattern) {
            Ok(p) => return p.matches(name),
            Err(_) => return false,
        }
    }

    name == pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_directory_validation_config() {
        let config = DirectoryValidationConfig::new();
        assert!(config.is_excluded(".git"));
        assert!(config.is_excluded("node_modules"));
        assert!(!config.is_excluded("src"));
    }

    #[test]
    fn test_directory_rule_validation() {
        let rule = DirectoryRule::new(CaseConvention::KebabCase);

        assert!(rule.validate("my-directory").is_none());
        assert!(rule.validate("my_directory").is_some());
        assert!(rule.validate("MyDirectory").is_some());
    }

    #[test]
    fn test_directory_constraint_kebab_case() {
        let constraint = DirectoryConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);

        let context = ConstraintContext::new();

        // Create a temp directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test-directory");
        std::fs::create_dir(&test_dir).unwrap();

        let result = constraint.validate(&test_dir, &context).unwrap();
        assert!(result.passed);

        // Invalid directory name
        let bad_dir = temp_dir.path().join("test_directory");
        std::fs::create_dir(&bad_dir).unwrap();

        let result = constraint.validate(&bad_dir, &context).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_directory_constraint_excludes_git() {
        let constraint = DirectoryConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);

        let context = ConstraintContext::new();

        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();

        // Should not report .git as invalid (it's excluded)
        let result = constraint.validate(&git_dir, &context).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_directory_constraint_distinguishes_types() {
        let constraint = DirectoryConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);

        let context = ConstraintContext::new();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test-file.txt");
        std::fs::write(&file_path, "content").unwrap();

        // Should fail because it's a file, not a directory
        let result = constraint.validate(&file_path, &context).unwrap();
        assert!(!result.passed);
    }
}
