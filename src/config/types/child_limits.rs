//! Children-count policy configuration types.

use serde::{Deserialize, Serialize};

use super::Severity;

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
