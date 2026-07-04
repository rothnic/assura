use serde::{Deserialize, Serialize};

/// A reusable repository-reference validation policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RepositoryReferenceConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Source file glob patterns to scan. Empty means all supported source
    /// reference file types.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,
    /// Markdown frontmatter fields that contain repository-relative references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontmatter_fields: Vec<String>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}
