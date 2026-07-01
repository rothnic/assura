//! Repository-reference query output helpers.

use super::context::{ContentQueryError, QueryContext};
use super::output::{RepositoryReferenceOutput, RepositoryReferencesOutput};
use crate::intelligence::{resource_id, RepositoryReferenceEdge};
use std::path::PathBuf;

pub(super) fn repository_references(
    context: &QueryContext,
    source: Option<&PathBuf>,
    target: Option<&PathBuf>,
    limit: usize,
) -> Result<RepositoryReferencesOutput, ContentQueryError> {
    match (source, target) {
        (Some(_), Some(_)) | (None, None) => Err(ContentQueryError::configuration(
            "references requires exactly one of --source or --target",
        )),
        (Some(source), None) => {
            let references = context
                .store
                .repository_references_from_path(source)
                .into_iter()
                .map(reference_output)
                .take(limit)
                .collect();
            Ok(RepositoryReferencesOutput {
                mode: "source",
                path: source.clone(),
                references,
            })
        }
        (None, Some(target)) => {
            let target_id = resource_id(target);
            let references = context
                .store
                .repository_references_to(&target_id)
                .into_iter()
                .map(reference_output)
                .take(limit)
                .collect();
            Ok(RepositoryReferencesOutput {
                mode: "target",
                path: target.clone(),
                references,
            })
        }
    }
}

fn reference_output(edge: &RepositoryReferenceEdge) -> RepositoryReferenceOutput {
    RepositoryReferenceOutput {
        id: edge.id.to_string(),
        source_path: edge.source_path.clone(),
        source_line: edge.source_line,
        source_column: edge.source_column,
        target_id: edge.target_id.as_ref().map(ToString::to_string),
        target_path: edge.target_path.clone(),
        target_anchor: edge.target_anchor.clone(),
        target_line_start: edge.target_line_start,
        target_line_end: edge.target_line_end,
        target_exists: edge.target_exists,
        reference_kind: edge.reference_kind.clone(),
        rule: edge.rule.clone(),
        confidence: edge.confidence.clone(),
    }
}
