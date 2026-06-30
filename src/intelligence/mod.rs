//! Dependency graph intelligence modules and public re-exports.
pub mod agent_surface;
pub mod error;
pub mod facts;
pub mod graph;
pub mod node;
pub mod persistence;
pub mod query;
pub mod semantic;
pub mod store;

pub use agent_surface::{
    project_intelligence_agent_context, AgentSurfaceCapability, AgentSurfaceSummary,
    ProjectIntelligenceAgentContext,
};
pub use error::{GraphError, GraphResult};
pub use facts::{
    model_instance_id, resource_id, CodeProviderEvidence, CodeSymbol, Diagnostic, EdgeId,
    EmbeddingRecord, FactGeneration, FactId, FactIngestor, FactOrigin, FactSet, FieldDefinition,
    MarkdownDocument, MarkdownSection, ModelDefinition, ModelInstance, PathScope, ProjectEdge,
    ProjectFact, RelationshipDefinition, RelationshipEdge, Resource, SafeFix, SearchChunk,
    SourceLocation, SymbolRef,
};
pub use graph::{GraphBuilder, GraphStats, IntelligenceGraph};
pub use node::{DirectoryNode, Edge, FileNode, Node, NodeId, NodeMetadata, NodeType, Relationship};
pub use persistence::{GraphPersistence, PersistenceFormat};
pub use query::{GraphQuery, QueryResult};
pub use semantic::{
    cosine_similarity, local_hash_embedding, local_hash_embedding_record, semantic_text_hash,
    LOCAL_HASH_EMBEDDING_DIMENSIONS, LOCAL_HASH_EMBEDDING_PROVIDER,
};
pub use store::{FactStoreStats, InMemoryFactStore, SemanticSearchHit};
