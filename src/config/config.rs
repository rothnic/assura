//! Unified configuration format
//!
//! Structure-first hierarchical configuration with:
//! - Hierarchical inheritance
//! - Bundled validation rules per directory
//! - Top-level file patterns
//! - Required file/directory existence checks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

pub use crate::config::inheritance::{ResolvedRule, RuleResolver};
pub use crate::config::loader::ConfigLoader;
pub use crate::config::ls_compat::LsLintCompatibility;

/// Root configuration struct
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Top-level file patterns for applying rules globally
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[validate(nested)]
    pub patterns: HashMap<String, FileBundle>,

    /// The structure hierarchy - each key is a directory path
    #[validate(nested)]
    pub structure: HashMap<String, DirectoryNode>,

    /// Optional LS-Lint compatibility layer (for testing only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ls: Option<LsLintCompatibility>,

    /// Paths to exclude from validation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// Regex for size string validation
pub static SIZE_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^\d+\s*(B|KB|MB|GB|TB)$").unwrap());

/// A node in the structure hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct DirectoryNode {
    /// File validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub files: Option<FileBundle>,

    /// Markdown validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub markdown: Option<MarkdownBundle>,

    /// Required files/directories validation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub exists: Option<ExistsValidation>,

    /// Child directories with their own rules
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub children: Option<HashMap<String, DirectoryNode>>,

    /// Whether to inherit rules from parent (default: true)
    #[serde(default = "default_true")]
    pub inherit: bool,
}

fn default_true() -> bool {
    true
}

/// Bundle of all file validations for a directory node
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct FileBundle {
    /// Naming convention (e.g., "snake_case", "kebab-case", "PascalCase")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_naming_convention"))]
    pub naming: Option<String>,

    /// Maximum lines per file
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 100000))]
    pub max_lines: Option<usize>,

    /// Maximum file size (e.g., "100KB", "1MB", "10MB")
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_size_string"))]
    pub max_size: Option<String>,

    /// Whether documentation is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_docs: Option<bool>,

    /// Allowed file extensions (e.g., ["rs", "md"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Severity level for violations in this node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,

    /// Allowed file names (for root directory policy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_names: Option<Vec<String>>,
}

/// Bundle of markdown validations for a directory node
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct MarkdownBundle {
    /// Whether frontmatter is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_frontmatter: Option<bool>,

    /// Required frontmatter fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_fields: Option<Vec<String>>,

    /// Maximum heading level depth
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(range(min = 1, max = 6))]
    pub max_heading_depth: Option<u8>,

    /// Whether to check for dead links
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_links: Option<bool>,

    /// Required sections in markdown files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_sections: Option<Vec<String>>,
}

/// Required files/directories existence validation
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct ExistsValidation {
    /// Required files that must exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>,

    /// Required directories that must exist
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directories: Option<Vec<String>>,
}

/// Validates that a naming convention string is valid
fn validate_naming_convention(conv: &str) -> Result<(), validator::ValidationError> {
    let valid_conventions = [
        "snake_case",
        "camelCase",
        "PascalCase",
        "kebab-case",
        "SCREAMING_SNAKE_CASE",
        "dot.case",
        "flatcase",
        "FLATCASE",
        "COBOL-CASE",
        "Train-Case",
        "lowercase",
        "UPPERCASE",
        "regex:", // Prefix for regex patterns
    ];

    // Check if it's a valid convention name or starts with "regex:"
    if valid_conventions.iter().any(|&c| conv == c || conv.starts_with(c))
        || conv.starts_with("regex:")
    {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_naming_convention");
        err.message = Some(
            format!(
                "'{}' is not a valid naming convention. Valid options: {:?}",
                conv, valid_conventions
            )
            .into(),
        );
        Err(err)
    }
}

