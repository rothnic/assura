//! Serializable content query output types.

use super::context::ContentQueryError;
use crate::cli::OutputFormat;
use crate::intelligence::ProjectIntelligenceAgentContext;
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::PathBuf;

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
    pub(super) project_root: PathBuf,
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
    pub(super) project_root: PathBuf,
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
    pub(super) project_root: PathBuf,
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
    pub(super) path: PathBuf,
    pub(super) title: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct InstanceOutput {
    pub(super) id: String,
    pub(super) collection: String,
    pub(super) object_type: String,
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
    pub(super) source_path: Option<PathBuf>,
    pub(super) field: Option<String>,
    pub(super) symbol: String,
    pub(super) provider: Option<String>,
    pub(super) resolved: bool,
    pub(super) target_id: Option<String>,
    pub(super) target_symbol: Option<String>,
    pub(super) target_path: Option<PathBuf>,
    pub(super) evidence: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SearchMatchOutput {
    pub(super) source_id: String,
    pub(super) source_kind: String,
    pub(super) collection: Option<String>,
    pub(super) instance_id: Option<String>,
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
    pub(super) path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub(super) struct DiagnosticOutput {
    pub(super) id: String,
    pub(super) rule: String,
    pub(super) severity: String,
    pub(super) message: String,
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

pub(super) trait TextRender {
    fn render_text(&self) -> String;
}

impl TextRender for AgentQueryOutput {
    fn render_text(&self) -> String {
        format!(
            "Agent query: {} via {}",
            self.request.capability, self.request.cli
        )
    }
}

impl TextRender for ContextPackOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!("Context pack: {}", self.request.mode)];
        lines.push(format!(
            "diagnostics: {}; missing relations: {}; safe fixes: {}",
            self.diagnostics.len(),
            self.missing_relations.len(),
            self.safe_fixes.len()
        ));
        if let Some(instance) = &self.instance {
            lines.push(format!(
                "instance: {}:{} ({})",
                instance.collection,
                instance.id,
                instance.path.display()
            ));
        }
        if let Some(search) = &self.search {
            lines.push(format!("search matches: {}", search.matches.len()));
        }
        if !self.bounds.omissions.is_empty() {
            lines.push(format!("omissions: {}", self.bounds.omissions.len()));
        }
        if !self.bounds.truncated.is_empty() {
            lines.push(format!("truncated: {}", self.bounds.truncated.len()));
        }
        lines.join("\n")
    }
}

impl TextRender for CollectionsOutput {
    fn render_text(&self) -> String {
        let mut lines = vec!["Content collections".to_string()];
        for collection in &self.collections {
            lines.push(format!(
                "{} ({}) - {} instance(s)",
                collection.collection, collection.object_type, collection.instances
            ));
        }
        lines.join("\n")
    }
}

impl TextRender for ProjectIntelligenceAgentContext {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Project intelligence agent context: {}",
            self.schema
        )];
        lines.push(format!(
            "models: {}; diagnostics: {}; safe fixes: {}; relationships: {}; symbol refs: {}",
            self.summary.model_instances,
            self.summary.diagnostics,
            self.summary.safe_fixes,
            self.summary.relationship_edges,
            self.summary.symbol_refs
        ));
        for capability in &self.capabilities {
            lines.push(format!(
                "{} [{}] - {}",
                capability.name, capability.status, capability.cli
            ));
        }
        lines.join("\n")
    }
}

impl TextRender for InstancesOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!("Content instances: {}", self.collection)];
        for instance in &self.instances {
            lines.push(format!("{} - {}", instance.id, instance.path.display()));
        }
        lines.join("\n")
    }
}

impl TextRender for InstanceOutput {
    fn render_text(&self) -> String {
        format!(
            "{}:{}\npath: {}\noutgoing: {}\nincoming: {}\ndiagnostics: {}",
            self.collection,
            self.id,
            self.path.display(),
            self.outgoing_relations.len(),
            self.incoming_relations.len(),
            self.diagnostics.len()
        )
    }
}

impl TextRender for MissingRelationsOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Missing relations: {}",
            self.missing_relations.len()
        )];
        for relation in &self.missing_relations {
            lines.push(format!(
                "{} -> {} ({})",
                relation.source_id, relation.target_instance_id, relation.field
            ));
        }
        lines.join("\n")
    }
}

impl TextRender for DiagnosticsOutput {
    fn render_text(&self) -> String {
        format!("Diagnostics: {}", self.diagnostics.len())
    }
}

impl TextRender for SafeFixesOutput {
    fn render_text(&self) -> String {
        format!("Safe fixes: {}", self.safe_fixes.len())
    }
}

impl TextRender for SearchOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!("Search matches: {}", self.matches.len())];
        for item in &self.matches {
            let label = item
                .instance_id
                .as_deref()
                .unwrap_or(item.source_id.as_str());
            lines.push(format!("{} - {}", label, item.text));
        }
        lines.join("\n")
    }
}

impl TextRender for SemanticSearchOutput {
    fn render_text(&self) -> String {
        if !self.enabled {
            return self
                .message
                .clone()
                .unwrap_or_else(|| "Semantic search disabled".to_string());
        }
        let mut lines = vec![format!("Semantic candidates: {}", self.matches.len())];
        for item in &self.matches {
            let label = item
                .instance_id
                .as_deref()
                .unwrap_or(item.source_id.as_str());
            lines.push(format!("{:.3} {} - {}", item.score, label, item.text));
        }
        lines.join("\n")
    }
}

impl TextRender for SymbolsOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Code symbols for {}:{} ({})",
            self.collection,
            self.id,
            self.symbols.len()
        )];
        for item in &self.symbols {
            lines.push(format!(
                "{} {}{}",
                if item.resolved {
                    "resolved"
                } else {
                    "unresolved"
                },
                item.symbol,
                item.provider
                    .as_deref()
                    .map(|provider| format!(" [{provider}]"))
                    .unwrap_or_default()
            ));
        }
        lines.join("\n")
    }
}

impl TextRender for SymbolRefsOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Code symbol references for {}: {}",
            self.symbol,
            self.references.len()
        )];
        for item in &self.references {
            let source = item
                .instance_id
                .as_deref()
                .unwrap_or(item.source_id.as_str());
            lines.push(format!(
                "{} {} -> {}",
                if item.resolved {
                    "resolved"
                } else {
                    "unresolved"
                },
                source,
                item.symbol
            ));
        }
        lines.join("\n")
    }
}

impl TextRender for ExpandOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!("Graph expansion: {}", self.root_id)];
        for item in &self.related {
            lines.push(format!("{} {} {}", item.relationship, item.kind, item.id));
        }
        lines.join("\n")
    }
}
