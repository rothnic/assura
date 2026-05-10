use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn from_raw(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "node_{}", self.0)
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    File,
    Directory,
    Symbol,
    Package,
    Module,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::File => write!(f, "file"),
            NodeType::Directory => write!(f, "directory"),
            NodeType::Symbol => write!(f, "symbol"),
            NodeType::Package => write!(f, "package"),
            NodeType::Module => write!(f, "module"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub created_at: u64,
    pub modified_at: u64,
    pub size: u64,
    pub custom: HashMap<String, String>,
}

impl NodeMetadata {
    pub fn new() -> Self {
        Self {
            created_at: 0,
            modified_at: 0,
            size: 0,
            custom: HashMap::new(),
        }
    }

    pub fn with_timestamp(created: u64, modified: u64) -> Self {
        Self {
            created_at: created,
            modified_at: modified,
            size: 0,
            custom: HashMap::new(),
        }
    }

    pub fn with_custom<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

impl Default for NodeMetadata {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub id: NodeId,
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub metadata: NodeMetadata,
    pub content_hash: Option<String>,
}

impl FileNode {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let extension = path.extension().map(|e| e.to_string_lossy().to_string());

        Self {
            id: NodeId::new(),
            path,
            name,
            extension,
            metadata: NodeMetadata::new(),
            content_hash: None,
        }
    }

    pub fn with_metadata(mut self, metadata: NodeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_content_hash<S: Into<String>>(mut self, hash: S) -> Self {
        self.content_hash = Some(hash.into());
        self
    }

    pub fn node_type() -> NodeType {
        NodeType::File
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub id: NodeId,
    pub path: PathBuf,
    pub name: String,
    pub metadata: NodeMetadata,
    pub child_count: usize,
}

impl DirectoryNode {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref().to_path_buf();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        Self {
            id: NodeId::new(),
            path,
            name,
            metadata: NodeMetadata::new(),
            child_count: 0,
        }
    }

    pub fn with_metadata(mut self, metadata: NodeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_child_count(mut self, count: usize) -> Self {
        self.child_count = count;
        self
    }

    pub fn node_type() -> NodeType {
        NodeType::Directory
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relationship {
    Contains,
    DependsOn,
    References,
    Imports,
    Exports,
    Extends,
    Implements,
}

impl fmt::Display for Relationship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relationship::Contains => write!(f, "contains"),
            Relationship::DependsOn => write!(f, "depends_on"),
            Relationship::References => write!(f, "references"),
            Relationship::Imports => write!(f, "imports"),
            Relationship::Exports => write!(f, "exports"),
            Relationship::Extends => write!(f, "extends"),
            Relationship::Implements => write!(f, "implements"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub relationship: Relationship,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

impl Edge {
    pub fn new(source: NodeId, target: NodeId, relationship: Relationship) -> Self {
        Self {
            source,
            target,
            relationship,
            weight: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Node {
    File(FileNode),
    Directory(DirectoryNode),
}

impl Node {
    pub fn id(&self) -> NodeId {
        match self {
            Node::File(f) => f.id,
            Node::Directory(d) => d.id,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Node::File(f) => &f.path,
            Node::Directory(d) => &d.path,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::File(f) => &f.name,
            Node::Directory(d) => &d.name,
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            Node::File(_) => NodeType::File,
            Node::Directory(_) => NodeType::Directory,
        }
    }

    pub fn metadata(&self) -> &NodeMetadata {
        match self {
            Node::File(f) => &f.metadata,
            Node::Directory(d) => &d.metadata,
        }
    }

    pub fn metadata_mut(&mut self) -> &mut NodeMetadata {
        match self {
            Node::File(f) => &mut f.metadata,
            Node::Directory(d) => &mut d.metadata,
        }
    }
}

impl From<FileNode> for Node {
    fn from(f: FileNode) -> Self {
        Node::File(f)
    }
}

impl From<DirectoryNode> for Node {
    fn from(d: DirectoryNode) -> Self {
        Node::Directory(d)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSchema {
    pub version: String,
    pub description: String,
    pub node_types: Vec<NodeType>,
    pub edge_types: Vec<Relationship>,
}

impl GraphSchema {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Assura intelligence graph schema".to_string(),
            node_types: vec![NodeType::File, NodeType::Directory],
            edge_types: vec![
                Relationship::Contains,
                Relationship::DependsOn,
                Relationship::References,
            ],
        }
    }
}

impl Default for GraphSchema {
    fn default() -> Self {
        Self::new()
    }
}
