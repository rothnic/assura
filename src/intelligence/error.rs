use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("WalkDir error: {0}")]
    WalkDir(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Edge not found: {0} -> {1}")]
    EdgeNotFound(String, String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Graph already contains node with id: {0}")]
    DuplicateNode(String),

    #[error("Graph traversal error: {0}")]
    TraversalError(String),

    #[error("Persistence error: {0}")]
    PersistenceError(String),

    #[error("Parallel execution error: {0}")]
    ParallelError(String),
}

pub type GraphResult<T> = Result<T, GraphError>;