/// Validates that a size string is valid (e.g., "100KB", "1MB", "10MB")
fn validate_size_string(size: &str) -> Result<(), validator::ValidationError> {
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

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            structure: HashMap::new(),
            ls: None,
            exclude: Vec::new(),
        }
    }

    /// Add a structure node at the given path
    pub fn with_node(mut self, path: impl Into<String>, node: DirectoryNode) -> Self {
        self.structure.insert(path.into(), node);
        self
    }

    /// Add an exclude pattern
    pub fn with_exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    /// Add a top-level pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>, bundle: FileBundle) -> Self {
        self.patterns.insert(pattern.into(), bundle);
        self
    }

    /// Get the effective bundle for a path by resolving inheritance
    pub fn resolve_for_path(&self, path: &std::path::Path) -> Option<ResolvedFileBundle> {
        let resolver = RuleResolver::new(self);
        resolver.resolve_for_path(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectoryNode {
    /// Create a new empty node
    pub fn new() -> Self {
        Self {
            files: None,
            markdown: None,
            exists: None,
            children: None,
            inherit: true,
        }
    }

    /// Set file validation bundle
    pub fn with_files(mut self, files: FileBundle) -> Self {
        self.files = Some(files);
        self
    }

    /// Set markdown validation bundle
    pub fn with_markdown(mut self, markdown: MarkdownBundle) -> Self {
        self.markdown = Some(markdown);
        self
    }

    /// Set exists validation
    pub fn with_exists(mut self, exists: ExistsValidation) -> Self {
        self.exists = Some(exists);
        self
    }

    /// Add a child node
    pub fn with_child(mut self, name: impl Into<String>, child: DirectoryNode) -> Self {
        self.children
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), child);
        self
    }

    /// Set inheritance behavior
    pub fn with_inherit(mut self, inherit: bool) -> Self {
        self.inherit = inherit;
        self
    }
}

impl Default for DirectoryNode {
    fn default() -> Self {
        Self::new()
    }
}

impl FileBundle {
    /// Create a new empty bundle
    pub fn new() -> Self {
        Self {
            naming: None,
            max_lines: None,
            max_size: None,
            require_docs: None,
            extensions: None,
            severity: None,
            allowed_names: None,
        }
    }

    /// Set naming convention
    pub fn with_naming(mut self, naming: impl Into<String>) -> Self {
        self.naming = Some(naming.into());
        self
    }

    /// Set maximum lines
    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = Some(max_lines);
        self
    }

    /// Set maximum size
    pub fn with_max_size(mut self, max_size: impl Into<String>) -> Self {
        self.max_size = Some(max_size.into());
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

    /// Set allowed file names
    pub fn with_allowed_names(mut self, allowed_names: Vec<String>) -> Self {
        self.allowed_names = Some(allowed_names);
        self
    }
}

impl Default for FileBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownBundle {
    /// Create a new empty bundle
    pub fn new() -> Self {
        Self {
            require_frontmatter: None,
            required_fields: None,
            max_heading_depth: None,
            check_links: None,
            required_sections: None,
        }
    }

    /// Set frontmatter requirement
    pub fn with_require_frontmatter(mut self, require: bool) -> Self {
        self.require_frontmatter = Some(require);
        self
    }

    /// Set required frontmatter fields
    pub fn with_required_fields(mut self, fields: Vec<String>) -> Self {
        self.required_fields = Some(fields);
        self
    }

    /// Set maximum heading depth
    pub fn with_max_heading_depth(mut self, depth: u8) -> Self {
        self.max_heading_depth = Some(depth);
        self
    }

    /// Set link checking
    pub fn with_check_links(mut self, check: bool) -> Self {
        self.check_links = Some(check);
        self
    }

    /// Set required sections
    pub fn with_required_sections(mut self, sections: Vec<String>) -> Self {
        self.required_sections = Some(sections);
        self
    }
}

