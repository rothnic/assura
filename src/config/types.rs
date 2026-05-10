//! Core configuration types for the unified policy-based config format
//!
//! This module defines the types for the new config format:
//! - `Rule`: Reusable validation rules defined in the `rules` section
//! - `Config`: Root configuration with rules, policy tree, and exclusions
//! - `PolicyNode`: Tree structure for path/extension-based policy application
//! - `PolicyEntry`: Individual policy entries (rule refs, conventions, directives)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

/// A reusable validation rule defined in the `rules` section
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Rule {
    /// File extensions this rule applies to (e.g., ["rs", "ts"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Naming convention(s) for files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naming: Option<NamingConvention>,

    /// Maximum lines per file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 100000))]
    pub max_lines: Option<usize>,

    /// Maximum file size (e.g., "100KB", "1MB")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_size_string"))]
    pub max_size: Option<String>,

    /// Whether documentation is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_docs: Option<bool>,

    /// Test file pattern (e.g., "{{name}}.test.tsx")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_test: Option<String>,

    /// Custom messages for violations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,

    /// Extends another rule by name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

/// Naming convention specification - single or multiple cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", untagged)]
pub enum NamingConvention {
    /// A single naming convention
    Single(Case),
    /// Multiple allowed naming conventions
    Multiple(Vec<Case>),
}

/// Individual case convention
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Case {
    /// snake_case
    #[serde(rename = "snake_case")]
    SnakeCase,
    /// camelCase
    #[serde(rename = "camelCase")]
    CamelCase,
    /// PascalCase
    #[serde(rename = "PascalCase")]
    PascalCase,
    /// kebab-case
    #[serde(rename = "kebab-case")]
    KebabCase,
    /// SCREAMING_SNAKE_CASE
    #[serde(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    /// dot.case
    #[serde(rename = "dot.case")]
    DotCase,
    /// flatcase (all lowercase, no separators)
    #[serde(rename = "flatcase")]
    Flatcase,
    /// FLATCASE (all uppercase, no separators)
    #[serde(rename = "FLATCASE")]
    FlatcaseUpper,
    /// COBOL-CASE (uppercase with hyphens)
    #[serde(rename = "COBOL-CASE")]
    CobolCase,
    /// Train-Case (Capitalized words with hyphens)
    #[serde(rename = "Train-Case")]
    TrainCase,
    /// lowercase (all lowercase, spaces allowed)
    #[serde(rename = "lowercase")]
    Lowercase,
    /// UPPERCASE (all uppercase, spaces allowed)
    #[serde(rename = "UPPERCASE")]
    Uppercase,
    /// Regex pattern (custom validation)
    #[serde(rename = "regex:")]
    Regex(String),
}

/// Custom message configuration for violations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    /// Violation description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violation: Option<String>,

    /// Explanation of why this rule exists
    #[serde(skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,

    /// How to fix the violation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,

    /// How to override this rule
    #[serde(rename = "override", skip_serializing_if = "Option::is_none")]
    pub override_: Option<String>,

    /// Link to documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// Root configuration struct
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Reusable rule definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[validate(nested)]
    pub rules: HashMap<String, Rule>,

    /// Policy tree - hierarchical policy application
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub policy: Option<PolicyNode>,

    /// Legacy structure format (for backwards compatibility)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub structure: HashMap<String, crate::config::config::DirectoryNode>,

    /// Paths to exclude from validation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// A node in the policy tree
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct PolicyNode {
    /// Entries in this policy node (path/extension → policy entry)
    #[serde(flatten)]
    pub entries: HashMap<String, PolicyEntry>,
}

/// Individual policy entry - can be a rule reference, convention, directive, inline rule, or nested node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum PolicyEntry {
    /// Reference to a rule by name (e.g., "@rust-source")
    RuleRef(String),
    /// Direct naming convention specification
    Convention(NamingConvention),
    /// Directive (apply, require, exists, message, severity)
    Directive(Directive),
    /// Inline rule definition (properties like naming, max_lines, etc.)
    InlineRule(InlineRule),
    /// Nested policy node for subdirectories
    Nested(PolicyNode),
}

/// Inline rule definition for use directly in policy tree
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct InlineRule {
    /// File extensions this rule applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Naming convention(s) for files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naming: Option<NamingConvention>,

    /// Maximum lines per file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 100000))]
    pub max_lines: Option<usize>,

    /// Maximum file size (e.g., "100KB", "1MB")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_size_string"))]
    pub max_size: Option<String>,

    /// Whether documentation is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_docs: Option<bool>,

    /// Test file pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_test: Option<String>,

    /// Custom messages for violations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,

    /// Severity level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
}

