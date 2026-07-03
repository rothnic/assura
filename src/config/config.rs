//! Unified configuration format
//!
//! Structure-first hierarchical configuration with:
//! - Hierarchical inheritance
//! - Bundled validation rules per directory
//! - Top-level file patterns
//! - Required file/directory existence checks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "full-cli")]
use validator::Validate;

mod bundles;
mod content;
mod extensions;
mod quality;
#[cfg(feature = "yaml-config")]
mod structure_notation;
mod validation;

pub use crate::config::inheritance::{ResolvedRule, RuleResolver};
#[cfg(feature = "yaml-config")]
pub use crate::config::loader::ConfigLoader;
pub use crate::config::ls_compat::LsLintCompatibility;
pub(crate) use bundles::{merge_markdown_rule_configs, MarkdownOutlineView};
pub use bundles::{
    DirectoryBundle, ExistsValidation, FileBundle, MarkdownBundle, MarkdownOutlineEntry,
    MarkdownOutlineNode, MarkdownRuleConfig, MarkdownlintCandidateConfig, ResolvedFileBundle,
};
pub use content::{
    ContentCodeSymbolConfig, ContentCollectionConfig, ContentModelConfig, ContentRelationConfig,
};
pub use extensions::{
    AgentGuidanceConfig, CommandSurfaceCommand, CommandSurfaceContract, CommandSurfaceFlag,
    CustomConstraintConfig, DocsLifecycleClaimPatternConfig, DocsLifecycleConfig, ExtensionConfig,
    ManifestSemanticsConfig, ManifestSemanticsManifestConfig, ModuleTopologyConfig,
    ModuleTopologyModuleConfig, RelationshipConstraintConfig, RelationshipProviderConfig,
    ReleaseArtifactConfig, ReleaseContractConfig, RepositoryReferenceConfig,
    RequirementsTraceabilityConfig, SupportMatrixConfig, SupportMatrixDocsClaimSourceConfig,
    SupportMatrixEntryConfig, TestRelationshipConfig, TestRelationshipFixtureFamilyConfig,
    TestRelationshipIgnoredTestConfig, TestRelationshipSourceConfig,
};
pub use quality::{QualityConfig, QualityScopeConfig};
#[cfg(feature = "yaml-config")]
pub(crate) use structure_notation::normalize_structure_config_value;
pub(crate) use validation::split_naming_conventions;
#[cfg(feature = "yaml-config")]
pub(crate) use validation::validate_config_semantics;
#[cfg(test)]
pub(super) use validation::{validate_naming_convention, validate_size_string};

/// Root configuration struct
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// Top-level file patterns for applying rules globally
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub patterns: HashMap<String, FileBundle>,

    /// The structure hierarchy - each key is a directory path
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub structure: HashMap<String, DirectoryNode>,

    /// Optional LS-Lint compatibility layer (for testing only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ls: Option<LsLintCompatibility>,

    /// Experimental first-party extension configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ExtensionConfig>,

    /// High-level quality gate policy for changed-file planning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityConfig>,

    /// Optional repo-native content runtime model artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<ContentModelConfig>,

    /// Optional repo-native content collections keyed by local collection id.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub collections: HashMap<String, ContentCollectionConfig>,

    /// Optional repo-native content relations keyed as `collection.field`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub relations: HashMap<String, ContentRelationConfig>,

    /// Optional code-symbol reference fields keyed as `collection.field`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub code_symbols: HashMap<String, ContentCodeSymbolConfig>,

    /// Paths to exclude from validation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
}

/// A node in the structure hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct DirectoryNode {
    /// File validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub files: Option<FileBundle>,

    /// Direct child directory validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub directories: Option<DirectoryBundle>,

    /// Validation rules for the directory represented by this node.
    ///
    /// This is primarily used by the LS-Lint compatibility layer for `.dir`
    /// rules, which apply to the indexed directory itself rather than to its
    /// children.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub self_directory: Option<DirectoryBundle>,

    /// Markdown validation rules for this node
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub markdown: Option<MarkdownBundle>,

    /// Required files/directories validation
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub exists: Option<ExistsValidation>,

    /// Child directories with their own rules
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
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
            extensions: None,
            quality: None,
            models: None,
            collections: HashMap::new(),
            relations: HashMap::new(),
            code_symbols: HashMap::new(),
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

    /// Add experimental extension configuration.
    pub fn with_extensions(mut self, extensions: ExtensionConfig) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Add high-level quality gate policy.
    pub fn with_quality(mut self, quality: QualityConfig) -> Self {
        self.quality = Some(quality);
        self
    }

    /// Add a repo-native content runtime model artifact.
    pub fn with_models(mut self, models: ContentModelConfig) -> Self {
        self.models = Some(models);
        self
    }

    /// Add a repo-native content collection.
    pub fn with_collection(
        mut self,
        name: impl Into<String>,
        collection: ContentCollectionConfig,
    ) -> Self {
        self.collections.insert(name.into(), collection);
        self
    }

    /// Add a repo-native content relation.
    pub fn with_relation(
        mut self,
        key: impl Into<String>,
        relation: ContentRelationConfig,
    ) -> Self {
        self.relations.insert(key.into(), relation);
        self
    }

    /// Add a repo-native content code-symbol reference field.
    pub fn with_code_symbol(
        mut self,
        key: impl Into<String>,
        symbol: ContentCodeSymbolConfig,
    ) -> Self {
        self.code_symbols.insert(key.into(), symbol);
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
            self_directory: None,
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

    /// Set validation bundle for this directory itself.
    pub fn with_self_directory(mut self, directory: DirectoryBundle) -> Self {
        self.self_directory = Some(directory);
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
