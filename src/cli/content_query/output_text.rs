//! Human-readable rendering for content query output types.

use super::output::*;
use crate::intelligence::ProjectIntelligenceAgentContext;
use std::path::Path;

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
            "diagnostics: {}; missing relations: {}; repository refs: {}; safe fixes: {}",
            self.diagnostics.len(),
            self.missing_relations.len(),
            self.repository_references
                .as_ref()
                .map(|references| references.inbound.len() + references.outbound.len())
                .unwrap_or_default(),
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
            "models: {}; diagnostics: {}; safe fixes: {}; relationships: {}; repository refs: {}; symbol refs: {}",
            self.summary.model_instances,
            self.summary.diagnostics,
            self.summary.safe_fixes,
            self.summary.relationship_edges,
            self.summary.repository_reference_edges,
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

impl TextRender for RepositoryReferencesOutput {
    fn render_text(&self) -> String {
        let mut lines = vec![format!(
            "Repository references: {} {} ({})",
            self.mode,
            display_path(&self.path),
            self.references.len()
        )];
        for reference in &self.references {
            lines.push(format!(
                "source={}:{}:{} target={} anchor={} lines={} exists={} rule={} kind={} confidence={}",
                display_path(&reference.source_path),
                optional_usize(reference.source_line),
                optional_usize(reference.source_column),
                display_path(&reference.target_path),
                optional_string(reference.target_anchor.as_deref()),
                target_lines(reference.target_line_start, reference.target_line_end),
                reference.target_exists,
                reference.rule,
                reference.reference_kind,
                reference.confidence,
            ));
        }
        lines.join("\n")
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn optional_usize(value: Option<usize>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn optional_string(value: Option<&str>) -> String {
    value
        .map(ToString::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn target_lines(start: Option<usize>, end: Option<usize>) -> String {
    match (start, end) {
        (Some(start), Some(end)) => format!("{start}-{end}"),
        (Some(start), None) => start.to_string(),
        (None, Some(end)) => format!("-{end}"),
        (None, None) => "-".to_string(),
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
        let mut lines = vec![format!(
            "Search matches: {} mode={} fallback={}",
            self.matches.len(),
            self.mode,
            self.fallback_used
        )];
        for item in &self.matches {
            let label = item
                .instance_id
                .as_deref()
                .unwrap_or(item.source_id.as_str());
            let location = item.path.as_ref().map(|path| {
                format!(
                    " [{}:{}:{}]",
                    display_path(path),
                    optional_usize(item.line),
                    optional_usize(item.column)
                )
            });
            lines.push(format!(
                "{:.3} {}{} - {}",
                item.score,
                label,
                location.unwrap_or_default(),
                compact_text(&item.text)
            ));
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
            lines.push(format!(
                "{:.3} {} - {}",
                item.score,
                label,
                compact_text(&item.text)
            ));
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

fn compact_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
