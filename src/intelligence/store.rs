//! Lightweight project intelligence fact store.
//!
//! This store is the Assura-owned fallback for graph/search spikes. It keeps
//! checked repository files as canonical state and indexes normalized facts for
//! local traversal and query workloads.

use super::facts::{
    CodeSymbol, EdgeId, EmbeddingRecord, FactId, FactSet, PathScope, ProjectEdge, ProjectFact,
    RelationshipEdge, RepositoryReferenceEdge, SearchChunk, SymbolRef,
};
use super::semantic::{cosine_similarity, semantic_text_hash};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Local in-memory indexes over normalized project intelligence facts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InMemoryFactStore {
    facts: FactSet,
    facts_by_id: BTreeMap<FactId, Vec<usize>>,
    edges_by_id: BTreeMap<EdgeId, Vec<usize>>,
    edges_by_source: BTreeMap<FactId, Vec<usize>>,
    repository_references_by_source_path: BTreeMap<PathBuf, Vec<usize>>,
    repository_references_by_target: BTreeMap<FactId, Vec<usize>>,
    missing_relationship_edges: Vec<usize>,
    search_chunks: Vec<SearchChunk>,
    embedding_records: Vec<EmbeddingRecord>,
    code_symbols: Vec<CodeSymbol>,
    symbol_refs: Vec<SymbolRef>,
    path_scopes: Vec<PathScope>,
}

impl InMemoryFactStore {
    /// Build a store from a fact set.
    pub fn load(facts: FactSet) -> Self {
        let mut store = Self {
            facts,
            ..Self::default()
        };
        store.rebuild_indexes();
        store
    }

    /// Replace all facts and edges for one generation and rebuild indexes.
    pub fn replace_generation(&mut self, generation: &str, replacement: FactSet) {
        self.facts.replace_generation(generation, replacement);
        self.rebuild_indexes();
    }

    /// Return the underlying fact set.
    pub fn facts(&self) -> &FactSet {
        &self.facts
    }

    /// Return facts with the requested stable ID across all retained generations.
    pub fn facts_by_id(&self, id: &FactId) -> Vec<&ProjectFact> {
        self.facts_by_id
            .get(id)
            .into_iter()
            .flat_map(|indexes| indexes.iter())
            .map(|index| &self.facts.facts[*index])
            .collect()
    }

    /// Return edges with the requested stable ID across all retained generations.
    pub fn edges_by_id(&self, id: &EdgeId) -> Vec<&ProjectEdge> {
        self.edges_by_id
            .get(id)
            .into_iter()
            .flat_map(|indexes| indexes.iter())
            .map(|index| &self.facts.edges[*index])
            .collect()
    }

    /// Expand graph edges from a source fact.
    pub fn edges_from(&self, source_id: &FactId) -> Vec<&ProjectEdge> {
        self.edges_by_source
            .get(source_id)
            .into_iter()
            .flat_map(|indexes| indexes.iter())
            .map(|index| &self.facts.edges[*index])
            .collect()
    }

    /// Return relationship edges that do not resolve to exactly one target fact.
    pub fn missing_relationship_targets(&self) -> Vec<&RelationshipEdge> {
        self.missing_relationship_edges
            .iter()
            .filter_map(|index| match &self.facts.edges[*index] {
                ProjectEdge::Relationship(edge) => Some(edge),
                _ => None,
            })
            .collect()
    }

    /// Return repository-reference edges that target the requested fact.
    pub fn repository_references_to(&self, target_id: &FactId) -> Vec<&RepositoryReferenceEdge> {
        self.repository_references_by_target
            .get(target_id)
            .into_iter()
            .flat_map(|indexes| indexes.iter())
            .filter_map(|index| match &self.facts.edges[*index] {
                ProjectEdge::RepositoryReference(edge) => Some(edge),
                _ => None,
            })
            .collect()
    }

    /// Return repository-reference edges from the requested source path.
    pub fn repository_references_from_path(
        &self,
        source_path: impl AsRef<Path>,
    ) -> Vec<&RepositoryReferenceEdge> {
        self.repository_references_by_source_path
            .get(source_path.as_ref())
            .into_iter()
            .flat_map(|indexes| indexes.iter())
            .filter_map(|index| match &self.facts.edges[*index] {
                ProjectEdge::RepositoryReference(edge) => Some(edge),
                _ => None,
            })
            .collect()
    }

