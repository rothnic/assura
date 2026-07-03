//! Repository-reference query output helpers.

use super::context::{ContentQueryError, QueryContext};
use super::output::{RepositoryReferenceOutput, RepositoryReferencesOutput};
use crate::intelligence::{resource_id, RepositoryReferenceEdge};
use std::path::PathBuf;

pub(super) struct ReferenceMode<'a> {
    pub(super) source: Option<&'a PathBuf>,
    pub(super) target: Option<&'a PathBuf>,
    pub(super) all: bool,
    pub(super) unresolved: bool,
}

pub(super) fn repository_references(
    context: &QueryContext,
    mode: ReferenceMode<'_>,
    limit: usize,
) -> Result<RepositoryReferencesOutput, ContentQueryError> {
    let selector_count = usize::from(mode.source.is_some())
        + usize::from(mode.target.is_some())
        + usize::from(mode.all)
        + usize::from(mode.unresolved);
    if selector_count != 1 {
        return Err(ContentQueryError::configuration(
            "references requires exactly one of --source, --target, --all, or --unresolved",
        ));
    }

    match (mode.source, mode.target, mode.all, mode.unresolved) {
        (Some(source), None, false, false) => {
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
        (None, Some(target), false, false) => {
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
        (None, None, true, false) => {
            let references = context
                .store
                .repository_references()
                .into_iter()
                .map(reference_output)
                .take(limit)
                .collect();
            Ok(RepositoryReferencesOutput {
                mode: "all",
                path: PathBuf::from("."),
                references,
            })
        }
        (None, None, false, true) => {
            let references = context
                .store
                .unresolved_repository_references()
                .into_iter()
                .map(reference_output)
                .take(limit)
                .collect();
            Ok(RepositoryReferencesOutput {
                mode: "unresolved",
                path: PathBuf::from("."),
                references,
            })
        }
        _ => unreachable!("selector count was already validated"),
    }
}

pub(super) fn reference_output(edge: &RepositoryReferenceEdge) -> RepositoryReferenceOutput {
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
