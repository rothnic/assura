use super::types::{FactGeneration, FactId, FactOrigin};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Markdown-authored local link from one source document to a repository path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkdownLink {
    /// Stable fact ID.
    pub id: FactId,
    /// Generation that produced this fact.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Owning Markdown document fact ID.
    pub document_id: FactId,
    /// Repository-relative source path.
    pub source_path: PathBuf,
    /// One-based source line.
    pub source_line: usize,
    /// One-based source column.
    pub source_column: usize,
    /// Raw link target authored in Markdown.
    pub raw_target: String,
    /// Repository-relative target path.
    pub target_path: PathBuf,
    /// Optional target anchor without the leading `#`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_anchor: Option<String>,
    /// Optional GitHub line-anchor start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_line_start: Option<usize>,
    /// Optional GitHub line-anchor end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_line_end: Option<usize>,
    /// Whether the target path exists when fact ingestion ran.
    pub target_exists: bool,
    /// Related Markdown validation rule ID.
    pub rule: String,
}
