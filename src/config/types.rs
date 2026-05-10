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

mod child_limits;
mod message;
pub use child_limits::{ChildrenCountRange, ChildrenLimitConfig};
pub use message::Message;

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

#[cfg(test)]
mod types_tests;