    /// Return all repository-reference edges in stable store order.
    pub fn repository_references(&self) -> Vec<&RepositoryReferenceEdge> {
        self.facts
            .edges
            .iter()
            .filter_map(|edge| match edge {
                ProjectEdge::RepositoryReference(edge) => Some(edge),
                _ => None,
            })
            .collect()
    }

    /// Return repository-reference edges whose target path did not exist at ingestion time.
    pub fn unresolved_repository_references(&self) -> Vec<&RepositoryReferenceEdge> {
        self.repository_references()
            .into_iter()
            .filter(|edge| !edge.target_exists)
            .collect()
    }

    /// Return path scopes whose glob pattern matches a repository-relative path.
    pub fn path_scopes_for_path(&self, path: impl AsRef<Path>) -> Vec<&PathScope> {
        let normalized = normalize_path(path.as_ref());
        self.path_scopes
            .iter()
            .filter(|scope| {
                Pattern::new(&normalize_pattern(&scope.pattern))
                    .map(|pattern| pattern.matches(&normalized))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Return all code-symbol facts in stable store order.
    pub fn code_symbols(&self) -> Vec<&CodeSymbol> {
        self.code_symbols.iter().collect()
    }

    /// Return all code-symbol reference edges in stable store order.
    pub fn symbol_refs(&self) -> Vec<&SymbolRef> {
        self.symbol_refs.iter().collect()
    }

    /// Return search chunks that contain all query terms case-insensitively.
    pub fn keyword_search(&self, query: &str) -> Vec<&SearchChunk> {
        let terms = query
            .split_whitespace()
            .map(|term| term.to_ascii_lowercase())
            .filter(|term| !term.is_empty())
            .collect::<Vec<_>>();
        if terms.is_empty() {
            return Vec::new();
        }
        self.search_chunks
            .iter()
            .filter(|chunk| {
                let text = chunk.text.to_ascii_lowercase();
                terms.iter().all(|term| text.contains(term))
            })
            .collect()
    }

    /// Return embedding-backed candidate chunks for a query vector.
    pub fn semantic_search(
        &self,
        query_vector: &[f32],
        provider: &str,
        limit: usize,
    ) -> Vec<SemanticSearchHit<'_>> {
        if limit == 0 || query_vector.is_empty() {
            return Vec::new();
        }

        let mut hits = self
            .embedding_records
            .iter()
            .filter(|record| {
                record.provider == provider
                    && record.dimensions == query_vector.len()
                    && record.vector.len() == query_vector.len()
            })
            .filter_map(|record| {
                let chunk = self.search_chunk_for_record(record)?;
                if record.text_hash != semantic_text_hash(&chunk.text) {
                    return None;
                }
                let score = cosine_similarity(query_vector, &record.vector);
                if score <= 0.0 {
                    return None;
                }
                Some(SemanticSearchHit {
                    chunk,
                    embedding: record,
                    score,
                })
            })
            .collect::<Vec<_>>();
        hits.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.chunk.id.cmp(&right.chunk.id))
        });
        hits.truncate(limit);
        hits
    }

    /// Return measured index sizes for comparison and decision records.
    pub fn stats(&self) -> FactStoreStats {
        FactStoreStats {
            fact_count: self.facts.facts.len(),
            edge_count: self.facts.edges.len(),
            indexed_fact_ids: self.facts_by_id.len(),
            indexed_edge_ids: self.edges_by_id.len(),
            source_index_entries: self.edges_by_source.len(),
            repository_reference_target_count: self.repository_references_by_target.len(),
            search_chunk_count: self.search_chunks.len(),
            embedding_record_count: self.embedding_records.len(),
            path_scope_count: self.path_scopes.len(),
            serialized_bytes: serde_json::to_vec(&self.facts)
                .map(|bytes| bytes.len())
                .unwrap_or_default(),
        }
    }

    fn rebuild_indexes(&mut self) {
        self.facts_by_id.clear();
        self.edges_by_id.clear();
        self.edges_by_source.clear();
        self.repository_references_by_source_path.clear();
        self.repository_references_by_target.clear();
        self.missing_relationship_edges.clear();
        self.search_chunks.clear();
        self.embedding_records.clear();
        self.code_symbols.clear();
        self.symbol_refs.clear();
        self.path_scopes.clear();

        for (index, fact) in self.facts.facts.iter().enumerate() {
            self.facts_by_id
                .entry(fact.id().clone())
                .or_default()
                .push(index);
            match fact {
                ProjectFact::SearchChunk(chunk) => self.search_chunks.push(chunk.clone()),
                ProjectFact::EmbeddingRecord(record) => self.embedding_records.push(record.clone()),
                ProjectFact::CodeSymbol(symbol) => self.code_symbols.push(symbol.clone()),
                ProjectFact::PathScope(scope) => self.path_scopes.push(scope.clone()),
                _ => {}
            }
        }

        for (index, edge) in self.facts.edges.iter().enumerate() {
            self.edges_by_id
                .entry(edge.id().clone())
                .or_default()
                .push(index);
            match edge {
                ProjectEdge::Relationship(edge) => {
                    self.edges_by_source
                        .entry(edge.source_id.clone())
                        .or_default()
                        .push(index);
                    if self.relationship_target_missing(edge) {
                        self.missing_relationship_edges.push(index);
                    }
                }
                ProjectEdge::RepositoryReference(edge) => {
                    self.edges_by_source
                        .entry(edge.source_id.clone())
                        .or_default()
                        .push(index);
                    self.repository_references_by_source_path
                        .entry(edge.source_path.clone())
                        .or_default()
                        .push(index);
                    if let Some(target_id) = &edge.target_id {
                        self.repository_references_by_target
                            .entry(target_id.clone())
                            .or_default()
                            .push(index);
                    }
                }
                ProjectEdge::SymbolRef(edge) => {
                    self.edges_by_source
                        .entry(edge.source_id.clone())
                        .or_default()
                        .push(index);
                    self.symbol_refs.push(edge.clone());
                }
            }
        }
    }

    fn relationship_target_missing(&self, edge: &RelationshipEdge) -> bool {
        match &edge.target_id {
            Some(target_id) => self
                .facts_by_id
                .get(target_id)
                .map(|indexes| {
                    indexes
                        .iter()
                        .filter(|index| {
                            self.facts.facts[**index].generation().id == edge.generation.id
                        })
                        .count()
                        != 1
                })
                .unwrap_or(true),
            None => true,
        }
    }

    fn search_chunk_for_record(&self, record: &EmbeddingRecord) -> Option<&SearchChunk> {
        self.search_chunks.iter().find(|chunk| {
            chunk.id == record.chunk_id && chunk.generation.id == record.generation.id
        })
    }
}

