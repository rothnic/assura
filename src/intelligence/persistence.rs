use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::{GraphError, GraphResult};
use super::graph::IntelligenceGraph;
use super::node::{Edge, GraphSchema, Node, NodeId};

#[derive(Debug, Serialize, Deserialize)]
struct GraphData {
    schema: GraphSchema,
    nodes: Vec<Node>,
    edges: Vec<SerializedEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SerializedEdge {
    source: u64,
    target: u64,
    relationship: String,
    weight: f64,
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct GraphPersistence {
    storage_dir: PathBuf,
    format: PersistenceFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceFormat {
    Json,
    Yaml,
    Binary,
}

impl GraphPersistence {
    pub fn new<P: AsRef<Path>>(storage_dir: P) -> GraphResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();

        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir).map_err(GraphError::Io)?;
        }

        Ok(Self {
            storage_dir,
            format: PersistenceFormat::Json,
        })
    }

    pub fn with_format(mut self, format: PersistenceFormat) -> Self {
        self.format = format;
        self
    }

    pub fn save(&self, graph: &IntelligenceGraph, name: &str) -> GraphResult<PathBuf> {
        let data = self.graph_to_data(graph)?;
        let path = self
            .storage_dir
            .join(format!("{}.{}", name, self.format.extension()));

        match self.format {
            PersistenceFormat::Json => {
                let json =
                    serde_json::to_string_pretty(&data).map_err(GraphError::Serialization)?;
                fs::write(&path, json).map_err(GraphError::Io)?;
            }
            PersistenceFormat::Yaml => {
                let yaml = serde_yaml::to_string(&data)
                    .map_err(|e| GraphError::PersistenceError(e.to_string()))?;
                fs::write(&path, yaml).map_err(GraphError::Io)?;
            }
            PersistenceFormat::Binary => {
                let bytes = bincode::serialize(&data)
                    .map_err(|e| GraphError::PersistenceError(e.to_string()))?;
                fs::write(&path, bytes).map_err(GraphError::Io)?;
            }
        }

        Ok(path)
    }

    pub fn load(&self, name: &str) -> GraphResult<IntelligenceGraph> {
        let path = self
            .storage_dir
            .join(format!("{}.{}", name, self.format.extension()));

        if !path.exists() {
            return Err(GraphError::PersistenceError(format!(
                "Graph file not found: {}",
                path.display()
            )));
        }

        let content = fs::read(&path).map_err(GraphError::Io)?;

        let data: GraphData = match self.format {
            PersistenceFormat::Json => {
                serde_json::from_slice(&content).map_err(GraphError::Serialization)?
            }
            PersistenceFormat::Yaml => serde_yaml::from_slice(&content)
                .map_err(|e| GraphError::PersistenceError(e.to_string()))?,
            PersistenceFormat::Binary => bincode::deserialize(&content)
                .map_err(|e| GraphError::PersistenceError(e.to_string()))?,
        };

        self.data_to_graph(data)
    }

    pub fn update(&self, graph: &mut IntelligenceGraph, name: &str) -> GraphResult<PathBuf> {
        let existing = self.load(name);

        if let Ok(existing_graph) = existing {
            self.merge_graphs(graph, existing_graph)?;
        }

        self.save(graph, name)
    }

    pub fn list_saved_graphs(&self) -> GraphResult<Vec<String>> {
        let mut graphs = Vec::new();

        for entry in fs::read_dir(&self.storage_dir).map_err(GraphError::Io)? {
            let entry = entry.map_err(GraphError::Io)?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == self.format.extension() {
                    if let Some(stem) = path.file_stem() {
                        graphs.push(stem.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(graphs)
    }

    pub fn delete(&self, name: &str) -> GraphResult<()> {
        let path = self
            .storage_dir
            .join(format!("{}.{}", name, self.format.extension()));

        if path.exists() {
            fs::remove_file(&path).map_err(GraphError::Io)?;
        }

        Ok(())
    }

    fn graph_to_data(&self, graph: &IntelligenceGraph) -> GraphResult<GraphData> {
        let mut nodes = Vec::new();
        for node in graph.nodes() {
            nodes.push(node.clone());
        }

        let mut edges = Vec::new();
        for (source_id, target_id, rel) in graph.iter_edges() {
            edges.push(SerializedEdge {
                source: source_id.as_u64(),
                target: target_id.as_u64(),
                relationship: rel.to_string(),
                weight: 1.0,
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(GraphData {
            schema: GraphSchema::new(),
            nodes,
            edges,
        })
    }

    fn data_to_graph(&self, data: GraphData) -> GraphResult<IntelligenceGraph> {
        let mut graph = IntelligenceGraph::new();
        let mut id_mapping: std::collections::HashMap<u64, NodeId> =
            std::collections::HashMap::new();

        for node in data.nodes {
            let old_id = node.id().as_u64();
            let new_id = graph.add_node(node);
            id_mapping.insert(old_id, new_id);
        }

        for edge in data.edges {
            let source = *id_mapping.get(&edge.source).ok_or_else(|| {
                GraphError::PersistenceError(format!("Unknown source node: {}", edge.source))
            })?;

            let target = *id_mapping.get(&edge.target).ok_or_else(|| {
                GraphError::PersistenceError(format!("Unknown target node: {}", edge.target))
            })?;

            let relationship = match edge.relationship.as_str() {
                "contains" => super::node::Relationship::Contains,
                "depends_on" => super::node::Relationship::DependsOn,
                "references" => super::node::Relationship::References,
                "imports" => super::node::Relationship::Imports,
                "exports" => super::node::Relationship::Exports,
                "extends" => super::node::Relationship::Extends,
                "implements" => super::node::Relationship::Implements,
                _ => super::node::Relationship::References,
            };

            let edge_obj = Edge::new(source, target, relationship).with_weight(edge.weight);
            graph.add_edge(edge_obj)?;
        }

        Ok(graph)
    }

    fn merge_graphs(
        &self,
        target: &mut IntelligenceGraph,
        source: IntelligenceGraph,
    ) -> GraphResult<()> {
        let path_to_node: std::collections::HashMap<_, _> = target
            .iter_nodes()
            .map(|(id, node)| (node.path().to_path_buf(), id))
            .collect();

        for (_source_id, source_node) in source.iter_nodes() {
            let path = source_node.path();

            if let Some(&existing_id) = path_to_node.get(path) {
                let metadata = source_node.metadata().clone();
                if let Some(node) = target.get_node_mut(existing_id) {
                    *node.metadata_mut() = metadata;
                }
            } else {
                let new_node = source_node.clone();
                target.add_node(new_node);
            }
        }

        Ok(())
    }
}

impl PersistenceFormat {
    fn extension(&self) -> &'static str {
        match self {
            PersistenceFormat::Json => "json",
            PersistenceFormat::Yaml => "yaml",
            PersistenceFormat::Binary => "bin",
        }
    }
}
