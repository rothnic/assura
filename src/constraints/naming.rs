//! Naming convention constraint
//!
//! Provides validation for file and directory naming conventions
//! including case styles (kebab, camel, snake, etc.) and pattern-based rules.

use regex::Regex;
use std::path::Path;

use super::error::{ConstraintError, ConstraintResult, ValidationFailure, ValidationFailures};
use super::r#trait::{Constraint, ConstraintContext, ConstraintOutput};
use super::severity::Severity;

mod case;
pub use case::CaseConvention;

/// File extension rule
#[derive(Debug, Clone)]
pub struct ExtensionRule {
    /// Allowed extensions (without dots)
    pub extensions: Vec<String>,
    /// Whether extensions are required
    pub required: bool,
    /// Whether the extension must be lowercase
    pub lowercase_only: bool,
    /// Severity for violations
    pub severity: Severity,
}

impl ExtensionRule {
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
            required: true,
            lowercase_only: true,
            severity: Severity::Medium,
        }
    }

    pub fn allow_extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.push(ext.into());
        self
    }

    pub fn allow_extensions(mut self, exts: Vec<String>) -> Self {
        self.extensions = exts;
        self
    }

    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn allow_mixed_case(mut self) -> Self {
        self.lowercase_only = false;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Validate a filename against this rule
    pub fn validate(&self, filename: &str) -> Option<String> {
        if filename.starts_with('.') {
            // Hidden file, extension is optional
            if !self.required {
                return None;
            }
        }

        let parts: Vec<&str> = filename.rsplitn(2, '.').collect();

        if parts.len() < 2 {
            if self.required {
                return Some("File is missing an extension".to_string());
            }
            return None;
        }

        let ext = parts[0];

        // Check lowercase requirement
        if self.lowercase_only && ext.chars().any(|c| c.is_uppercase()) {
            return Some(format!("Extension '{}' should be lowercase", ext));
        }

        // Check allowed extensions
        if !self.extensions.is_empty() {
            let ext_lower = ext.to_lowercase();
            if !self
                .extensions
                .iter()
                .any(|e| e.to_lowercase() == ext_lower)
            {
                return Some(format!(
                    "Extension '{}' is not allowed (allowed: {:?})",
                    ext, self.extensions
                ));
            }
        }

        None
    }
}

impl Default for ExtensionRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern-based naming rule
#[derive(Debug, Clone)]
pub struct NamingPattern {
    /// Name of the pattern
    pub name: String,
    /// Regex pattern to match
    pub pattern: Regex,
    /// Whether this is a required or forbidden pattern
    pub required: bool,
    /// Severity for violations
    pub severity: Severity,
    /// Human-readable description
    pub description: String,
}

impl NamingPattern {
    pub fn new(name: impl Into<String>, pattern: &str) -> ConstraintResult<Self> {
        let name_str: String = name.into();
        let regex = Regex::new(pattern).map_err(|e| {
            ConstraintError::pattern(name_str.clone(), format!("Invalid regex: {}", e))
        })?;

        Ok(Self {
            name: name_str,
            pattern: regex,
            required: true,
            severity: Severity::Medium,
            description: String::new(),
        })
    }

    pub fn forbidden(name: impl Into<String>, pattern: &str) -> ConstraintResult<Self> {
        let mut pattern = Self::new(name, pattern)?;
        pattern.required = false;
        Ok(pattern)
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Validate a filename against this pattern
    pub fn validate(&self, filename: &str) -> Option<ValidationFailure> {
        let matches = self.pattern.is_match(filename);

        if self.required && !matches {
            return Some(ValidationFailure::new(
                &self.name,
                std::path::PathBuf::from(filename),
                format!(
                    "Filename does not match required pattern: {}",
                    self.description
                ),
            ));
        }

        if !self.required && matches {
            return Some(ValidationFailure::new(
                &self.name,
                std::path::PathBuf::from(filename),
                format!("Filename matches forbidden pattern: {}", self.description),
            ));
        }

        None
    }
}

/// A naming convention rule
#[derive(Debug, Clone)]
pub enum NamingRule {
    /// Case convention rule
    Case {
        convention: CaseConvention,
        severity: Severity,
    },
    /// Extension rule
    Extension(ExtensionRule),
    /// Pattern rule
    Pattern(NamingPattern),
    /// Forbidden pattern
    ForbiddenPattern(NamingPattern),
}

impl NamingRule {
    /// Validate a filename (without path)
    pub fn validate(&self, filename: &str, path: &Path) -> Option<ValidationFailure> {
        match self {
            NamingRule::Case {
                convention,
                severity: _,
            } => {
                // Strip extension for case validation (e.g., "my-file.txt" -> "my-file")
                let stem = filename.rsplit_once('.').map(|x| x.0).unwrap_or(filename);

                if !convention.validate(stem) {
                    return Some(ValidationFailure::new(
                        "naming_case",
                        path,
                        format!(
                            "Filename '{}' does not follow {} convention (e.g., {})",
                            filename,
                            convention.name(),
                            convention.example()
                        ),
                    ));
                }
                None
            }
            NamingRule::Extension(rule) => rule
                .validate(filename)
                .map(|msg| ValidationFailure::new("naming_extension", path, msg)),
            NamingRule::Pattern(pattern) => pattern.validate(filename),
            NamingRule::ForbiddenPattern(pattern) => pattern.validate(filename),
        }
    }

