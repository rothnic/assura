//! Dependency graph intelligence modules and public re-exports.
pub mod error;
pub mod facts;
pub mod graph;
pub mod node;
pub mod persistence;
pub mod query;

pub use error::{GraphError, GraphResult};
pub use facts::{
    model_instance_id, resource_id, CodeSymbol, Diagnostic, EdgeId, EmbeddingRecord,
    FactGeneration, FactId, FactIngestor, FactOrigin, FactSet, FieldDefinition, MarkdownDocument,
    MarkdownSection, ModelDefinition, ModelInstance, PathScope, ProjectEdge, ProjectFact,
    RelationshipDefinition, RelationshipEdge, Resource, SafeFix, SearchChunk, SourceLocation,
    SymbolRef,
};
pub use graph::{GraphBuilder, GraphStats, IntelligenceGraph};
pub use node::{DirectoryNode, Edge, FileNode, Node, NodeId, NodeMetadata, NodeType, Relationship};
pub use persistence::{GraphPersistence, PersistenceFormat};
pub use query::{GraphQuery, QueryResult};
