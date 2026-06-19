//! Experimental extension configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Experimental extension configuration for first-party custom constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtensionConfig {
    /// First-party custom constraints executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_constraints: Vec<CustomConstraintConfig>,

    /// Internal relationship constraints normalized from structure notation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipConstraintConfig>,
}

/// A first-party custom constraint declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CustomConstraintConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Constraint implementation name.
    #[serde(rename = "type")]
    pub kind: String,
    /// Source glob, relative to the project root.
    pub source: String,
    /// Target path template, relative to the project root.
    pub target: String,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// A capture-based relationship constraint normalized from structure notation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelationshipConstraintConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Source path pattern with named captures, relative to the project root.
    pub source: String,
    /// Structure entry that declared the source side of this relationship.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_declaration: Option<String>,
    /// Logical relationship name, such as `doc` or a generated counterpart id.
    pub need: String,
    /// Provider alternatives that can satisfy the need.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<RelationshipProviderConfig>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

/// One provider alternative for a relationship need.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RelationshipProviderConfig {
    /// Provider path template with named captures, relative to the project root.
    pub path: String,
    /// Optional Markdown heading text template inside the provider path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    /// Human-readable provider kind used in diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Structure entry that declared this provider alternative.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<String>,
}

/// Checked command-surface contract loaded by the `command_surface_docs`
/// custom constraint.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceContract {
    /// Supported command families and their documented flag/value surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<CommandSurfaceCommand>,
}

/// A command family in a checked command-surface contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceCommand {
    /// Canonical command name, such as `assura check`.
    pub name: String,
    /// Whether non-flag positional arguments are allowed.
    #[serde(default)]
    pub allow_positionals: bool,
    /// Supported flags keyed by their canonical spelling.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub flags: HashMap<String, CommandSurfaceFlag>,
}

/// A supported flag in a checked command-surface contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CommandSurfaceFlag {
    /// Whether this flag takes a value.
    #[serde(default)]
    pub takes_value: bool,
    /// Optional aliases such as short flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Optional allowlist of accepted values for value-taking flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    /// Required companion flag values keyed by canonical flag name.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub requires: HashMap<String, String>,
}

impl ExtensionConfig {
    /// Create an empty extension config.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom constraint declaration.
    pub fn with_custom_constraint(mut self, constraint: CustomConstraintConfig) -> Self {
        self.custom_constraints.push(constraint);
        self
    }

    /// Add an internal relationship constraint declaration.
    pub fn with_relationship(mut self, relationship: RelationshipConstraintConfig) -> Self {
        self.relationships.push(relationship);
        self
    }
}

impl CustomConstraintConfig {
    /// Create a paired-file custom constraint.
    pub fn paired_file_exists(
        id: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: "paired_file_exists".to_string(),
            source: source.into(),
            target: target.into(),
            severity: None,
        }
    }

    /// Set diagnostic severity.
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        self.severity = Some(severity.into());
        self
    }
}