/// Directives for policy nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "directive", content = "value")]
pub enum Directive {
    /// Apply one or more rules with optional overrides
    Apply(Vec<ApplyEntry>),
    /// Require specific files or directories to exist
    Require(RequireConfig),
    /// Alias for require (exists: [file1, file2])
    Exists(Vec<String>),
    /// Limit the number of direct children (files and/or directories)
    /// Used to encourage nesting when directories get too large
    LimitChildren(ChildrenLimitConfig),
    /// Set custom messages
    Message(Message),
    /// Set severity level
    Severity(Severity),
}

/// Entry in an apply directive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ApplyEntry {
    /// Name of the rule to apply (with @ prefix)
    pub rule: String,

    /// Optional property overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Rule>,
}

/// Configuration for require directive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RequireConfig {
    /// Required files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// Required directories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirs: Option<Vec<String>>,

    /// Custom message for missing items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Severity for missing items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
}

/// Configuration for limiting the number of direct children in a directory
///
/// This encourages better organization by preventing overly flat directory structures.
/// When a directory reaches the maximum allowed children, developers should create
/// subdirectories to organize files into logical groups.
///
/// # Examples
///
/// ```yaml
/// # Allow max 10 direct children (files + dirs) in utils/
/// utils/:
///   limit_children:
///     max: 10
///     message: "Too many files in utils/. Organize into subdirectories by category."
///
/// # Allow between 2-5 files and 0-3 subdirectories in components/
/// components/:
///   limit_children:
///     files:
///       min: 2
///       max: 5
///     dirs:
///       max: 3
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChildrenLimitConfig {
    /// Maximum total children (files + directories) allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<usize>,

    /// Minimum total children required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<usize>,

    /// Limits specifically for files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<ChildrenCountRange>,

    /// Limits specifically for directories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirs: Option<ChildrenCountRange>,

    /// Whether to count hidden files/directories (starting with .)
    #[serde(default = "default_true")]
    pub include_hidden: bool,

    /// Custom message when limit is exceeded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Severity for violations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<Severity>,
}

/// Range configuration for counting files or directories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ChildrenCountRange {
    /// Minimum count required (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<usize>,

    /// Maximum count allowed (inclusive)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<usize>,
}

/// Severity levels for violations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Critical - must fix, blocks CI
    Critical,
    /// High - should fix soon
    High,
    /// Medium - fix when convenient
    Medium,
    /// Low - informational
    Low,
    /// Off - disable this rule
    Off,
}

/// Validates that a size string is valid (e.g., "100KB", "1MB", "10MB")
fn validate_size_string(size: &str) -> Result<(), validator::ValidationError> {
    use regex::Regex;
    lazy_static::lazy_static! {
        static ref SIZE_REGEX: Regex = Regex::new(r"^\d+\s*(B|KB|MB|GB|TB)$").unwrap();
    }

    if SIZE_REGEX.is_match(size) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_size_string");
        err.message = Some(
            format!(
                "'{}' is not a valid size string. Expected format: '<number><unit>' where unit is B, KB, MB, GB, or TB",
                size
            )
            .into(),
        );
        Err(err)
    }
}

impl Rule {
    /// Create a new empty rule
    pub fn new() -> Self {
        Self {
            extensions: None,
            naming: None,
            max_lines: None,
            max_size: None,
            require_docs: None,
            require_test: None,
            message: None,
            extends: None,
        }
    }

    /// Set extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set naming convention
    pub fn with_naming(mut self, naming: NamingConvention) -> Self {
        self.naming = Some(naming);
        self
    }

    /// Set max lines
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Set max size
    pub fn with_max_size(mut self, max_size: impl Into<String>) -> Self {
        self.max_size = Some(max_size.into());
        self
    }

    /// Set require docs
    pub fn with_require_docs(mut self, require_docs: bool) -> Self {
        self.require_docs = Some(require_docs);
        self
    }

    /// Set require test pattern
    pub fn with_require_test(mut self, pattern: impl Into<String>) -> Self {
        self.require_test = Some(pattern.into());
        self
    }

    /// Set message
    pub fn with_message(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }

    /// Set extends
    pub fn with_extends(mut self, extends: impl Into<String>) -> Self {
        self.extends = Some(extends.into());
        self
    }
}

impl Default for Rule {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            policy: None,
            structure: HashMap::new(),
            exclude: Vec::new(),
        }
    }

    /// Add a rule
    pub fn with_rule(mut self, name: impl Into<String>, rule: Rule) -> Self {
        self.rules.insert(name.into(), rule);
        self
    }

    /// Set the policy node
    pub fn with_policy(mut self, policy: PolicyNode) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Add an exclude pattern
    pub fn with_exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyNode {
    /// Create a new empty policy node
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Add an entry
    pub fn with_entry(mut self, key: impl Into<String>, entry: PolicyEntry) -> Self {
        self.entries.insert(key.into(), entry);
        self
    }
}

impl Default for PolicyNode {
    fn default() -> Self {
        Self::new()
    }
}

impl Message {
    /// Create a new empty message
    pub fn new() -> Self {
        Self::default()
    }

