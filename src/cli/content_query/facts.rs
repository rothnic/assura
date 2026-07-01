//! Fact lookup helpers for content query commands.

use super::context::QueryContext;
use super::output::{InstanceSummary, SectionOutput};
use crate::intelligence::{FactId, FactSet, ModelDefinition, ModelInstance, ProjectFact, Resource};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn sections_for_path(
    context: &QueryContext,
    path: &std::path::Path,
) -> Vec<SectionOutput> {
    let Some(document_id) = context
        .store
        .facts()
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::MarkdownDocument(document) if document.path == path => {
                Some(document.id.clone())
            }
            _ => None,
        })
    else {
        return Vec::new();
    };

    context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::MarkdownSection(section) if section.document_id == document_id => {
                Some(SectionOutput {
                    title: section.title.clone(),
                    level: section.level,
                    line_number: section.line_number,
                })
            }
            _ => None,
        })
        .collect()
}

pub(super) fn model_definitions(context: &QueryContext) -> Vec<&ModelDefinition> {
    let mut models = context
        .store
        .facts()
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::ModelDefinition(model) => Some(model),
            _ => None,
        })
        .collect::<Vec<_>>();
    models.sort_by(|left, right| left.collection.cmp(&right.collection));
    models
}

pub(super) fn path_scope_for_collection(
    context: &QueryContext,
    collection: &str,
) -> Option<String> {
    context
        .store
        .facts()
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::PathScope(scope) if scope.collection == collection => {
                Some(scope.pattern.clone())
            }
            _ => None,
        })
}

pub(super) fn instance_summary(
    instance: &ModelInstance,
    resources: &BTreeMap<FactId, Resource>,
) -> InstanceSummary {
    InstanceSummary {
        id: instance.instance_id.clone(),
        object_type: instance.object_type.clone(),
        path: resources
            .get(&instance.resource_id)
            .map(|resource| resource.path.clone())
            .unwrap_or_default(),
        title: instance
            .data
            .get("title")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    }
}

pub(super) fn resources_by_id(facts: &FactSet) -> BTreeMap<FactId, Resource> {
    facts
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::Resource(resource) => Some((resource.id.clone(), resource.clone())),
            _ => None,
        })
        .collect()
}

pub(super) fn fact_by_id<'a>(context: &'a QueryContext, id: &FactId) -> Option<&'a ProjectFact> {
    context.store.facts_by_id(id).into_iter().next()
}

pub(super) fn document_path(context: &QueryContext, document_id: &FactId) -> Option<PathBuf> {
    context
        .store
        .facts_by_id(document_id)
        .into_iter()
        .find_map(|fact| match fact {
            ProjectFact::MarkdownDocument(document) => Some(document.path.clone()),
            _ => None,
        })
}

pub(super) fn fact_path(context: &QueryContext, fact: &ProjectFact) -> Option<PathBuf> {
    let resources = resources_by_id(context.store.facts());
    match fact {
        ProjectFact::Resource(resource) => Some(resource.path.clone()),
        ProjectFact::ModelInstance(instance) => resources
            .get(&instance.resource_id)
            .map(|resource| resource.path.clone()),
        ProjectFact::MarkdownDocument(document) => Some(document.path.clone()),
        ProjectFact::MarkdownSection(section) => document_path(context, &section.document_id),
        ProjectFact::MarkdownLink(link) => Some(link.source_path.clone()),
        ProjectFact::Diagnostic(diagnostic) => diagnostic
            .location
            .as_ref()
            .map(|location| location.path.clone()),
        _ => None,
    }
}

pub(super) fn fact_kind(fact: &ProjectFact) -> &'static str {
    match fact {
        ProjectFact::ModelDefinition(_) => "model_definition",
        ProjectFact::FieldDefinition(_) => "field_definition",
        ProjectFact::RelationshipDefinition(_) => "relationship_definition",
        ProjectFact::PathScope(_) => "path_scope",
        ProjectFact::Resource(_) => "resource",
        ProjectFact::MarkdownDocument(_) => "markdown_document",
        ProjectFact::MarkdownSection(_) => "markdown_section",
        ProjectFact::MarkdownLink(_) => "markdown_link",
        ProjectFact::ModelInstance(_) => "model_instance",
        ProjectFact::Diagnostic(_) => "diagnostic",
        ProjectFact::SafeFix(_) => "safe_fix",
        ProjectFact::SearchChunk(_) => "search_chunk",
        ProjectFact::EmbeddingRecord(_) => "embedding_record",
        ProjectFact::CodeSymbol(_) => "code_symbol",
        ProjectFact::CodeProviderEvidence(_) => "code_provider_evidence",
    }
}