impl Default for MarkdownBundle {
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

/// Resolved file bundle after inheritance is applied
#[derive(Debug, Clone)]
pub struct ResolvedFileBundle {
    /// The path pattern this bundle applies to
    pub path_pattern: String,
    /// The naming convention
    pub naming: Option<String>,
    /// Maximum lines
    pub max_lines: Option<usize>,
    /// Maximum size
    pub max_size: Option<String>,
    /// Documentation required
    pub require_docs: Option<bool>,
    /// Allowed extensions
    pub extensions: Option<Vec<String>>,
    /// Severity level
    pub severity: Option<String>,
    /// Allowed file names
    pub allowed_names: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!(config.structure.is_empty());
        assert!(config.exclude.is_empty());
        assert!(config.patterns.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_node(
                "src/",
                DirectoryNode::new().with_files(
                    FileBundle::new().with_naming("snake_case"),
                ),
            )
            .with_exclude("target/**")
            .with_pattern("**/*.rs", FileBundle::new().with_max_lines(500));

        assert!(config.structure.contains_key("src/"));
        assert_eq!(config.exclude.len(), 1);
        assert!(config.patterns.contains_key("**/*.rs"));
    }

    #[test]
    fn test_validate_naming_convention_valid() {
        assert!(validate_naming_convention("snake_case").is_ok());
        assert!(validate_naming_convention("PascalCase").is_ok());
        assert!(validate_naming_convention("kebab-case").is_ok());
        assert!(validate_naming_convention("regex:^[a-z]+$").is_ok());
    }

    #[test]
    fn test_validate_naming_convention_invalid() {
        assert!(validate_naming_convention("invalid_case").is_err());
        assert!(validate_naming_convention("UnknownCase").is_err());
    }

    #[test]
    fn test_validate_size_string_valid() {
        assert!(validate_size_string("100KB").is_ok());
        assert!(validate_size_string("1MB").is_ok());
        assert!(validate_size_string("10 MB").is_ok());
        assert!(validate_size_string("500B").is_ok());
    }

    #[test]
    fn test_validate_size_string_invalid() {
        assert!(validate_size_string("100").is_err());
        assert!(validate_size_string("large").is_err());
        assert!(validate_size_string("100XB").is_err());
    }

    #[test]
    fn test_directory_node_builder() {
        let node = DirectoryNode::new()
            .with_files(FileBundle::new().with_naming("kebab-case"))
            .with_child(
                "components/",
                DirectoryNode::new().with_files(
                    FileBundle::new().with_naming("PascalCase"),
                ),
            )
            .with_inherit(false);

        assert!(node.files.is_some());
        assert!(node.children.is_some());
        assert!(!node.inherit);
    }

    #[test]
    fn test_file_bundle_validation() {
        let bundle = FileBundle::new()
            .with_naming("snake_case")
            .with_max_lines(500)
            .with_max_size("1MB");

        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn test_file_bundle_invalid_naming() {
        let bundle = FileBundle::new().with_naming("invalid_case");
        assert!(bundle.validate().is_err());
    }

    #[test]
    fn test_yaml_serialization() {
        let config = Config::new().with_node(
            "src/",
            DirectoryNode::new().with_files(
                FileBundle::new()
                    .with_naming("snake_case")
                    .with_max_lines(500),
            ),
        );

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("structure:"));
        assert!(yaml.contains("src/"));
    }

    #[test]
    fn test_exists_validation() {
        let exists = ExistsValidation::new()
            .with_files(vec!["README.md".to_string(), "LICENSE".to_string()])
            .with_directories(vec!["src".to_string()]);

        assert_eq!(exists.files.as_ref().unwrap().len(), 2);
        assert_eq!(exists.directories.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_allowed_names() {
        let bundle = FileBundle::new()
            .with_allowed_names(vec!["README.md".to_string(), "LICENSE".to_string()]);

        assert!(bundle.allowed_names.is_some());
        assert_eq!(bundle.allowed_names.unwrap().len(), 2);
    }
}
