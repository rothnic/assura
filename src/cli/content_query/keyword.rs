//! Deterministic keyword-search command behavior.

use super::context::QueryContext;
use super::facts::{document_path, fact_by_id, resources_by_id};
use super::output::{SearchMatchOutput, SearchOutput};
use crate::intelligence::{FactId, ProjectFact, Resource, SearchChunk};
use std::collections::BTreeMap;

pub(super) fn search(context: &QueryContext, query: &str) -> SearchOutput {
    let resources = resources_by_id(context.store.facts());
    let mut matches = context
        .store
        .keyword_search(query)
        .into_iter()
        .map(|chunk| {
            let score = lexical_match_score(query, &chunk.text);
            search_match(context, chunk, score, &resources)
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.source_id.cmp(&right.source_id))
    });
    SearchOutput {
        query: query.to_string(),
        matches,
    }
}

fn lexical_match_score(query: &str, text: &str) -> f32 {
    let normalized_text = text.to_ascii_lowercase();
    let normalized_query = query.to_ascii_lowercase();
    let terms = normalized_query
        .split_whitespace()
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return 0.0;
    }

    let occurrences = terms
        .iter()
        .map(|term| normalized_text.matches(term).count())
        .sum::<usize>();
    let phrase_boost = if normalized_text.contains(normalized_query.trim()) {
        1.0
    } else {
        0.0
    };

    occurrences as f32 / terms.len() as f32 + phrase_boost
}

fn search_match(
    context: &QueryContext,
    chunk: &SearchChunk,
    score: f32,
    resources: &BTreeMap<FactId, Resource>,
) -> SearchMatchOutput {
    match fact_by_id(context, &chunk.source_id) {
        Some(ProjectFact::ModelInstance(instance)) => SearchMatchOutput {
            source_id: chunk.source_id.to_string(),
            source_kind: "model_instance".to_string(),
            score,
            collection: Some(instance.collection.clone()),
            instance_id: Some(instance.instance_id.clone()),
            path: resources
                .get(&instance.resource_id)
                .map(|resource| resource.path.clone()),
            text: chunk.text.clone(),
        },
        Some(ProjectFact::MarkdownSection(section)) => SearchMatchOutput {
            source_id: chunk.source_id.to_string(),
            source_kind: "markdown_section".to_string(),
            score,
            collection: None,
            instance_id: None,
            path: document_path(context, &section.document_id),
            text: chunk.text.clone(),
        },
        Some(ProjectFact::Diagnostic(diagnostic)) => SearchMatchOutput {
            source_id: chunk.source_id.to_string(),
            source_kind: "diagnostic".to_string(),
            score,
            collection: None,
            instance_id: None,
            path: diagnostic
                .location
                .as_ref()
                .map(|location| location.path.clone()),
            text: chunk.text.clone(),
        },
        _ => SearchMatchOutput {
            source_id: chunk.source_id.to_string(),
            source_kind: "unknown".to_string(),
            score,
            collection: None,
            instance_id: None,
            path: None,
            text: chunk.text.clone(),
        },
    }
}
