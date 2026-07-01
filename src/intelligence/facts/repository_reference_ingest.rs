use super::repository_references::RepositoryReferenceEdge;
use super::set::{normalize_path, resource_id};
use super::types::{EdgeId, FactId, FactOrigin, ProjectEdge, ProjectFact, Resource};
use super::FactIngestor;
use crate::repository_references::{source_references, SourceReference};
use std::path::Path;

impl FactIngestor {
    /// Ingest conservative source/comment/string repository references.
    pub fn ingest_source_references(
        &mut self,
        project_root: &Path,
        source_rel: &Path,
        content: &str,
    ) {
        let references = source_references(source_rel, content);
        if references.is_empty() {
            return;
        }
        let source_id = self.upsert_repository_reference_resource(source_rel);
        for reference in references {
            let target_exists = project_root.join(&reference.target_path).is_file();
            let target_id = target_exists
                .then(|| self.upsert_repository_reference_resource(&reference.target_path));
            self.facts
                .upsert_edge(ProjectEdge::RepositoryReference(RepositoryReferenceEdge {
                    id: EdgeId::from_parts(
                        "repository_reference",
                        &format!(
                            "{}:{}:{}:{}",
                            normalize_path(source_rel),
                            reference.line_number,
                            reference.column_number,
                            reference.raw
                        ),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    source_id: source_id.clone(),
                    target_id,
                    source_path: source_rel.to_path_buf(),
                    source_line: Some(reference.line_number),
                    source_column: Some(reference.column_number),
                    target_path: reference.target_path.clone(),
                    target_anchor: reference.target_anchor.clone(),
                    target_line_start: reference.target_line_start,
                    target_line_end: reference.target_line_end,
                    target_exists,
                    reference_kind: reference.kind.to_string(),
                    rule: reference_rule(&reference, target_exists),
                    confidence: reference.confidence.to_string(),
                }));
        }
    }

    fn upsert_repository_reference_resource(&mut self, path: &Path) -> FactId {
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

fn reference_rule(reference: &SourceReference, target_exists: bool) -> String {
    if !target_exists {
        "repository_reference_target".to_string()
    } else if reference.target_line_start.is_some() {
        "repository_reference_line_anchor".to_string()
    } else if reference.target_anchor.is_some() {
        "repository_reference_anchor".to_string()
    } else {
        "repository_reference_target".to_string()
    }
}
