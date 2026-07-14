//! Validation bundles used by the structure-first configuration model.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "full-cli")]
use validator::Validate;

#[cfg(feature = "full-cli")]
use super::validation::{validate_naming_convention, validate_size_string};

mod markdown;
pub(crate) use markdown::{merge_markdown_rule_configs, MarkdownOutlineView};
pub use markdown::{
    MarkdownBundle, MarkdownOutlineEntry, MarkdownOutlineNode, MarkdownRuleConfig,
    MarkdownlintCandidateConfig,
};

/// Bundle of all file validations for a directory node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(rename_all = "snake_case")]
pub struct FileBundle {
    /// Naming convention (e.g., "snake_case", "kebab-case", "PascalCase")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "full-cli",
        validate(custom(function = "validate_naming_convention"))
    )]
    pub naming: Option<String>,

    /// Naming conventions keyed by direct file glob pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naming_patterns: Option<HashMap<String, String>>,

    /// Maximum lines per file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(range(min = 1, max = 100000)))]
    pub max_lines: Option<usize>,

    /// Maximum lines keyed by direct file glob pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines_patterns: Option<HashMap<String, usize>>,

    /// Maximum file size (e.g., "100KB", "1MB", "10MB")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "full-cli",
        validate(custom(function = "validate_size_string"))
    )]
    pub max_size: Option<String>,

    /// Maximum file size keyed by direct file glob pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_patterns: Option<HashMap<String, String>>,

    /// Whether documentation is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_docs: Option<bool>,

    /// Allowed file extensions (e.g., ["rs", "md"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Severity level for violations in this node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    /// Required files in this directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Allowed file names (for root directory policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_names: Option<Vec<String>>,

    /// Allowed direct file glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_patterns: Option<Vec<String>>,

    /// Forbidden direct file glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_patterns: Option<Vec<String>>,

    /// Whether direct files not explicitly allowed are accepted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_extra: Option<bool>,

    /// Direct file count constraints keyed by glob pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<HashMap<String, String>>,
}

/// Bundle of direct child directory validations for a directory node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(rename_all = "snake_case")]
pub struct DirectoryBundle {
    /// Naming convention for direct child directories
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        feature = "full-cli",
        validate(custom(function = "validate_naming_convention"))
    )]
    pub naming: Option<String>,

    /// Required direct child directories
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    /// Allowed direct child directory names
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_names: Option<Vec<String>>,

    /// Allowed direct child directory glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_patterns: Option<Vec<String>>,

    /// Forbidden direct child directory glob patterns
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_patterns: Option<Vec<String>>,

    /// Whether direct child directories not explicitly allowed are accepted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_extra: Option<bool>,

    /// Severity level for violations in this node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    /// Direct child directory count constraints keyed by glob pattern
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exists: Option<HashMap<String, String>>,
}

/// Required files/directories existence validation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(rename_all = "snake_case")]
pub struct ExistsValidation {
    /// Required files that must exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// Required directories that must exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
}

/// Resolved file bundle after inheritance is applied
#[derive(Debug, Clone)]
pub struct ResolvedFileBundle {
    /// The path pattern this bundle applies to
    pub path_pattern: String,
    /// The naming convention
    pub naming: Option<String>,
    /// Naming conventions keyed by direct file glob pattern
    pub naming_patterns: Option<HashMap<String, String>>,
    /// Maximum lines
    pub max_lines: Option<usize>,
    /// Maximum lines keyed by direct file glob pattern
    pub max_lines_patterns: Option<HashMap<String, usize>>,
    /// Maximum size
    pub max_size: Option<String>,
    /// Maximum size keyed by direct file glob pattern
    pub max_size_patterns: Option<HashMap<String, String>>,
    /// Documentation required
    pub require_docs: Option<bool>,
    /// Allowed extensions
    pub extensions: Option<Vec<String>>,
    /// Severity level
    pub severity: Option<String>,
    /// Required file names
    pub required: Option<Vec<String>>,
    /// Allowed file names
    pub allowed_names: Option<Vec<String>>,
    /// Allowed direct file glob patterns
    pub allowed_patterns: Option<Vec<String>>,
    /// Forbidden direct file glob patterns
    pub forbidden_patterns: Option<Vec<String>>,
    /// Whether unexpected direct files are allowed
    pub allow_extra: Option<bool>,
    /// Direct file count constraints
    pub exists: Option<HashMap<String, String>>,
}

