use std::collections::HashMap;
use std::path::{Path, PathBuf};

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use walkdir::WalkDir;

use super::error::{GraphError, GraphResult};
use super::node::{DirectoryNode, Edge, FileNode, Node, NodeId, Relationship};

#[derive(Debug)]
pub struct IntelligenceGraph {
    graph: DiGraph<Node, Relationship>,
    node_indices: HashMap<NodeId, NodeIndex>,
    path_to_node: HashMap<PathBuf, NodeId>,
}

impl IntelligenceGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            path_to_node: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let node_id = node.id();
        let path = node.path().to_path_buf();

        let idx = self.graph.add_node(node);
        self.node_indices.insert(node_id, idx);
        self.path_to_node.insert(path, node_id);

        node_id
    }

    pub fn add_edge(&mut self, edge: Edge) -> GraphResult<()> {
        let source_idx = self
            .node_indices
            .get(&edge.source)
            .copied()
            .ok_or_else(|| GraphError::NodeNotFound(format!("{}", edge.source)))?;

        let target_idx = self
            .node_indices
            .get(&edge.target)
            .copied()
            .ok_or_else(|| GraphError::NodeNotFound(format!("{}", edge.target)))?;

        self.graph
            .add_edge(source_idx, target_idx, edge.relationship);
        Ok(())
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.node_indices
            .get(&id)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.node_indices
            .get(&id)
            .copied()
            .and_then(move |idx| self.graph.node_weight_mut(idx))
    }

    pub fn get_node_by_path(&self, path: &Path) -> Option<&Node> {
        self.path_to_node
            .get(path)
            .and_then(|&id| self.get_node(id))
    }

    pub fn get_node_by_path_mut(&mut self, path: &Path) -> Option<&mut Node> {
        self.path_to_node
            .get(path)
            .copied()
            .and_then(move |id| self.get_node_mut(id))
    }

    pub fn contains_node(&self, id: NodeId) -> bool {
        self.node_indices.contains_key(&id)
    }

    pub fn contains_path(&self, path: &Path) -> bool {
        self.path_to_node.contains_key(path)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.graph.node_weights()
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeId, &Node)> {
        self.node_indices
            .iter()
            .filter_map(move |(id, &idx)| self.graph.node_weight(idx).map(|node| (*id, node)))
    }

    pub fn iter_edges(&self) -> impl Iterator<Item = (&NodeId, &NodeId, &Relationship)> {
        self.graph.edge_references().filter_map(move |edge| {
            let source_idx = edge.source();
            let target_idx = edge.target();
            let rel = edge.weight();

            let _source_id = self.graph.node_weight(source_idx).map(|n| n.id())?;
            let _target_id = self.graph.node_weight(target_idx).map(|n| n.id())?;

            Some((
                self.node_indices
                    .iter()
                    .find(|(_, &idx)| idx == source_idx)
                    .map(|(id, _)| id)?,
                self.node_indices
                    .iter()
                    .find(|(_, &idx)| idx == target_idx)
                    .map(|(id, _)| id)?,
                rel,
            ))
        })
    }

    pub fn outgoing_edges(&self, id: NodeId) -> Vec<(NodeId, Relationship)> {
        let idx = match self.node_indices.get(&id) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        self.graph
            .edges(idx)
            .filter_map(|edge| {
                let target_idx = edge.target();
                self.graph
                    .node_weight(target_idx)
                    .map(|node| (node.id(), *edge.weight()))
            })
            .collect()
    }

    pub fn incoming_edges(&self, id: NodeId) -> Vec<(NodeId, Relationship)> {
        let idx = match self.node_indices.get(&id) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        self.graph
            .edges_directed(idx, petgraph::Direction::Incoming)
            .filter_map(|edge| {
                let source_idx = edge.source();
                self.graph
                    .node_weight(source_idx)
                    .map(|node| (node.id(), *edge.weight()))
            })
            .collect()
    }

    pub fn get_neighbors(&self, id: NodeId) -> Vec<NodeId> {
        let idx = match self.node_indices.get(&id) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        let mut neighbors = Vec::new();

        for edge in self.graph.edges(idx) {
            if let Some(node) = self.graph.node_weight(edge.target()) {
                neighbors.push(node.id());
            }
        }

        for edge in self
            .graph
            .edges_directed(idx, petgraph::Direction::Incoming)
        {
            if let Some(node) = self.graph.node_weight(edge.source()) {
                if !neighbors.contains(&node.id()) {
                    neighbors.push(node.id());
                }
            }
        }

        neighbors
    }

    pub fn find_path(&self, from: NodeId, to: NodeId) -> Option<Vec<NodeId>> {
        let from_idx = self.node_indices.get(&from).copied()?;
        let to_idx = self.node_indices.get(&to).copied()?;

        let path = petgraph::algo::dijkstra(&self.graph, from_idx, Some(to_idx), |_| 1u32);

        if path.contains_key(&to_idx) {
            let mut result = Vec::new();
            let mut current = to_idx;

            while current != from_idx {
                result.push(self.graph.node_weight(current).map(|n| n.id())?);

                let neighbors: Vec<_> = self
                    .graph
                    .edges_directed(current, petgraph::Direction::Incoming)
                    .collect();

                let mut found = false;
                for edge in neighbors {
                    let source = edge.source();
                    if path.contains_key(&source) && path[&source] < path[&current] {
                        current = source;
                        found = true;
                        break;
                    }
                }

                if !found {
                    return None;
                }
            }

            result.push(from);
            result.reverse();
            Some(result)
        } else {
            None
        }
    }

    pub fn remove_node(&mut self, id: NodeId) -> GraphResult<Option<Node>> {
        let idx = match self.node_indices.remove(&id) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        if let Some(node) = self.graph.node_weight(idx) {
            self.path_to_node.remove(node.path());
        }

        Ok(self.graph.remove_node(idx))
    }

    pub fn clear(&mut self) {
        self.graph.clear();
        self.node_indices.clear();
        self.path_to_node.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    pub fn inner_graph(&self) -> &DiGraph<Node, Relationship> {
        &self.graph
    }

    pub fn node_indices_map(&self) -> &HashMap<NodeId, NodeIndex> {
        &self.node_indices
    }
}

