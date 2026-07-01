use super::types::{EdgeId, FactGeneration, FactId, FactOrigin};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Repository-internal reference edge between a source fact and a target path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryReferenceEdge {
    /// Stable edge ID.
    pub id: EdgeId,
    /// Generation that produced this edge.
    pub generation: FactGeneration,
    /// Source or derived marker.
    pub origin: FactOrigin,
    /// Source fact carrying the reference.
    pub source_id: FactId,
    /// Target resource fact ID when the path exists.
    pub target_id: Option<FactId>,
    /// Repository-relative source path.
    pub source_path: PathBuf,
    /// One-based source line when known.
    pub source_line: Option<usize>,
    /// One-based source column when known.
    pub source_column: Option<usize>,
    /// Repository-relative target path.
    pub target_path: PathBuf,
    /// Optional target anchor without the leading `#`.
    pub target_anchor: Option<String>,
    /// Optional target line start.
    pub target_line_start: Option<usize>,
    /// Optional target line end.
    pub target_line_end: Option<usize>,
    /// Whether the target path exists when ingestion ran.
    pub target_exists: bool,
    /// Source reference kind, such as `markdown_link`.
    pub reference_kind: String,
    /// Related validation rule ID.
    pub rule: String,
    /// Confidence level for conservative scanners.
    pub confidence: String,
}
