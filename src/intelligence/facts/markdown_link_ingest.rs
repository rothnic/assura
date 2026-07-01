use super::ingest::FactIngestor;
use super::markdown_links::MarkdownLink as MarkdownLinkFact;
use super::repository_references::RepositoryReferenceEdge;
use super::set::{normalize_path, resource_id};
use super::types::{
    EdgeId, FactId, FactOrigin, MarkdownDocument, ProjectEdge, ProjectFact, Resource,
};
use crate::markdown::links::{
    is_markdown_file, markdown_links, parse_line_anchor, parse_markdown_link_target,
};
use std::path::Path;

impl FactIngestor {
    /// Ingest Markdown-authored local link facts from one source document.
    pub fn ingest_markdown_links(&mut self, project_root: &Path, source_rel: &Path, content: &str) {
        let source_resource_id = self.upsert_markdown_link_resource(source_rel);
        let document_id = markdown_document_id(source_rel);
        self.facts
            .upsert_fact(ProjectFact::MarkdownDocument(MarkdownDocument {
                id: document_id.clone(),
                generation: self.generation.clone(),
                origin: FactOrigin::Source,
                resource_id: source_resource_id,
                path: source_rel.to_path_buf(),
            }));

        for link in markdown_links(content) {
            let Some(target) = parse_markdown_link_target(source_rel, &link.target) else {
                continue;
            };
            let target_exists = project_root.join(&target.path).is_file();
            let target_id = target_exists.then(|| self.upsert_markdown_link_resource(&target.path));
            let (target_line_start, target_line_end) = target
                .anchor
                .as_deref()
                .and_then(parse_line_anchor)
                .map(|(start, end)| (Some(start), Some(end)))
                .unwrap_or((None, None));
            let rule = markdown_link_rule(&target.path, target.anchor.as_deref(), target_exists);
            let target_path = target.path;
            let target_anchor = target.anchor;

            let link_key = format!(
                "{}:{}:{}:{}",
                normalize_path(source_rel),
                link.line_number,
                link.column_number,
                link.target
            );
            let link_id = FactId::from_parts("markdown_link", &link_key);
            self.facts
                .upsert_fact(ProjectFact::MarkdownLink(MarkdownLinkFact {
                    id: link_id.clone(),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    document_id: document_id.clone(),
                    source_path: source_rel.to_path_buf(),
                    source_line: link.line_number,
                    source_column: link.column_number,
                    raw_target: link.target,
                    target_path: target_path.clone(),
                    target_anchor: target_anchor.clone(),
                    target_line_start,
                    target_line_end,
                    target_exists,
                    rule: rule.clone(),
                }));
            self.facts
                .upsert_edge(ProjectEdge::RepositoryReference(RepositoryReferenceEdge {
                    id: EdgeId::from_parts("repository_reference", &link_key),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    source_id: link_id,
                    target_id,
                    source_path: source_rel.to_path_buf(),
                    source_line: Some(link.line_number),
                    source_column: Some(link.column_number),
                    target_path,
                    target_anchor,
                    target_line_start,
                    target_line_end,
                    target_exists,
                    reference_kind: "markdown_link".to_string(),
                    rule,
                    confidence: "exact".to_string(),
                }));
        }
    }

    fn upsert_markdown_link_resource(&mut self, path: &Path) -> FactId {
        let id = resource_id(path);
        self.facts.upsert_fact(ProjectFact::Resource(Resource {
            id: id.clone(),
            generation: self.generation.clone(),
            origin: FactOrigin::Source,
            path: path.to_path_buf(),
            extension: path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(ToOwned::to_owned),
        }));
        id
    }
}

fn markdown_document_id(path: impl AsRef<Path>) -> FactId {
    FactId::from_parts("markdown_document", &normalize_path(path.as_ref()))
}

fn markdown_link_rule(target_path: &Path, anchor: Option<&str>, target_exists: bool) -> String {
    match anchor {
        _ if !target_exists => "markdown_link_target",
        Some(anchor) if parse_line_anchor(anchor).is_some() => "markdown_link_line_anchor",
        Some(_) if is_markdown_file(target_path) => "markdown_link_heading_anchor",
        _ => "markdown_link_target",
    }
    .to_string()
}
