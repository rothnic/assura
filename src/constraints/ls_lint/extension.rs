//! Complex extension handling for LS-Lint parity
//!
//! Provides support for multi-part extensions like .d.ts, .test.js, .min.css
//! and extension-specific naming rules.

use regex::Regex;
use std::collections::HashMap;

use crate::constraints::error::ConstraintResult;
use crate::constraints::naming::CaseConvention;
use crate::constraints::severity::Severity;

/// Represents a multi-part extension pattern
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtensionPattern {
    /// The full extension pattern (e.g., "d.ts", "test.js")
    pattern: String,
    /// The parts of the extension (e.g., ["d", "ts"])
    parts: Vec<String>,
}

impl ExtensionPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        let parts: Vec<String> = pattern.split('.').map(|s| s.to_string()).collect();
        Self { pattern, parts }
    }

    /// Parse an extension from a filename
    pub fn from_filename(filename: &str) -> Option<Self> {
        if filename.starts_with('.') {
            // Hidden file - handle specially
            let without_leading_dot = &filename[1..];
            let parts: Vec<&str> = without_leading_dot.split('.').collect();
            if parts.len() >= 2 {
                return Some(Self {
                    pattern: without_leading_dot.to_string(),
                    parts: parts.iter().map(|s| s.to_string()).collect(),
                });
            }
            return None;
        }

        let parts: Vec<&str> = filename.split('.').collect();
        if parts.len() >= 2 {
            // Get extension parts (everything after the first dot in the name)
            let ext_parts = &parts[1..];
            Some(Self {
                pattern: ext_parts.join("."),
                parts: ext_parts.iter().map(|s| s.to_string()).collect(),
            })
        } else {
            None
        }
    }

    /// Check if this pattern matches another pattern (supports wildcards)
    pub fn matches(&self, other: &ExtensionPattern) -> bool {
        if self.parts.len() != other.parts.len() {
            return false;
        }

        for (a, b) in self.parts.iter().zip(&other.parts) {
            if a != "*" && a != b {
                return false;
            }
        }

        true
    }

    /// Check if this pattern matches a string pattern
    pub fn matches_str(&self, pattern: &str) -> bool {
        let other = ExtensionPattern::new(pattern);
        self.matches(&other)
    }

    pub fn as_str(&self) -> &str {
        &self.pattern
    }

    pub fn parts(&self) -> &[String] {
        &self.parts
    }
}

impl std::fmt::Display for ExtensionPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pattern)
    }
}

/// Rule for validating multi-part extensions
#[derive(Debug, Clone)]
pub struct MultiPartExtensionRule {
    /// Allowed extension patterns
    allowed: Vec<ExtensionPattern>,
    /// Required extension patterns
    required: Vec<ExtensionPattern>,
    /// Whether extensions are required
    extensions_required: bool,
    /// Severity for violations
    severity: Severity,
    /// Naming convention for specific extensions
    naming_conventions: HashMap<String, CaseConvention>,
    /// Whether to allow any extension (when allowed is empty)
    allow_any: bool,
}

impl MultiPartExtensionRule {
    pub fn new() -> Self {
        Self {
            allowed: Vec::new(),
            required: Vec::new(),
            extensions_required: true,
            severity: Severity::Medium,
            naming_conventions: HashMap::new(),
            allow_any: true,
        }
    }

    pub fn allow_extension(mut self, pattern: impl Into<String>) -> Self {
        self.allowed.push(ExtensionPattern::new(pattern));
        self.allow_any = false;
        self
    }

    pub fn allow_extensions(mut self, patterns: Vec<String>) -> Self {
        for pattern in patterns {
            self.allowed.push(ExtensionPattern::new(pattern));
        }
        self.allow_any = false;
        self
    }

    pub fn require_extension(mut self, pattern: impl Into<String>) -> Self {
        self.required.push(ExtensionPattern::new(pattern));
        self
    }

    pub fn optional(mut self) -> Self {
        self.extensions_required = false;
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set a naming convention for a specific extension pattern
    pub fn with_naming_convention(
        mut self,
        extension: impl Into<String>,
        convention: CaseConvention,
    ) -> Self {
        self.naming_conventions.insert(extension.into(), convention);
        self
    }

    /// Validate a filename against this rule
    pub fn validate(&self, filename: &str) -> Option<String> {
        // Extract the extension
        let extension = ExtensionPattern::from_filename(filename);

        if extension.is_none() {
            if self.extensions_required {
                return Some("File is missing an extension".to_string());
            }
            return None;
        }

        let ext = extension.unwrap();

        // Check required extensions
        for required in &self.required {
            if !ext.matches(required) {
                return Some(format!(
                    "Extension '{}' does not match required pattern '{}'",
                    ext, required
                ));
            }
        }

        // Check allowed extensions
        if !self.allow_any && !self.allowed.is_empty() {
            let matches_allowed = self.allowed.iter().any(|a| ext.matches(a));
            if !matches_allowed {
                let allowed_str: Vec<_> = self.allowed.iter().map(|a| a.as_str()).collect();
                return Some(format!(
                    "Extension '{}' is not allowed (allowed: {:?})",
                    ext, allowed_str
                ));
            }
        }

        // Check naming convention for this extension
        for (ext_pattern, convention) in &self.naming_conventions {
            if ext.matches_str(ext_pattern) {
                // Extract the stem (filename without extension)
                let stem = extract_stem(filename, &ext);
                if !convention.validate(&stem) {
                    return Some(format!(
                        "Filename stem '{}' does not follow {} convention for extension '{}'",
                        stem,
                        convention.name(),
                        ext
                    ));
                }
            }
        }

        None
    }

    /// Get the severity for this rule
    pub fn severity(&self) -> Severity {
        self.severity
    }
}

impl Default for MultiPartExtensionRule {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract the stem from a filename given its extension pattern
fn extract_stem(filename: &str, ext: &ExtensionPattern) -> String {
    let ext_len = ext.as_str().len();
    if filename.len() > ext_len + 1 {
        // +1 for the dot
        filename[..filename.len() - ext_len - 1].to_string()
    } else {
        filename.to_string()
    }
}

/// Complex extension with support for multiple patterns
#[derive(Debug, Clone)]
pub struct ComplexExtension {
    patterns: Vec<ExtensionPattern>,
    base_extension: Option<String>,
}

impl ComplexExtension {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            base_extension: None,
        }
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(ExtensionPattern::new(pattern));
        self
    }

