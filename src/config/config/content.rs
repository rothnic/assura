//! Repo-native content runtime configuration.

use serde::{Deserialize, Serialize};

/// Repo-native content runtime model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentModelConfig {
    /// Optional authoring source, such as a LinkML profile, used outside the
    /// runtime path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Checked-in JSON Schema-compatible runtime artifact loaded by Assura.
    pub validation_artifact: String,
}

/// One repo-native content collection declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentCollectionConfig {
    /// Runtime schema class name for records in this collection.
    #[serde(rename = "class")]
    pub class_name: String,
    /// Glob path relative to the project root.
    pub path: String,
    /// File adapter: `markdown_frontmatter`, `json_record`, `yaml_record`, or
    /// `jsonl_record`.
    pub adapter: String,
    /// Optional data source hint for adapter-specific docs and future writes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Optional body source hint for Markdown-backed collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Field used as the stable object id.
    pub id: String,
}

/// One cross-collection content relation declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContentRelationConfig {
    /// Target collection id for ordinary single-target references.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Explicit candidate target collections for references that may point at
    /// more than one collection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    /// Whether this relation stores multiple target IDs.
    #[serde(default)]
    pub many: bool,
    /// Whether the source field must be present and non-empty.
    #[serde(default)]
    pub required: bool,
    /// Whether this relation must not participate in a directed cycle.
    #[serde(default)]
    pub acyclic: bool,
}
