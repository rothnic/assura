//! Serializable content query output types.

use super::context::ContentQueryError;
use crate::cli::OutputFormat;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub(super) struct AgentQueryOutput {
    pub(super) schema: &'static str,
    pub(super) request: AgentQueryRequestOutput,
    pub(super) response: Value,
}

#[derive(Debug, Serialize)]
pub(super) struct AgentQueryRequestOutput {
    pub(super) capability: &'static str,
    pub(super) cli: &'static str,
    #[serde(serialize_with = "serialize_path")]
    pub(super) project_root: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    pub(super) config_path: PathBuf,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextPackOutput {
    pub(super) schema: &'static str,
    pub(super) request: ContextPackRequestOutput,
    pub(super) bounds: ContextPackBoundsOutput,
    pub(super) diagnostics: Vec<DiagnosticOutput>,
    pub(super) instance: Option<InstanceOutput>,
    pub(super) related: Option<ExpandOutput>,
    pub(super) search: Option<SearchOutput>,
    pub(super) missing_relations: Vec<RelationOutput>,
    pub(super) safe_fixes: Vec<SafeFixOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextPackRequestOutput {
    pub(super) mode: &'static str,
    pub(super) cli: &'static str,
    #[serde(serialize_with = "serialize_path")]
    pub(super) project_root: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    pub(super) config_path: PathBuf,
    pub(super) collection: Option<String>,
    pub(super) id: Option<String>,
    pub(super) text: Option<String>,
    pub(super) limit: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextPackBoundsOutput {
    pub(super) limit: usize,
    pub(super) truncated: Vec<ContextPackTruncationOutput>,
    pub(super) omissions: Vec<ContextPackOmissionOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextPackTruncationOutput {
    pub(super) field: &'static str,
    pub(super) original_count: usize,
    pub(super) returned_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextPackOmissionOutput {
    pub(super) field: &'static str,
    pub(super) reason: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct CollectionsOutput {
    #[serde(serialize_with = "serialize_path")]
    pub(super) project_root: PathBuf,
    #[serde(serialize_with = "serialize_path")]
    pub(super) config_path: PathBuf,
    pub(super) collections: Vec<CollectionOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct CollectionOutput {
    pub(super) collection: String,
    pub(super) object_type: String,
    pub(super) adapter: String,
    pub(super) path_pattern: Option<String>,
    pub(super) instances: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct InstancesOutput {
    pub(super) collection: String,
    pub(super) instances: Vec<InstanceSummary>,
}

#[derive(Debug, Serialize)]
pub(super) struct InstanceSummary {
    pub(super) id: String,
    pub(super) object_type: String,
    #[serde(serialize_with = "serialize_path")]
    pub(super) path: PathBuf,
    pub(super) title: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct InstanceOutput {
    pub(super) id: String,
    pub(super) collection: String,
    pub(super) object_type: String,
    #[serde(serialize_with = "serialize_path")]
    pub(super) path: PathBuf,
    pub(super) data: Map<String, Value>,
    pub(super) outgoing_relations: Vec<RelationOutput>,
    pub(super) incoming_relations: Vec<RelationOutput>,
    pub(super) diagnostics: Vec<DiagnosticOutput>,
    pub(super) sections: Vec<SectionOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct RelationOutput {
    pub(super) field: String,
    pub(super) source_id: String,
    pub(super) target_id: Option<String>,
    pub(super) target_instance_id: String,
    pub(super) target_collections: Vec<String>,
    pub(super) missing: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct MissingRelationsOutput {
    pub(super) missing_relations: Vec<RelationOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct RepositoryReferencesOutput {
    pub(super) mode: &'static str,
    #[serde(serialize_with = "serialize_path")]
    pub(super) path: PathBuf,
    pub(super) references: Vec<RepositoryReferenceOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct RepositoryReferenceOutput {
    pub(super) id: String,
    #[serde(serialize_with = "serialize_path")]
    pub(super) source_path: PathBuf,
    pub(super) source_line: Option<usize>,
    pub(super) source_column: Option<usize>,
    pub(super) target_id: Option<String>,
    #[serde(serialize_with = "serialize_path")]
    pub(super) target_path: PathBuf,
    pub(super) target_anchor: Option<String>,
    pub(super) target_line_start: Option<usize>,
    pub(super) target_line_end: Option<usize>,
    pub(super) target_exists: bool,
    pub(super) reference_kind: String,
    pub(super) rule: String,
    pub(super) confidence: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DiagnosticsOutput {
    pub(super) diagnostics: Vec<DiagnosticOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SafeFixesOutput {
    pub(super) safe_fixes: Vec<SafeFixOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SafeFixOutput {
    pub(super) id: String,
    pub(super) audit_id: Option<String>,
    pub(super) diagnostic_id: String,
    pub(super) target_id: Option<String>,
    pub(super) operation: String,
    pub(super) summary: String,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) path: Option<PathBuf>,
    pub(super) line: Option<usize>,
    pub(super) column: Option<usize>,
    pub(super) field: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchOutput {
    pub(super) query: String,
    pub(super) matches: Vec<SearchMatchOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SemanticSearchOutput {
    pub(super) query: String,
    pub(super) enabled: bool,
    pub(super) provider: Option<String>,
    pub(super) message: Option<String>,
    pub(super) matches: Vec<SemanticSearchMatchOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SemanticSearchMatchOutput {
    pub(super) source_id: String,
    pub(super) source_kind: String,
    pub(super) score: f32,
    pub(super) collection: Option<String>,
    pub(super) instance_id: Option<String>,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) path: Option<PathBuf>,
    pub(super) text_hash: String,
    pub(super) text: String,
    pub(super) related: Vec<RelatedFactOutput>,
    pub(super) diagnostics: Vec<DiagnosticOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SymbolsOutput {
    pub(super) collection: String,
    pub(super) id: String,
    pub(super) source_id: String,
    pub(super) symbols: Vec<SymbolRefOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SymbolRefsOutput {
    pub(super) symbol: String,
    pub(super) references: Vec<SymbolRefOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct SymbolRefOutput {
    pub(super) source_id: String,
    pub(super) source_kind: String,
    pub(super) collection: Option<String>,
    pub(super) instance_id: Option<String>,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) source_path: Option<PathBuf>,
    pub(super) field: Option<String>,
    pub(super) symbol: String,
    pub(super) provider: Option<String>,
    pub(super) resolved: bool,
    pub(super) target_id: Option<String>,
    pub(super) target_symbol: Option<String>,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) target_path: Option<PathBuf>,
    pub(super) evidence: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchMatchOutput {
    pub(super) source_id: String,
    pub(super) source_kind: String,
    pub(super) score: f32,
    pub(super) collection: Option<String>,
    pub(super) instance_id: Option<String>,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) path: Option<PathBuf>,
    pub(super) text: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ExpandOutput {
    pub(super) root_id: String,
    pub(super) related: Vec<RelatedFactOutput>,
}

#[derive(Debug, Serialize)]
pub(super) struct RelatedFactOutput {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) relationship: String,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub(super) struct DiagnosticOutput {
    pub(super) id: String,
    pub(super) rule: String,
    pub(super) severity: String,
    pub(super) message: String,
    #[serde(serialize_with = "serialize_optional_path")]
    pub(super) path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub(super) struct SectionOutput {
    pub(super) title: String,
    pub(super) level: usize,
    pub(super) line_number: usize,
}

pub(super) fn render<T: Serialize + TextRender>(
    value: T,
    format: OutputFormat,
) -> Result<String, ContentQueryError> {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&value)
            .map_err(|error| ContentQueryError::runtime(error.to_string())),
        OutputFormat::Yaml => serde_yaml::to_string(&value)
            .map_err(|error| ContentQueryError::runtime(error.to_string())),
        OutputFormat::Text | OutputFormat::Advice | OutputFormat::Status => Ok(value.render_text()),
    }
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&portable_path(path))
}

fn serialize_optional_path<S>(path: &Option<PathBuf>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match path {
        Some(path) => serializer.serialize_some(&portable_path(path)),
        None => serializer.serialize_none(),
    }
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) trait TextRender {
    fn render_text(&self) -> String;
}