impl FileBundle {
    /// Create a new empty bundle
    pub fn new() -> Self {
        Self {
            naming: None,
            naming_patterns: None,
            max_lines: None,
            max_lines_patterns: None,
            max_size: None,
            max_size_patterns: None,
            require_docs: None,
            extensions: None,
            severity: None,
            required: None,
            allowed_names: None,
            allowed_patterns: None,
            forbidden_patterns: None,
            allow_extra: None,
            exists: None,
        }
    }

    /// Set naming convention
    pub fn with_naming(mut self, naming: impl Into<String>) -> Self {
        self.naming = Some(naming.into());
        self
    }

    /// Set naming conventions by direct file glob pattern
    pub fn with_naming_patterns(mut self, naming_patterns: HashMap<String, String>) -> Self {
        self.naming_patterns = Some(naming_patterns);
        self
    }

    /// Set maximum lines
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Set maximum lines by direct file glob pattern
    pub fn with_max_lines_patterns(mut self, max_lines_patterns: HashMap<String, usize>) -> Self {
        self.max_lines_patterns = Some(max_lines_patterns);
        self
    }

    /// Set maximum size
    pub fn with_max_size(mut self, max_size: impl Into<String>) -> Self {
        self.max_size = Some(max_size.into());
        self
    }

    /// Set maximum sizes by direct file glob pattern
    pub fn with_max_size_patterns(mut self, max_size_patterns: HashMap<String, String>) -> Self {
        self.max_size_patterns = Some(max_size_patterns);
        self
    }

    /// Set documentation requirement
    pub fn with_require_docs(mut self, require_docs: bool) -> Self {
        self.require_docs = Some(require_docs);
        self
    }

    /// Set allowed extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Set severity level
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = Some(severity.into());
        self
    }

    /// Set required file names
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Set allowed file names
    pub fn with_allowed_names(mut self, allowed_names: Vec<String>) -> Self {
        self.allowed_names = Some(allowed_names);
        self
    }

    /// Set allowed direct file glob patterns
    pub fn with_allowed_patterns(mut self, allowed_patterns: Vec<String>) -> Self {
        self.allowed_patterns = Some(allowed_patterns);
        self
    }

    /// Set forbidden direct file glob patterns
    pub fn with_forbidden_patterns(mut self, forbidden_patterns: Vec<String>) -> Self {
        self.forbidden_patterns = Some(forbidden_patterns);
        self
    }

    /// Set whether unexpected direct files are allowed
    pub fn with_allow_extra(mut self, allow_extra: bool) -> Self {
        self.allow_extra = Some(allow_extra);
        self
    }

    /// Set direct file count constraints
    pub fn with_exists(mut self, exists: HashMap<String, String>) -> Self {
        self.exists = Some(exists);
        self
    }
}

impl Default for FileBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryBundle {
    /// Create a new empty directory bundle
    pub fn new() -> Self {
        Self {
            naming: None,
            required: None,
            allowed_names: None,
            allowed_patterns: None,
            forbidden_patterns: None,
            allow_extra: None,
            severity: None,
            exists: None,
        }
    }

    /// Set naming convention
    pub fn with_naming(mut self, naming: impl Into<String>) -> Self {
        self.naming = Some(naming.into());
        self
    }

    /// Set required direct child directory names
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Set allowed direct child directory names
    pub fn with_allowed_names(mut self, allowed_names: Vec<String>) -> Self {
        self.allowed_names = Some(allowed_names);
        self
    }

    /// Set allowed direct child directory glob patterns
    pub fn with_allowed_patterns(mut self, allowed_patterns: Vec<String>) -> Self {
        self.allowed_patterns = Some(allowed_patterns);
        self
    }

    /// Set forbidden direct child directory glob patterns
    pub fn with_forbidden_patterns(mut self, forbidden_patterns: Vec<String>) -> Self {
        self.forbidden_patterns = Some(forbidden_patterns);
        self
    }

    /// Set whether unexpected direct child directories are allowed
    pub fn with_allow_extra(mut self, allow_extra: bool) -> Self {
        self.allow_extra = Some(allow_extra);
        self
    }

    /// Set severity level
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = Some(severity.into());
        self
    }

    /// Set direct child directory count constraints
    pub fn with_exists(mut self, exists: HashMap<String, String>) -> Self {
        self.exists = Some(exists);
        self
    }
}

impl Default for DirectoryBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl ExistsValidation {
    /// Create a new empty exists validation
    pub fn new() -> Self {
        Self {
            files: None,
            directories: None,
        }
    }

    /// Set required files
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.files = Some(files);
        self
    }

    /// Set required directories
    pub fn with_directories(mut self, directories: Vec<String>) -> Self {
        self.directories = Some(directories);
        self
    }
}

impl Default for ExistsValidation {
    fn default() -> Self {
        Self::new()
    }
}
