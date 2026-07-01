use super::types::{FactId, ProjectEdge, ProjectFact};
use serde::{Deserialize, Serialize};
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
        self.facts
            .retain(|existing| existing.id() != &id || existing.generation().id != generation);
        self.facts.push(fact);
        self.facts.sort_by(|left, right| {
            left.id().cmp(right.id()).then_with(|| {
                left.generation()
                    .id
                    .as_str()
                    .cmp(right.generation().id.as_str())
            })
        });
    }

    /// Add or replace one edge by ID.
    pub fn upsert_edge(&mut self, edge: ProjectEdge) {
        let id = edge.id().clone();
        let generation = edge.generation().id.clone();
        self.edges
            .retain(|existing| existing.id() != &id || existing.generation().id != generation);
        self.edges.push(edge);
        self.edges.sort_by(|left, right| {
            left.id().cmp(right.id()).then_with(|| {
                left.generation()
                    .id
                    .as_str()
                    .cmp(right.generation().id.as_str())
            })
        });
    }

    /// Replace all facts and edges produced by one generation.
    pub fn replace_generation(&mut self, generation: &str, replacement: FactSet) {
        self.facts
            .retain(|fact| fact.generation().id.as_str() != generation);
        self.edges
            .retain(|edge| edge.generation().id.as_str() != generation);
        for fact in replacement.facts {
            self.upsert_fact(fact);
        }
        for edge in replacement.edges {
            self.upsert_edge(edge);
        }
    }

    /// Count facts with the requested variant name.
    pub fn count_kind(&self, kind: &str) -> usize {
        self.facts
            .iter()
            .filter(|fact| fact_kind(fact) == kind)
            .count()
    }
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
