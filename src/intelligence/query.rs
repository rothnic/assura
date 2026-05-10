use std::collections::HashSet;
use std::path::Path;

use super::error::{GraphError, GraphResult};
use super::graph::IntelligenceGraph;
use super::node::{Node, NodeId, NodeType, Relationship};

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub nodes: Vec<Node>,
    pub total_count: usize,
}

impl QueryResult {
    pub fn new(nodes: Vec<Node>) -> Self {
        let count = nodes.len();
        Self {
            nodes,
            total_count: count,
        }
    }

    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            total_count: 0,
        }
    }

    pub fn first(&self) -> Option<&Node> {
        self.nodes.first()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

pub struct GraphQuery<'a> {
    graph: &'a IntelligenceGraph,
}

impl<'a> GraphQuery<'a> {
    pub fn new(graph: &'a IntelligenceGraph) -> Self {
        Self { graph }
    }

    pub fn find_by_id(&self, id: NodeId) -> Option<&Node> {
        self.graph.get_node(id)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<&Node> {
        self.graph.get_node_by_path(path)
    }

    pub fn find_by_name(&self, name: &str) -> QueryResult {
        let nodes: Vec<Node> = self
            .graph
            .nodes()
            .filter(|node| node.name() == name)
            .cloned()
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_by_type(&self, node_type: NodeType) -> QueryResult {
        let nodes: Vec<Node> = self
            .graph
            .nodes()
            .filter(|node| node.node_type() == node_type)
            .cloned()
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_by_extension(&self, extension: &str) -> QueryResult {
        let nodes: Vec<Node> = self
            .graph
            .nodes()
            .filter(|node| {
                if let Node::File(file) = node {
                    file.extension.as_ref().map(|e| e.as_str()) == Some(extension)
                } else {
                    false
                }
            })
            .cloned()
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_children(&self, parent_id: NodeId) -> QueryResult {
        let edges = self.graph.outgoing_edges(parent_id);
        let nodes: Vec<Node> = edges
            .into_iter()
            .filter(|(_, rel)| *rel == Relationship::Contains)
            .filter_map(|(id, _)| self.graph.get_node(id).cloned())
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_parent(&self, child_id: NodeId) -> QueryResult {
        let edges = self.graph.incoming_edges(child_id);
        let nodes: Vec<Node> = edges
            .into_iter()
            .filter(|(_, rel)| *rel == Relationship::Contains)
            .filter_map(|(id, _)| self.graph.get_node(id).cloned())
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_related(&self, node_id: NodeId, relationship: Relationship) -> QueryResult {
        let edges = self.graph.outgoing_edges(node_id);
        let nodes: Vec<Node> = edges
            .into_iter()
            .filter(|(_, rel)| *rel == relationship)
            .filter_map(|(id, _)| self.graph.get_node(id).cloned())
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_referring_to(&self, node_id: NodeId, relationship: Relationship) -> QueryResult {
        let edges = self.graph.incoming_edges(node_id);
        let nodes: Vec<Node> = edges
            .into_iter()
            .filter(|(_, rel)| *rel == relationship)
            .filter_map(|(id, _)| self.graph.get_node(id).cloned())
            .collect();
        QueryResult::new(nodes)
    }

    pub fn find_path_between(&self, from: NodeId, to: NodeId) -> GraphResult<Vec<Node>> {
        let path_ids = self
            .graph
            .find_path(from, to)
            .ok_or_else(|| GraphError::TraversalError("No path found".to_string()))?;

        let nodes: Vec<Node> = path_ids
            .into_iter()
            .filter_map(|id| self.graph.get_node(id).cloned())
            .collect();

        Ok(nodes)
    }

    pub fn find_common_ancestors(&self, nodes: &[NodeId]) -> QueryResult {
        if nodes.is_empty() {
            return QueryResult::empty();
        }

        let mut common_ancestors: HashSet<NodeId> = HashSet::new();
        let mut first = true;

        for node_id in nodes {
            let ancestors = self.collect_ancestors(*node_id);

            if first {
                common_ancestors = ancestors;
                first = false;
            } else {
                common_ancestors = common_ancestors.intersection(&ancestors).copied().collect();
            }
        }

        let nodes: Vec<Node> = common_ancestors
            .into_iter()
            .filter_map(|id| self.graph.get_node(id).cloned())
            .collect();

        QueryResult::new(nodes)
    }

    pub fn find_all_descendants(&self, root_id: NodeId) -> QueryResult {
        let mut visited = HashSet::new();
        let mut descendants = Vec::new();
        let mut stack = vec![root_id];

        while let Some(current) = stack.pop() {
            if visited.insert(current) {
                if let Some(node) = self.graph.get_node(current) {
                    if current != root_id {
                        descendants.push(node.clone());
                    }

                    let children = self.graph.outgoing_edges(current);
                    for (child_id, _) in children {
                        if !visited.contains(&child_id) {
                            stack.push(child_id);
                        }
                    }
                }
            }
        }

        QueryResult::new(descendants)
    }

    pub fn search_by_pattern(&self, pattern: &str) -> QueryResult {
        let pattern_lower = pattern.to_lowercase();
        let nodes: Vec<Node> = self
            .graph
            .nodes()
            .filter(|node| {
                let name_lower = node.name().to_lowercase();
                let path_lower = node.path().to_string_lossy().to_lowercase();
                name_lower.contains(&pattern_lower) || path_lower.contains(&pattern_lower)
            })
            .cloned()
            .collect();
        QueryResult::new(nodes)
    }

    pub fn resolve_path(&self, path: &Path) -> GraphResult<NodeId> {
        self.graph
            .get_node_by_path(path)
            .map(|n| n.id())
            .ok_or_else(|| GraphError::InvalidPath(path.to_string_lossy().to_string()))
    }

    pub fn get_graph(&self) -> &'a IntelligenceGraph {
        self.graph
    }

    fn collect_ancestors(&self, node_id: NodeId) -> HashSet<NodeId> {
        let mut ancestors = HashSet::new();
        let mut stack = vec![node_id];

        while let Some(current) = stack.pop() {
            let edges = self.graph.incoming_edges(current);
            for (parent_id, rel) in edges {
                if rel == Relationship::Contains && ancestors.insert(parent_id) {
                    stack.push(parent_id);
                }
            }
        }

        ancestors
    }
}

pub struct PathResolver;

impl PathResolver {
    pub fn resolve_relative(base: &Path, relative: &Path) -> Option<std::path::PathBuf> {
        if relative.is_absolute() {
            Some(relative.to_path_buf())
        } else {
            base.parent().map(|parent| parent.join(relative))
        }
    }

    pub fn normalize_path(path: &Path) -> std::path::PathBuf {
        use std::path::Component;

        let mut result = std::path::PathBuf::new();

        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    result.push(component);
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if !result.as_os_str().is_empty() {
                        result.pop();
                    }
                }
                Component::Normal(c) => {
                    result.push(c);
                }
            }
        }

        result
    }
}
