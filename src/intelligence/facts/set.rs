use super::types::{FactId, ProjectEdge, ProjectFact};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// Complete fact set for one or more project intelligence generations.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FactSet {
    /// Node-like facts.
    pub facts: Vec<ProjectFact>,
    /// Edge-like facts.
    pub edges: Vec<ProjectEdge>,
}

impl FactSet {
    /// Add or replace one fact by ID.
    pub fn upsert_fact(&mut self, fact: ProjectFact) {
        let id = fact.id().clone();
        let generation = fact.generation().id.clone();
        if let Some(existing) = self
            .facts
            .iter()
            .position(|existing| existing.id() == &id && existing.generation().id == generation)
        {
            self.facts[existing] = fact;
        } else {
            self.facts.push(fact);
        }
    }

    /// Add or replace one edge by ID.
    pub fn upsert_edge(&mut self, edge: ProjectEdge) {
        let id = edge.id().clone();
        let generation = edge.generation().id.clone();
        if let Some(existing) = self
            .edges
            .iter()
            .position(|existing| existing.id() == &id && existing.generation().id == generation)
        {
            self.edges[existing] = edge;
        } else {
            self.edges.push(edge);
        }
    }

    /// Replace all facts and edges produced by one generation.
    pub fn replace_generation(&mut self, generation: &str, replacement: FactSet) {
        self.facts
            .retain(|fact| fact.generation().id.as_str() != generation);
        self.edges
            .retain(|edge| edge.generation().id.as_str() != generation);
        self.facts.extend(replacement.facts);
        self.edges.extend(replacement.edges);
        self.sort_stable();
        dedupe_facts_keep_last(&mut self.facts);
        dedupe_edges_keep_last(&mut self.edges);
    }

    /// Sort facts and edges into deterministic ID/generation order.
    pub fn sort_stable(&mut self) {
        self.facts.sort_by(|left, right| {
            left.id().cmp(right.id()).then_with(|| {
                left.generation()
                    .id
                    .as_str()
                    .cmp(right.generation().id.as_str())
            })
        });
        self.edges.sort_by(|left, right| {
            left.id().cmp(right.id()).then_with(|| {
                left.generation()
                    .id
                    .as_str()
                    .cmp(right.generation().id.as_str())
            })
        });
    }

    /// Count facts with the requested variant name.
    pub fn count_kind(&self, kind: &str) -> usize {
        self.facts
            .iter()
            .filter(|fact| fact_kind(fact) == kind)
            .count()
    }
}

fn dedupe_facts_keep_last(facts: &mut Vec<ProjectFact>) {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::with_capacity(facts.len());
    for fact in facts.drain(..).rev() {
        let key = (fact.id().clone(), fact.generation().id.clone());
        if seen.insert(key) {
            deduped.push(fact);
        }
    }
    deduped.reverse();
    *facts = deduped;
}

fn dedupe_edges_keep_last(edges: &mut Vec<ProjectEdge>) {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::with_capacity(edges.len());
    for edge in edges.drain(..).rev() {
        let key = (edge.id().clone(), edge.generation().id.clone());
        if seen.insert(key) {
            deduped.push(edge);
        }
    }
    deduped.reverse();
    *edges = deduped;
}

/// Create a stable model definition ID for a collection binding.
pub fn model_definition_id(collection: &str, object_type: &str) -> FactId {
    FactId::from_parts("model", &format!("{collection}:{object_type}"))
}

/// Create a stable model instance ID.
pub fn model_instance_id(collection: &str, instance_id: &str) -> FactId {
    FactId::from_parts("instance", &format!("{collection}:{instance_id}"))
}

/// Create a stable resource ID for a repository-relative path.
pub fn resource_id(path: impl AsRef<Path>) -> FactId {
    FactId::from_parts("resource", &normalize_path(path.as_ref()))
}

pub(crate) fn normalize_path(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}

fn fact_kind(fact: &ProjectFact) -> &'static str {
    match fact {
        ProjectFact::ModelDefinition(_) => "ModelDefinition",
        ProjectFact::FieldDefinition(_) => "FieldDefinition",
        ProjectFact::RelationshipDefinition(_) => "RelationshipDefinition",
        ProjectFact::PathScope(_) => "PathScope",
        ProjectFact::Resource(_) => "Resource",
        ProjectFact::MarkdownDocument(_) => "MarkdownDocument",
        ProjectFact::MarkdownSection(_) => "MarkdownSection",
        ProjectFact::MarkdownLink(_) => "MarkdownLink",
        ProjectFact::ModelInstance(_) => "ModelInstance",
        ProjectFact::Diagnostic(_) => "Diagnostic",
        ProjectFact::SafeFix(_) => "SafeFix",
        ProjectFact::SearchChunk(_) => "SearchChunk",
        ProjectFact::EmbeddingRecord(_) => "EmbeddingRecord",
        ProjectFact::CodeSymbol(_) => "CodeSymbol",
        ProjectFact::CodeProviderEvidence(_) => "CodeProviderEvidence",
    }
}
