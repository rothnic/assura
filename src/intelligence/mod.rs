pub mod error;
pub mod graph;
pub mod node;
pub mod persistence;
pub mod query;

pub use error::{GraphError, GraphResult};
pub use graph::{GraphBuilder, GraphStats, IntelligenceGraph};
pub use node::{DirectoryNode, Edge, FileNode, Node, NodeId, NodeMetadata, NodeType, Relationship};
pub use persistence::{GraphPersistence, PersistenceFormat};
pub use query::{GraphQuery, QueryResult};
