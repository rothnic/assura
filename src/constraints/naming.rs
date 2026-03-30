//! Naming convention constraint
//!
//! Provides validation for file and directory naming conventions
//! including case styles (kebab, camel, snake, etc.) and pattern-based rules.

use regex::Regex;
use std::path::Path;

use super::error::{ConstraintError, ConstraintResult, ValidationFailure, ValidationFailures};
use super::r#trait::{Constraint, ConstraintContext, ConstraintOutput};
use super::severity::Severity;

/// Case conventions for naming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaseConvention {
    /// lowercase (e.g., filename)
    LowerCase,
    /// UPPERCASE (e.g., FILENAME)
    UpperCase,
    /// snake_case (e.g., file_name)
    SnakeCase,
    /// camelCase (e.g., fileName)
    CamelCase,
    /// PascalCase (e.g., FileName)
    PascalCase,
    /// kebab-case (e.g., file-name)
    KebabCase,
    /// SCREAMING_SNAKE_CASE (e.g., FILE_NAME)
    ScreamingSnakeCase,
    /// dot.case (e.g., file.name)
    DotCase,
    /// flatcase (e.g., filename) - lowercase, no separators
    FlatCase,
    /// FLATCASE (e.g., FILENAME) - UPPERCASE, no separators
    ScreamingFlatCase,
    /// COBOL-CASE (e.g., FILE-NAME) - UPPERCASE with hyphens
    CobolCase,
    /// Train-Case (e.g., File-Name) - Title-Case with hyphens
    TrainCase,
}

impl CaseConvention {
    /// Get the name of this convention
    pub fn name(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "lowercase",
            CaseConvention::UpperCase => "UPPERCASE",
            CaseConvention::SnakeCase => "snake_case",
            CaseConvention::CamelCase => "camelCase",
            CaseConvention::PascalCase => "PascalCase",
            CaseConvention::KebabCase => "kebab-case",
            CaseConvention::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
            CaseConvention::DotCase => "dot.case",
            CaseConvention::FlatCase => "flatcase",
            CaseConvention::ScreamingFlatCase => "FLATCASE",
            CaseConvention::CobolCase => "COBOL-CASE",
            CaseConvention::TrainCase => "Train-Case",
        }
    }

    /// Validate a string against this case convention
    pub fn validate(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        match self {
            CaseConvention::LowerCase => {
                name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::UpperCase => {
                name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::SnakeCase => {
                // Must be lowercase with underscores, no consecutive underscores
                if name.starts_with('_') || name.ends_with('_') || name.contains("__") {
                    return false;
                }
                name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::CamelCase => {
                // Must start with lowercase, can have uppercase in middle
                if !name.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                    return false;
                }
                // No underscores, no consecutive uppercase
                let mut prev_upper = false;
                for c in name.chars() {
                    if c == '_' || c == '-' {
                        return false;
                    }
                    if c.is_uppercase() {
                        if prev_upper {
                            return false;
                        }
                        prev_upper = true;
                    } else {
                        prev_upper = false;
                    }
                }
                true
            }
            CaseConvention::PascalCase => {
                // Must start with uppercase, rest follows camelCase rules
                if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    return false;
                }
                // No underscores, no consecutive uppercase
                let mut prev_upper = false;
                for c in name.chars() {
                    if c == '_' || c == '-' {
                        return false;
                    }
                    if c.is_uppercase() {
                        if prev_upper {
                            return false;
                        }
                        prev_upper = true;
                    } else {
                        prev_upper = false;
                    }
                }
                true
            }
            CaseConvention::KebabCase => {
                // Must be lowercase with hyphens, no consecutive hyphens
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
            }
            CaseConvention::ScreamingSnakeCase => {
                // Must be uppercase with underscores, no consecutive underscores
                if name.starts_with('_') || name.ends_with('_') || name.contains("__") {
                    return false;
                }
                name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '_')
            }
            CaseConvention::DotCase => {
                // Must be lowercase with dots, no consecutive dots
                if name.starts_with('.') || name.ends_with('.') || name.contains("..") {
                    return false;
                }
                name.chars().all(|c| c.is_lowercase() || c.is_numeric() || c == '.')
            }
            CaseConvention::FlatCase => {
                // Must be all lowercase letters and numbers, no separators
                if name.is_empty() {
                    return false;
                }
                name.chars().all(|c| c.is_lowercase() || c.is_numeric())
            }
            CaseConvention::ScreamingFlatCase => {
                // Must be all uppercase letters and numbers, no separators
                if name.is_empty() {
                    return false;
                }
                name.chars().all(|c| c.is_uppercase() || c.is_numeric())
            }
            CaseConvention::CobolCase => {
                // Must be uppercase with hyphens, no consecutive hyphens
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                name.chars().all(|c| c.is_uppercase() || c.is_numeric() || c == '-')
            }
            CaseConvention::TrainCase => {
                // Must start with uppercase, then alternating lowercase and uppercase with hyphens
                // Pattern: Word-Word-Word (each word starts with uppercase, rest lowercase)
                if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
                    return false;
                }
                // Check first character is uppercase
                if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    return false;
                }
                // Check pattern: uppercase followed by lowercase, then hyphen, repeat
                let parts: Vec<&str> = name.split('-').collect();
                for part in parts {
                    if part.is_empty() {
                        return false;
                    }
                    // Check if part is all numeric (allowed in Train-Case)
                    if part.chars().all(|c| c.is_numeric()) {
                        continue;
                    }
                    // Each part must start with uppercase
                    if !part.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        return false;
                    }
                    // Rest must be lowercase or numeric
                    for c in part.chars().skip(1) {
                        if !c.is_lowercase() && !c.is_numeric() {
                            return false;
                        }
                    }
                }
                true
            }
        }
    }

    /// Get an example for this convention
    pub fn example(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "filename",
            CaseConvention::UpperCase => "FILENAME",
            CaseConvention::SnakeCase => "file_name",
            CaseConvention::CamelCase => "fileName",
            CaseConvention::PascalCase => "FileName",
            CaseConvention::KebabCase => "file-name",
            CaseConvention::ScreamingSnakeCase => "FILE_NAME",
            CaseConvention::DotCase => "file.name",
            CaseConvention::FlatCase => "filename",
            CaseConvention::ScreamingFlatCase => "FILENAME",
            CaseConvention::CobolCase => "FILE-NAME",
            CaseConvention::TrainCase => "File-Name",
        }
    }

    /// Get a description of what's valid
    pub fn description(self) -> &'static str {
        match self {
            CaseConvention::LowerCase => "all lowercase letters and numbers",
            CaseConvention::UpperCase => "all uppercase letters and numbers",
            CaseConvention::SnakeCase => "lowercase with underscores between words",
            CaseConvention::CamelCase => "starts with lowercase, capitalizes word boundaries",
            CaseConvention::PascalCase => "starts with uppercase, capitalizes word boundaries",
            CaseConvention::KebabCase => "lowercase with hyphens between words",
            CaseConvention::ScreamingSnakeCase => "uppercase with underscores between words",
            CaseConvention::DotCase => "lowercase with dots between words",
            CaseConvention::FlatCase => "all lowercase letters and numbers, no separators",
            CaseConvention::ScreamingFlatCase => "all uppercase letters and numbers, no separators",
            CaseConvention::CobolCase => "uppercase with hyphens between words",
            CaseConvention::TrainCase => "title case words separated by hyphens",
        }
    }
}