    /// Set violation message
    pub fn with_violation(mut self, msg: impl Into<String>) -> Self {
        self.violation = Some(msg.into());
        self
    }

    /// Set why message
    pub fn with_why(mut self, msg: impl Into<String>) -> Self {
        self.why = Some(msg.into());
        self
    }

    /// Set fix message
    pub fn with_fix(mut self, msg: impl Into<String>) -> Self {
        self.fix = Some(msg.into());
        self
    }

    /// Set override message
    pub fn with_override(mut self, msg: impl Into<String>) -> Self {
        self.override_ = Some(msg.into());
        self
    }

    /// Set docs link
    pub fn with_docs(mut self, docs: impl Into<String>) -> Self {
        self.docs = Some(docs.into());
        self
    }
}

impl ApplyEntry {
    /// Create a new apply entry
    pub fn new(rule: impl Into<String>) -> Self {
        Self {
            rule: rule.into(),
            overrides: None,
        }
    }

    /// Set overrides
    pub fn with_overrides(mut self, overrides: Rule) -> Self {
        self.overrides = Some(overrides);
        self
    }
}

impl RequireConfig {
    /// Create a new empty require config
    pub fn new() -> Self {
        Self {
            files: None,
            dirs: None,
            message: None,
            severity: None,
        }
    }

    /// Set required files
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = Some(files);
        self
    }

    /// Set required directories
    pub fn with_dirs(mut self, dirs: Vec<String>) -> Self {
        self.dirs = Some(dirs);
        self
    }

    /// Set message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }
}

impl Default for RequireConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildrenLimitConfig {
    /// Create a new children limit config
    pub fn new() -> Self {
        Self {
            max: None,
            min: None,
            files: None,
            dirs: None,
            include_hidden: true,
            message: None,
            severity: None,
        }
    }

    /// Set maximum total children
    pub fn with_max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }

    /// Set minimum total children
    pub fn with_min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set file count limits
    pub fn with_files(mut self, files: ChildrenCountRange) -> Self {
        self.files = Some(files);
        self
    }

    /// Set directory count limits
    pub fn with_dirs(mut self, dirs: ChildrenCountRange) -> Self {
        self.dirs = Some(dirs);
        self
    }

    /// Set whether to include hidden files
    pub fn with_include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Set custom message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }
}

impl Default for ChildrenLimitConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildrenCountRange {
    /// Create a new count range
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

    /// Set minimum count
    pub fn with_min(mut self, min: usize) -> Self {
        self.min = Some(min);
        self
    }

    /// Set maximum count
    pub fn with_max(mut self, max: usize) -> Self {
        self.max = Some(max);
        self
    }
}

impl Default for ChildrenCountRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Default value for boolean fields
fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_builder() {
        let rule = Rule::new()
            .with_extensions(vec!["rs".to_string()])
            .with_naming(NamingConvention::Single(Case::SnakeCase))
            .with_max_lines(500);

        assert_eq!(rule.extensions.unwrap(), vec!["rs"]);
        assert!(rule.max_lines.is_some());
    }

    #[test]
    fn test_naming_convention_single() {
        let yaml = "snake_case";
        let conv: NamingConvention = serde_yaml::from_str(yaml).unwrap();
        match conv {
            NamingConvention::Single(Case::SnakeCase) => {}
            _ => panic!("Expected single snake_case"),
        }
    }

    #[test]
    fn test_naming_convention_multiple() {
        let yaml = "[snake_case, camelCase]";
        let conv: NamingConvention = serde_yaml::from_str(yaml).unwrap();
        match conv {
            NamingConvention::Multiple(cases) => {
                assert_eq!(cases.len(), 2);
            }
            _ => panic!("Expected multiple cases"),
        }
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_rule("rust", Rule::new().with_max_lines(500))
            .with_exclude("target/**");

        assert!(config.rules.contains_key("rust"));
        assert_eq!(config.exclude.len(), 1);
    }

    #[test]
    fn test_yaml_serialization() {
        let config = Config::new().with_rule(
            "rust-source",
            Rule::new()
                .with_extensions(vec!["rs".to_string()])
                .with_naming(NamingConvention::Single(Case::SnakeCase))
                .with_max_lines(500),
        );

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("rules:"));
        assert!(yaml.contains("rust-source:"));
    }

    #[test]
    fn test_case_deserialization() {
        let cases = vec![
            ("snake_case", Case::SnakeCase),
            ("camelCase", Case::CamelCase),
            ("PascalCase", Case::PascalCase),
            ("kebab-case", Case::KebabCase),
            ("SCREAMING_SNAKE_CASE", Case::ScreamingSnakeCase),
        ];

        for (yaml, expected) in cases {
            let parsed: Case = serde_yaml::from_str(yaml).unwrap();
            assert!(
                std::mem::discriminant(&parsed) == std::mem::discriminant(&expected),
                "Failed for {}",
                yaml
            );
        }
    }
}