    pub fn with_base_extension(mut self, ext: impl Into<String>) -> Self {
        self.base_extension = Some(ext.into());
        self
    }

    /// Check if a filename matches any of the patterns
    pub fn matches(&self, filename: &str) -> bool {
        let ext = match ExtensionPattern::from_filename(filename) {
            Some(e) => e,
            None => return false,
        };

        self.patterns.iter().any(|p| ext.matches(p))
    }

    /// Get the extension from a filename
    pub fn extract(filename: &str) -> Option<ExtensionPattern> {
        ExtensionPattern::from_filename(filename)
    }

    /// Check if extension is a test file pattern (e.g., .test.js, .spec.ts)
    pub fn is_test_file(filename: &str) -> bool {
        if let Some(ext) = ExtensionPattern::from_filename(filename) {
            let ext_str = ext.as_str().to_lowercase();
            ext_str.contains("test") || ext_str.contains("spec")
        } else {
            false
        }
    }

    /// Check if extension is a minified file pattern (e.g., .min.js, .min.css)
    pub fn is_minified_file(filename: &str) -> bool {
        if let Some(ext) = ExtensionPattern::from_filename(filename) {
            ext.as_str().to_lowercase().contains("min")
        } else {
            false
        }
    }

    /// Check if extension is a declaration file pattern (e.g., .d.ts)
    pub fn is_declaration_file(filename: &str) -> bool {
        if let Some(ext) = ExtensionPattern::from_filename(filename) {
            ext.as_str() == "d.ts" || ext.as_str().ends_with(".d.ts")
        } else {
            false
        }
    }
}

impl Default for ComplexExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_pattern_parsing() {
        let pattern = ExtensionPattern::new("d.ts");
        assert_eq!(pattern.parts(), &["d", "ts"]);
        assert_eq!(pattern.as_str(), "d.ts");

        let pattern = ExtensionPattern::new("test.js");
        assert_eq!(pattern.parts(), &["test", "js"]);
    }

    #[test]
    fn test_extension_pattern_from_filename() {
        let ext = ExtensionPattern::from_filename("file.d.ts").unwrap();
        assert_eq!(ext.as_str(), "d.ts");

        let ext = ExtensionPattern::from_filename("component.test.js").unwrap();
        assert_eq!(ext.as_str(), "test.js");

        let ext = ExtensionPattern::from_filename("no_extension");
        assert!(ext.is_none());
    }

    #[test]
    fn test_extension_pattern_matching() {
        let pattern1 = ExtensionPattern::new("d.ts");
        let pattern2 = ExtensionPattern::new("d.ts");
        assert!(pattern1.matches(&pattern2));

        let pattern3 = ExtensionPattern::new("*.ts");
        assert!(pattern3.matches(&pattern1));

        let pattern4 = ExtensionPattern::new("test.js");
        assert!(!pattern1.matches(&pattern4));
    }

    #[test]
    fn test_multi_part_extension_rule() {
        let rule = MultiPartExtensionRule::new()
            .allow_extension("d.ts")
            .allow_extension("test.js")
            .allow_extension("min.css");

        assert!(rule.validate("types.d.ts").is_none());
        assert!(rule.validate("component.test.js").is_none());
        assert!(rule.validate("styles.min.css").is_none());
        assert!(rule.validate("file.txt").is_some());
    }

    #[test]
    fn test_multi_part_extension_with_naming_convention() {
        let rule = MultiPartExtensionRule::new()
            .allow_extension("test.js")
            .with_naming_convention("test.js", CaseConvention::KebabCase);

        assert!(rule.validate("my-component.test.js").is_none());
        assert!(rule.validate("my_component.test.js").is_some());
    }

    #[test]
    fn test_complex_extension_detection() {
        assert!(ComplexExtension::is_test_file("component.test.js"));
        assert!(ComplexExtension::is_test_file("component.spec.ts"));
        assert!(!ComplexExtension::is_test_file("component.js"));

        assert!(ComplexExtension::is_minified_file("bundle.min.js"));
        assert!(ComplexExtension::is_minified_file("styles.min.css"));
        assert!(!ComplexExtension::is_minified_file("bundle.js"));

        assert!(ComplexExtension::is_declaration_file("types.d.ts"));
        assert!(!ComplexExtension::is_declaration_file("script.ts"));
    }

    #[test]
    fn test_extension_pattern_with_hidden_files() {
        let ext = ExtensionPattern::from_filename(".gitignore");
        assert!(ext.is_none()); // No compound extension

        let ext = ExtensionPattern::from_filename(".eslintrc.js").unwrap();
        assert_eq!(ext.as_str(), "eslintrc.js");
    }
}
