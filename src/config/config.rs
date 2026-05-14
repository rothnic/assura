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

mod bundles;
mod validation;

pub use crate::config::inheritance::{ResolvedRule, RuleResolver};
pub use crate::config::loader::ConfigLoader;
pub use crate::config::ls_compat::LsLintCompatibility;
pub use bundles::{
    DirectoryBundle, ExistsValidation, FileBundle, MarkdownBundle, ResolvedFileBundle,
};
pub(crate) use validation::split_naming_conventions;
#[cfg(test)]
pub(super) use validation::{validate_naming_convention, validate_size_string};

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

/// A node in the structure hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "snake_case")]
pub struct DirectoryNode {
    /// File validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub files: Option<FileBundle>,

    /// Direct child directory validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub directories: Option<DirectoryBundle>,

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

    /// Whether this configured directory must exist (default: true).
    ///
    /// LS-Lint compatibility scopes can set this to false so a scoped rule
    /// applies when the directory exists without turning the scope into an
    /// existence requirement.
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
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
            directories: None,
            markdown: None,
            exists: None,
            children: None,
            inherit: true,
            required: true,
        }
    }

    /// Set file validation bundle
    pub fn with_files(mut self, files: FileBundle) -> Self {
        self.files = Some(files);
        self
    }

    /// Set direct child directory validation bundle
    pub fn with_directories(mut self, directories: DirectoryBundle) -> Self {
        self.directories = Some(directories);
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

    /// Set whether the configured directory itself must exist.
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl Default for DirectoryNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod config_tests;