impl Default for IntelligenceGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GraphBuilder {
    root_path: PathBuf,
    follow_symlinks: bool,
    max_depth: Option<usize>,
}

impl GraphBuilder {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root_path: root.as_ref().to_path_buf(),
            follow_symlinks: false,
            max_depth: None,
        }
    }

    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn build(self) -> GraphResult<IntelligenceGraph> {
        let mut graph = IntelligenceGraph::new();
        let mut dir_entries: Vec<_> = Vec::new();
        let mut file_entries: Vec<_> = Vec::new();

        let walker = WalkDir::new(&self.root_path).follow_links(self.follow_symlinks);

        let walker = if let Some(depth) = self.max_depth {
            walker.max_depth(depth)
        } else {
            walker
        };

        for entry in walker {
            let entry = entry.map_err(|e| GraphError::WalkDir(e.to_string()))?;
            let path = entry.path().to_path_buf();
            let metadata = entry
                .metadata()
                .map_err(|e| GraphError::WalkDir(e.to_string()))?;

            if metadata.is_dir() {
                dir_entries.push((path, metadata));
            } else if metadata.is_file() {
                file_entries.push((path, metadata));
            }
        }

        let mut dir_nodes: HashMap<PathBuf, NodeId> = HashMap::new();
        for (path, metadata) in dir_entries {
            let dir_node =
                DirectoryNode::new(&path).with_metadata(super::node::NodeMetadata::with_timestamp(
                    metadata
                        .created()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                    metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                ));

            let node_id = graph.add_node(dir_node.into());
            dir_nodes.insert(path, node_id);
        }

        let file_nodes: Vec<(PathBuf, NodeId, Node)> = file_entries
            .into_par_iter()
            .map(|(path, metadata)| {
                let file_node =
                    FileNode::new(&path).with_metadata(super::node::NodeMetadata::with_timestamp(
                        metadata
                            .created()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                        metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    ));

                let id = file_node.id;
                let node = Node::File(file_node);
                (path, id, node)
            })
            .collect();

        let mut file_node_map: HashMap<PathBuf, NodeId> = HashMap::new();
        for (path, id, node) in file_nodes {
            graph.add_node(node);
            file_node_map.insert(path, id);
        }

        for (path, dir_id) in &dir_nodes {
            if let Some(parent) = path.parent() {
                if let Some(&parent_id) = dir_nodes.get(parent) {
                    let edge = Edge::new(parent_id, *dir_id, Relationship::Contains);
                    let _ = graph.add_edge(edge);
                }
            }

            for (child_path, child_id) in &dir_nodes {
                if child_path.parent() == Some(path) && child_id != dir_id {
                    let edge = Edge::new(*dir_id, *child_id, Relationship::Contains);
                    let _ = graph.add_edge(edge);
                }
            }

            for (file_path, file_id) in &file_node_map {
                if file_path.parent() == Some(path) {
                    let edge = Edge::new(*dir_id, *file_id, Relationship::Contains);
                    let _ = graph.add_edge(edge);
                }
            }
        }

        Ok(graph)
    }

    pub fn build_parallel(self) -> GraphResult<IntelligenceGraph> {
        self.build()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub file_count: usize,
    pub directory_count: usize,
}

impl GraphStats {
    pub fn from_graph(graph: &IntelligenceGraph) -> Self {
        let mut file_count = 0;
        let mut directory_count = 0;

        for node in graph.nodes() {
            match node {
                Node::File(_) => file_count += 1,
                Node::Directory(_) => directory_count += 1,
            }
        }

        Self {
            node_count: graph.node_count(),
            edge_count: graph.edge_count(),
            file_count,
            directory_count,
        }
    }
}

use serde::{Deserialize, Serialize};
