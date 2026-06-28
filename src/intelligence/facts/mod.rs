//! Storage-independent project intelligence fact contract.
//!
//! This module defines stable facts later graph, search, semantic, code-symbol,
//! daemon, LSP, and MCP layers can consume without choosing a database.

mod ingest;
mod set;
mod types;

pub use ingest::FactIngestor;
pub use set::{model_definition_id, model_instance_id, resource_id, FactSet};
pub use types::{
    CodeSymbol, Diagnostic, EdgeId, EmbeddingRecord, FactGeneration, FactId, FactOrigin,
    FieldDefinition, MarkdownDocument, MarkdownSection, ModelDefinition, ModelInstance, PathScope,
    ProjectEdge, ProjectFact, RelationshipDefinition, RelationshipEdge, Resource, SafeFix,
    SearchChunk, SourceLocation, SymbolRef,
};
