//! Local semantic-search command behavior.

use super::context::QueryContext;
use super::facts::fact_kind;
use super::output::{
    DiagnosticOutput, RelatedFactOutput, SemanticSearchMatchOutput, SemanticSearchOutput,
};
use crate::intelligence::{
    local_hash_embedding, FactGeneration, FactId, ProjectEdge, ProjectFact,
    LOCAL_HASH_EMBEDDING_PROVIDER,
};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(super) fn semantic_search(
    context: &QueryContext,
    query: &str,
    limit: usize,
    enable_local: bool,
) -> SemanticSearchOutput {
    if !enable_local {
        return SemanticSearchOutput {
            query: query.to_string(),
            enabled: false,
            provider: None,
            message: Some(
                "Semantic search is disabled; pass --enable-local to use the local baseline"
                    .to_string(),
            ),
            matches: Vec::new(),
        };
    }

    let query_vector = local_hash_embedding(query);
    let matches = context
        .store
        .semantic_search(&query_vector, LOCAL_HASH_EMBEDDING_PROVIDER, limit)
        .into_iter()
        .map(|hit| semantic_match(context, hit))
        .collect();

    SemanticSearchOutput {
        query: query.to_string(),
        enabled: true,
        provider: Some(LOCAL_HASH_EMBEDDING_PROVIDER.to_string()),
        message: None,
        matches,
    }
}

fn semantic_match(
    context: &QueryContext,
    hit: crate::intelligence::SemanticSearchHit<'_>,
) -> SemanticSearchMatchOutput {
    let Some(fact) = fact_by_id_in_generation(context, &hit.chunk.source_id, &hit.chunk.generation)
    else {
        return SemanticSearchMatchOutput {
            source_id: hit.chunk.source_id.to_string(),
            source_kind: "unknown".to_string(),
            score: hit.score,
            collection: None,
            instance_id: None,
            path: None,
            text_hash: hit.embedding.text_hash.clone(),
            text: hit.chunk.text.clone(),
            related: Vec::new(),
            diagnostics: Vec::new(),
        };
    };

    match fact {
        ProjectFact::ModelInstance(instance) => SemanticSearchMatchOutput {
            source_id: hit.chunk.source_id.to_string(),
            source_kind: "model_instance".to_string(),
            score: hit.score,
            collection: Some(instance.collection.clone()),
            instance_id: Some(instance.instance_id.clone()),
            path: semantic_fact_path(context, fact, &hit.chunk.generation),
            text_hash: hit.embedding.text_hash.clone(),
            text: hit.chunk.text.clone(),
            related: related_context(context, &hit.chunk.source_id, &hit.chunk.generation, 5),
            diagnostics: diagnostics_for_targets(
                context,
                &[hit.chunk.source_id.clone(), instance.resource_id.clone()],
                &hit.chunk.generation,
            ),
        },
        _ => SemanticSearchMatchOutput {
            source_id: hit.chunk.source_id.to_string(),
            source_kind: fact_kind(fact).to_string(),
            score: hit.score,
            collection: None,
            instance_id: None,
            path: semantic_fact_path(context, fact, &hit.chunk.generation),
            text_hash: hit.embedding.text_hash.clone(),
            text: hit.chunk.text.clone(),
            related: related_context(context, &hit.chunk.source_id, &hit.chunk.generation, 5),
            diagnostics: diagnostics_for_targets(
                context,
                std::slice::from_ref(&hit.chunk.source_id),
                &hit.chunk.generation,
            ),
        },
    }
}

fn related_context(
    context: &QueryContext,
    source_id: &FactId,
    generation: &FactGeneration,
    limit: usize,
) -> Vec<RelatedFactOutput> {
    let mut related = Vec::new();
    for edge in context.store.edges_from(source_id) {
        if edge.generation().id != generation.id {
            continue;
        }
        if related.len() >= limit {
            break;
        }
        match edge {
            ProjectEdge::Relationship(edge) => {
                if let Some(target_id) = &edge.target_id {
                    push_related(
                        context,
                        &mut related,
                        target_id,
                        generation,
                        "outgoing_relation",
                    );
                }
            }
            ProjectEdge::SymbolRef(edge) => related.push(RelatedFactOutput {
                id: edge.id.to_string(),
                kind: "symbol_ref".to_string(),
                relationship: "symbol_ref".to_string(),
                path: None,
            }),
        }
    }
    related
}

fn push_related(
    context: &QueryContext,
    related: &mut Vec<RelatedFactOutput>,
    id: &FactId,
    generation: &FactGeneration,
    relationship: &str,
) {
    if let Some(fact) = fact_by_id_in_generation(context, id, generation) {
        related.push(RelatedFactOutput {
            id: id.to_string(),
            kind: fact_kind(fact).to_string(),
            relationship: relationship.to_string(),
            path: semantic_fact_path(context, fact, generation),
        });
    }
}

fn diagnostics_for_targets(
    context: &QueryContext,
    targets: &[FactId],
    generation: &FactGeneration,
) -> Vec<DiagnosticOutput> {
    let targets = targets.iter().collect::<BTreeSet<_>>();
    context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic)
                if diagnostic.generation.id == generation.id
                    && diagnostic
                        .target_id
                        .as_ref()
                        .is_some_and(|target| targets.contains(target)) =>
            {
                Some(DiagnosticOutput {
                    id: diagnostic.id.to_string(),
                    rule: diagnostic.rule.clone(),
                    severity: diagnostic.severity.clone(),
                    message: diagnostic.message.clone(),
                    path: diagnostic
                        .location
                        .as_ref()
                        .map(|location| location.path.clone()),
                })
            }
            _ => None,
        })
        .collect()
}

fn fact_by_id_in_generation<'a>(
    context: &'a QueryContext,
    id: &FactId,
    generation: &FactGeneration,
) -> Option<&'a ProjectFact> {
    context
        .store
        .facts_by_id(id)
        .into_iter()
        .find(|fact| fact.generation().id == generation.id)
}

fn semantic_fact_path(
    context: &QueryContext,
    fact: &ProjectFact,
    generation: &FactGeneration,
) -> Option<PathBuf> {
    match fact {
        ProjectFact::Resource(resource) => Some(resource.path.clone()),
        ProjectFact::ModelInstance(instance) => {
            fact_by_id_in_generation(context, &instance.resource_id, generation).and_then(|fact| {
                match fact {
                    ProjectFact::Resource(resource) => Some(resource.path.clone()),
                    _ => None,
                }
            })
        }
        ProjectFact::MarkdownDocument(document) => Some(document.path.clone()),
        ProjectFact::MarkdownSection(section) => {
            fact_by_id_in_generation(context, &section.document_id, generation).and_then(|fact| {
                match fact {
                    ProjectFact::MarkdownDocument(document) => Some(document.path.clone()),
                    _ => None,
                }
            })
        }
        ProjectFact::Diagnostic(diagnostic) => diagnostic
            .location
            .as_ref()
            .map(|location| location.path.clone()),
        _ => None,
    }
}