/// Semantic-search candidate returned by the in-memory fact store.
#[derive(Debug, Clone, Copy)]
pub struct SemanticSearchHit<'a> {
    /// Source text chunk matched by the embedding record.
    pub chunk: &'a SearchChunk,
    /// Embedding metadata used for candidate retrieval.
    pub embedding: &'a EmbeddingRecord,
    /// Cosine similarity score; it is candidate ranking, not validation truth.
    pub score: f32,
}

/// Measured size and index cardinality for a loaded fact store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactStoreStats {
    /// Number of facts retained in the store.
    pub fact_count: usize,
    /// Number of edges retained in the store.
    pub edge_count: usize,
    /// Number of unique fact IDs indexed.
    pub indexed_fact_ids: usize,
    /// Number of unique edge IDs indexed.
    pub indexed_edge_ids: usize,
    /// Number of source IDs with outbound edges.
    pub source_index_entries: usize,
    /// Number of target IDs with inbound repository-reference edges.
    pub repository_reference_target_count: usize,
    /// Number of searchable text chunks indexed.
    pub search_chunk_count: usize,
    /// Number of embedding records indexed.
    pub embedding_record_count: usize,
    /// Number of path scopes indexed.
    pub path_scope_count: usize,
    /// Serialized byte footprint of the retained fact set.
    pub serialized_bytes: usize,
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalize_pattern(pattern: &str) -> String {
    pattern.replace('\\', "/")
}
