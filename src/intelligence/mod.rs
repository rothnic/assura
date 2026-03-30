pub mod error;
pub mod graph;
pub mod node;
pub mod persistence;
pub mod query;

pub use error::{GraphError, GraphResult};
pub use graph::{IntelligenceGraph, GraphBuilder, GraphStats};
pub use node::{NodeId, Node, Edge, FileNode, DirectoryNode, Relationship, NodeType, NodeMetadata};
pub use query::{GraphQuery, QueryResult};
pub use persistence::{GraphPersistence, PersistenceFormat};
