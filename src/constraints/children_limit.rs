//! Children limit constraint for validating directory organization
//!
//! This constraint enforces limits on the number of direct children
//! in a directory to encourage better organization and prevent
//! overly flat directory structures.

use std::collections::HashMap;
use std::path::Path;

use crate::config::types::{ChildrenLimitConfig, ChildrenCountRange, Severity};
use crate::constraints::error::{ValidationFailure, ValidationFailures};
use crate::constraints::r#trait::{Constraint, ConstraintContext, ConstraintOutput};

/// Children limit constraint
#[derive(Debug)]
pub struct ChildrenLimitConstraint {
    name: String,
    /// Map of path patterns to their limit configurations
    limits: HashMap<String, ChildrenLimitConfig>,
    default_severity: Severity,
}

/// Result of counting children in a directory
#[derive(Debug, Clone)]
struct ChildrenCount {
    files: usize,
    dirs: usize,
    total: usize,
}

impl ChildrenLimitConstraint {
    pub fn new() -> Self {
        Self {
            name: "children_limit".to_string(),
            limits: HashMap::new(),
            default_severity: Severity::Medium,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn add_limit(mut self, path_pattern: impl Into<String>, config: ChildrenLimitConfig) -> Self {
        self.limits.insert(path_pattern.into(), config);
        self
    }

    pub fn with_default_severity(mut self, severity: Severity) -> Self {
        self.default_severity = severity;
        self
    }

    /// Check if a path matches a pattern
    fn matches_pattern(path: &Path, pattern: &str) -> bool {
        let path_str = path.to_string_lossy();

        // Handle glob patterns
        if pattern.contains('*') || pattern.contains('{') || pattern.contains('[') {
            match glob::Pattern::new(pattern) {
                Ok(p) => return p.matches(&path_str),
                Err(_) => return false,
            }
        }

        // Handle directory patterns (ending with /)
        if pattern.ends_with('/') {
            let prefix = &pattern[..pattern.len() - 1];
            return path_str.starts_with(prefix)
                && (path_str.len() == prefix.len() || path_str[prefix.len()..].starts_with('/'));
        }

        // Exact match
        path_str == pattern
    }

    /// Count children in a directory
    fn count_children(path: &Path, config: &ChildrenLimitConfig) -> std::io::Result<ChildrenCount> {
        let mut files = 0;
        let mut dirs = 0;

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Skip hidden files if not included
            if !config.include_hidden && name_str.starts_with('.') {
                continue;
            }

            if entry.file_type()?.is_dir() {
                dirs += 1;
            } else {
                files += 1;
            }
        }

        Ok(ChildrenCount {
            files,
            dirs,
            total: files + dirs,
        })
    }

    /// Validate a directory against a limit config
    fn validate_directory(
        &self,
        path: &Path,
        config: &ChildrenLimitConfig,
        severity: Severity,
    ) -> Vec<ValidationFailure> {
        let mut failures = Vec::new();

        let count = match Self::count_children(path, config) {
            Ok(c) => c,
            Err(_) => return failures, // Can't read directory, skip
        };

        // Check total children limits
        if let Some(max) = config.max {
            if count.total > max {
                let message = config.message.clone().unwrap_or_else(|| {
                    format!(
                        "Directory has {} direct children (max: {}). Consider organizing files into subdirectories.",
                        count.total, max
                    )
                });
                failures.push(
                    ValidationFailure::new(&self.name, path, message)
                        .with_suggestion("Create subdirectories to group related files"),
                );
            }
        }

        if let Some(min) = config.min {
            if count.total < min {
                let message = format!(
                    "Directory has only {} direct children (min: {})",
                    count.total, min
                );
                failures.push(ValidationFailure::new(&self.name, path, message));
            }
        }

        // Check file-specific limits
        if let Some(ref file_config) = config.files {
            if let Some(max) = file_config.max {
                if count.files > max {
                    let message = format!(
                        "Directory has {} files (max: {}). Consider organizing into subdirectories.",
                        count.files, max
                    );
                    failures.push(
                        ValidationFailure::new(&self.name, path, message)
                            .with_suggestion("Group related files into subdirectories"),
                    );
                }
            }

            if let Some(min) = file_config.min {
                if count.files < min {
                    let message = format!(
                        "Directory has only {} files (min: {})",
                        count.files, min
                    );
                    failures.push(ValidationFailure::new(&self.name, path, message));
                }
            }
        }

        // Check directory-specific limits
        if let Some(ref dir_config) = config.dirs {
            if let Some(max) = dir_config.max {
                if count.dirs > max {
                    let message = format!(
                        "Directory has {} subdirectories (max: {})",
                        count.dirs, max
                    );
                    failures.push(ValidationFailure::new(&self.name, path, message));
                }
            }

            if let Some(min) = dir_config.min {
                if count.dirs < min {
                    let message = format!(
                        "Directory has only {} subdirectories (min: {})",
                        count.dirs, min
                    );
                    failures.push(ValidationFailure::new(
&self.name, path, message));
                }
            }
        }

        failures
    }
}

impl Default for ChildrenLimitConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint for ChildrenLimitConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates that directories don't have too many direct children"
    }

    fn validate(
        &self,
        path: &Path,
        _context: &ConstraintContext,
    ) -> crate::constraints::error::ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let mut failures = ValidationFailures::new();

        // Only validate directories
        if !path.is_dir() {
            return Ok(ConstraintOutput::new(&self.name, path, true
            )
            .with_duration(start.elapsed().as_millis() as u64));
        }

        // Find matching limit configurations
        for (pattern, config) in &self.limits {
            if Self::matches_pattern(path, pattern) {
                let sev = config.severity.unwrap_or(self.default_severity);
                let dir_failures = self.validate_directory(path, config, sev);
                for failure in dir_failures {
                    failures.add(failure);
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let passed = failures.is_empty();

        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_duration(duration)
            .with_failures(failures))
    }

    fn applies_to(&self, path: &Path,
    ) -> bool {
        // Applies to all directories
        path.is_dir()
    }

    fn default_severity(&self) -> crate::constraints::severity::Severity {
        crate::constraints::severity::Severity::Medium
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_matches_pattern() {
        assert!(ChildrenLimitConstraint::matches_pattern(
            Path::new("src/utils"),
            "src/"
        ));
        assert!(ChildrenLimitConstraint::matches_pattern(
            Path::new("src/utils/helpers"),
            "src/"
        ));
        assert!(!ChildrenLimitConstraint::matches_pattern(
            Path::new("tests/utils"),
            "src/"
        ));
        assert!(ChildrenLimitConstraint::matches_pattern(
            Path::new("src/components/Button"),
            "src/**"
        ));
    }

    #[test]
    fn test_count_children() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        std::fs::create_dir(&dir_path).unwrap();

        // Create 3 files and 2 directories
        std::fs::File::create(dir_path.join("file1.txt")).unwrap();
        std::fs::File::create(dir_path.join("file2.txt")).unwrap();
        std::fs::File::create(dir_path.join("file3.txt")).unwrap();
        std::fs::create_dir(dir_path.join("subdir1")).unwrap();
        std::fs::create_dir(dir_path.join("subdir2")).unwrap();

        let config = ChildrenLimitConfig::new();
        let count = ChildrenLimitConstraint::count_children(&dir_path, &config).unwrap();

        assert_eq!(count.files, 3);
        assert_eq!(count.dirs, 2);
        assert_eq!(count.total, 5);
    }

    #[test]
    fn test_validate_directory_max_children() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("utils");
        std::fs::create_dir(&dir_path).unwrap();

        // Create 5 files
        for i in 0..5 {
            std::fs::File::create(dir_path.join(format!("file{}.txt", i))).unwrap();
        }

        let constraint = ChildrenLimitConstraint::new().add_limit(
            "utils",
            ChildrenLimitConfig::new()
                .with_max(3)
                .with_message("Too many files in utils folder".to_string()),
        );

        let failures = constraint.validate_directory(
            &dir_path,
            &ChildrenLimitConfig::new().with_max(3),
            Severity::Medium,
        );

        assert_eq!(failures.len(), 1);
        assert!(failures[0].message.contains("5 direct children"));
    }

    #[test]
    fn test_validate_directory_file_limits() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("components");
        std::fs::create_dir(&dir_path).unwrap();

        // Create 8 files
        for i in 0..8 {
            std::fs::File::create(dir_path.join(format!("Component{}.tsx", i))).unwrap();
        }

        let config = ChildrenLimitConfig::new()
            .with_files(ChildrenCountRange::new().with_max(5));

        let failures = ChildrenLimitConstraint::new().validate_directory(
            &dir_path,
            &config,
            Severity::Medium,
        );

        assert_eq!(failures.len(), 1);
        assert!(failures[0].message.contains("8 files"));
    }

    #[test]
    fn test_hidden_files_exclusion() {
        let temp_dir = tempfile::tempdir().unwrap();
        let dir_path = temp_dir.path().join("test");
        std::fs::create_dir(&dir_path).unwrap();

        // Create visible and hidden files
        std::fs::File::create(dir_path.join("visible.txt")).unwrap();
        std::fs::File::create(dir_path.join(".hidden")).unwrap();

        // With include_hidden = true (default)
        let config1 = ChildrenLimitConfig::new();
        let count1 = ChildrenLimitConstraint::count_children(&dir_path, &config1).unwrap();
        assert_eq!(count1.files, 2);

        // With include_hidden = false
        let config2 = ChildrenLimitConfig::new().with_include_hidden(false);
        let count2 = ChildrenLimitConstraint::count_children(&dir_path, &config2).unwrap();
        assert_eq!(count2.files, 1);
    }
}