impl Default for CaseConvention {
    fn default() -> Self {
        CaseConvention::SnakeCase
    }
}

impl std::fmt::Display for CaseConvention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

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
            if !self.extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
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
                format!("Filename does not match required pattern: {}", self.description),
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
            NamingRule::Case { convention, severity: _ } => {
                // Strip extension for case validation (e.g., "my-file.txt" -> "my-file")
                let stem = filename
                    .rsplitn(2, '.')
                    .nth(1)
                    .unwrap_or(filename);
                
                if !convention.validate(stem) {
                    return Some(ValidationFailure::new(
                        "naming_case",
                        path,
                        format!("Filename '{}' does not follow {} convention (e.g., {})",
                            filename, convention.name(), convention.example()),
                    ));
                }
                None
            }
            NamingRule::Extension(rule) => {
                rule.validate(filename).map(|msg| {
                    ValidationFailure::new("naming_extension", path, msg)
                })
            }
            NamingRule::Pattern(pattern) => {
                pattern.validate(filename)
            }
            NamingRule::ForbiddenPattern(pattern) => {
                pattern.validate(filename)
            }
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
                    .allow_extensions(vec!["js".to_string(), "ts".to_string(), "jsx".to_string(), "tsx".to_string()])
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
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

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
mod tests {
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
        let result = constraint.validate(Path::new("/test/my-file.txt"), &context).unwrap();
        assert!(result.passed);

        // Invalid case
        let result = constraint.validate(Path::new("/test/my_file.txt"), &context).unwrap();
        assert!(!result.passed);

        // Invalid extension
        let result = constraint.validate(Path::new("/test/my-file.doc"), &context).unwrap();
        assert!(!result.passed);
    }

    #[test]
    fn test_rust_config() {
        let constraint = NamingConstraint::rust_config();
        let context = ConstraintContext::new();

        let result = constraint.validate(Path::new("/test/my_module.rs"), &context).unwrap();
        assert!(result.passed);

        let result = constraint.validate(Path::new("/test/my-module.rs"), &context).unwrap();
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
        let constraint = NamingConstraint::new()
            .with_case_convention(CaseConvention::PascalCase);
        let context = ConstraintContext::new();

        // Multi-part extensions should still validate the file stem
        let result = constraint.validate(Path::new("MyComponent.d.ts"), &context).unwrap();
        assert!(result.passed);

        let result = constraint.validate(Path::new("myComponent.d.ts"), &context).unwrap();
        assert!(!result.passed); // Should fail - not PascalCase

        // Snake case with simple extension
        let constraint = NamingConstraint::new()
            .with_case_convention(CaseConvention::SnakeCase);
        
        let result = constraint.validate(Path::new("my_component.ts"), &context).unwrap();
        assert!(result.passed);

        let result = constraint.validate(Path::new("myComponent.ts"), &context).unwrap();
        assert!(!result.passed);

        // Kebab case with simple extension
        let constraint = NamingConstraint::new()
            .with_case_convention(CaseConvention::KebabCase);
        
        let result = constraint.validate(Path::new("my-component.js"), &context).unwrap();
        assert!(result.passed);

        // Camel case with simple extension
        let constraint = NamingConstraint::new()
            .with_case_convention(CaseConvention::CamelCase);
        
        let result = constraint.validate(Path::new("myComponent.tsx"), &context).unwrap();
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
}