//! Experimental extension configuration.

use serde::{Deserialize, Serialize};

/// Experimental extension configuration for first-party custom constraints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtensionConfig {
    /// First-party custom constraints executed by `assura check`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_constraints: Vec<CustomConstraintConfig>,
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