    /// Get the severity for this rule
    pub fn severity(&self) -> Severity {
        match self {
            NamingRule::Case { severity, .. } => *severity,
            NamingRule::Extension(rule) => rule.severity,
            NamingRule::Pattern(pattern) => pattern.severity,
            NamingRule::ForbiddenPattern(pattern) => pattern.severity,
        }
    }
}

/// Naming convention constraint
#[derive(Debug)]
pub struct NamingConstraint {
    name: String,
    /// File patterns to apply rules to
    file_patterns: Vec<String>,
    /// Naming rules to enforce
    rules: Vec<NamingRule>,
    default_severity: Severity,
}

impl NamingConstraint {
    pub fn new() -> Self {
        Self {
            name: "naming".to_string(),
            file_patterns: Vec::new(),
            rules: Vec::new(),
            default_severity: Severity::Medium,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_file_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.file_patterns.push(pattern.into());
        self
    }

    pub fn with_case_convention(mut self, convention: CaseConvention) -> Self {
        self.rules.push(NamingRule::Case {
            convention,
            severity: self.default_severity,
        });
        self
    }

    pub fn with_extension_rule(mut self, rule: ExtensionRule) -> Self {
        self.rules.push(NamingRule::Extension(rule));
        self
    }

    pub fn with_pattern(mut self, pattern: NamingPattern) -> Self {
        self.rules.push(NamingRule::Pattern(pattern));
        self
    }

    pub fn with_forbidden_pattern(mut self, pattern: NamingPattern) -> Self {
        self.rules.push(NamingRule::ForbiddenPattern(pattern));
        self
    }

    pub fn with_default_severity(mut self, severity: Severity) -> Self {
        self.default_severity = severity;
        self
    }

    /// Create a default configuration for Rust projects
    pub fn rust_config() -> Self {
        Self::new()
            .with_file_pattern("*.rs")
            .with_case_convention(CaseConvention::SnakeCase)
            .with_extension_rule(
                ExtensionRule::new()
                    .allow_extension("rs")
                    .with_severity(Severity::High),
            )
    }

    /// Create a default configuration for JavaScript/TypeScript projects
    pub fn javascript_config() -> Self {
        Self::new()
            .with_file_pattern("*.js")
            .with_file_pattern("*.ts")
            .with_file_pattern("*.jsx")
            .with_file_pattern("*.tsx")
            .with_case_convention(CaseConvention::KebabCase)
            .with_extension_rule(
                ExtensionRule::new()
                    .allow_extensions(vec![
                        "js".to_string(),
                        "ts".to_string(),
                        "jsx".to_string(),
                        "tsx".to_string(),
                    ])
                    .with_severity(Severity::High),
            )
    }

    /// Create a default configuration for general use
    pub fn general_config() -> Self {
        Self::new()
            .with_pattern(
                NamingPattern::forbidden("no_spaces", r".*\s.*")
                    .expect("Valid regex")
                    .with_description("Filenames cannot contain spaces")
                    .with_severity(Severity::High),
            )
            .with_pattern(
                NamingPattern::forbidden("no_special_chars", ".*[<>:\"|?*].*")
                    .expect("Valid regex")
                    .with_description("Filenames cannot contain special characters")
                    .with_severity(Severity::Critical),
            )
    }

    /// Check if this constraint applies to a path
    fn matches_pattern(&self, path: &Path) -> bool {
        if self.file_patterns.is_empty() {
            return true;
        }

        let path_str = path.to_string_lossy();
        for pattern in &self.file_patterns {
            if glob_matches(&path_str, pattern) {
                return true;
            }
        }

        false
    }
}

fn glob_matches(path: &str, pattern: &str) -> bool {
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

impl Default for NamingConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint for NamingConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates file and directory naming conventions"
    }

    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let mut failures = ValidationFailures::new();

        // Get the filename
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if filename.is_empty() {
            failures.add(ValidationFailure::new(
                &self.name,
                path,
                "Path has no filename",
            ));
        } else {
            // Apply all rules
            for rule in &self.rules {
                if let Some(failure) = rule.validate(filename, path) {
                    failures.add(failure);
                    if context.fail_fast {
                        break;
                    }
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let passed = failures.is_empty();

        // Determine severity
        let severity = if passed {
            self.default_severity
        } else {
            failures
                .failures()
                .iter()
                .map(|_| self.default_severity)
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
        self.matches_pattern(path)
    }

    fn default_severity(&self) -> Severity {
        self.default_severity
    }
}

#[cfg(test)]
mod naming_tests;
