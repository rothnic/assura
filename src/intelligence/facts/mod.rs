//! Storage-independent project intelligence fact contract.
//!
//! This module defines stable facts later graph, search, semantic, code-symbol,
//! daemon, LSP, and MCP layers can consume without choosing a database.

mod code_symbols;
mod ingest;
mod ingest_helpers;
mod markdown_link_ingest;
mod markdown_links;
mod repository_references;
mod set;
mod types;

pub use code_symbols::{CodeProviderEvidence, CodeSymbol, SymbolRef};
pub use ingest::FactIngestor;
pub use markdown_links::MarkdownLink;
pub use repository_references::RepositoryReferenceEdge;
pub use set::{model_definition_id, model_instance_id, resource_id, FactSet};
pub use types::{
    Diagnostic, EdgeId, EmbeddingRecord, FactGeneration, FactId, FactOrigin, FieldDefinition,
    MarkdownDocument, MarkdownSection, ModelDefinition, ModelInstance, PathScope, ProjectEdge,
    ProjectFact, RelationshipDefinition, RelationshipEdge, Resource, SafeFix, SearchChunk,
    SourceLocation,
};
