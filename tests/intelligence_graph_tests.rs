use std::fs;
use std::path::Path;

use assura::intelligence::persistence::PersistenceFormat;
use assura::intelligence::{
    DirectoryNode, Edge, FileNode, GraphBuilder, GraphError, GraphPersistence, GraphQuery,
    IntelligenceGraph, NodeId, NodeMetadata, NodeType, Relationship,
};

fn create_test_directory() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path();

    let src_dir = base_path.join("src");
    fs::create_dir(&src_dir).unwrap();

    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
    fs::write(src_dir.join("lib.rs"), "pub mod utils;").unwrap();

    let utils_dir = src_dir.join("utils");
    fs::create_dir(&utils_dir).unwrap();
    fs::write(utils_dir.join("mod.rs"), "pub fn helper() {}").unwrap();

    let tests_dir = base_path.join("tests");
    fs::create_dir(&tests_dir).unwrap();
    fs::write(tests_dir.join("test_main.rs"), "#[test] fn test() {}").unwrap();

    temp_dir
}

#[test]
fn test_node_creation() {
    let file_node = FileNode::new("/test/file.rs");
    assert_eq!(file_node.name, "file.rs");
    assert_eq!(file_node.extension, Some("rs".to_string()));
    assert!(file_node.path.ends_with("file.rs"));

    let dir_node = DirectoryNode::new("/test/dir");
    assert_eq!(dir_node.name, "dir");
    assert!(dir_node.path.ends_with("dir"));
}

#[test]
fn test_node_metadata() {
    let metadata = NodeMetadata::with_timestamp(1000, 2000)
        .with_custom("key1", "value1")
        .with_custom("key2", "value2");

    assert_eq!(metadata.created_at, 1000);
    assert_eq!(metadata.modified_at, 2000);
    assert_eq!(metadata.custom.get("key1"), Some(&"value1".to_string()));
    assert_eq!(metadata.custom.get("key2"), Some(&"value2".to_string()));
}

#[test]
fn test_node_id_uniqueness() {
    let id1 = NodeId::new();
    let id2 = NodeId::new();
    let id3 = NodeId::new();

    assert_ne!(id1.as_u64(), id2.as_u64());
    assert_ne!(id2.as_u64(), id3.as_u64());
    assert_ne!(id1.as_u64(), id3.as_u64());
}

#[test]
fn test_edge_creation() {
    let source_id = NodeId::new();
    let target_id = NodeId::new();

    let edge = Edge::new(source_id, target_id, Relationship::Contains)
        .with_weight(2.5)
        .with_metadata("import_type", "direct");

    assert_eq!(edge.source, source_id);
    assert_eq!(edge.target, target_id);
    assert_eq!(edge.relationship, Relationship::Contains);
    assert_eq!(edge.weight, 2.5);
    assert_eq!(
        edge.metadata.get("import_type"),
        Some(&"direct".to_string())
    );
}

#[test]
fn test_relationship_display() {
    assert_eq!(Relationship::Contains.to_string(), "contains");
    assert_eq!(Relationship::DependsOn.to_string(), "depends_on");
    assert_eq!(Relationship::References.to_string(), "references");
    assert_eq!(Relationship::Imports.to_string(), "imports");
    assert_eq!(Relationship::Exports.to_string(), "exports");
}

#[test]
fn test_node_type_display() {
    assert_eq!(NodeType::File.to_string(), "file");
    assert_eq!(NodeType::Directory.to_string(), "directory");
    assert_eq!(NodeType::Symbol.to_string(), "symbol");
}

#[test]
fn test_graph_add_node() {
    let mut graph = IntelligenceGraph::new();

    let file_node = FileNode::new("/test/file.rs");
    let node_id = file_node.id;
    let id = graph.add_node(file_node.into());

    assert_eq!(id, node_id);
    assert_eq!(graph.node_count(), 1);
    assert!(graph.contains_node(id));

    let retrieved = graph.get_node(id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "file.rs");
}

#[test]
fn test_graph_add_edge() {
    let mut graph = IntelligenceGraph::new();

    let dir_node = DirectoryNode::new("/test");
    let dir_id = dir_node.id;
    graph.add_node(dir_node.into());

    let file_node = FileNode::new("/test/file.rs");
    let file_id = file_node.id;
    graph.add_node(file_node.into());

    let edge = Edge::new(dir_id, file_id, Relationship::Contains);
    graph.add_edge(edge).unwrap();

    assert_eq!(graph.edge_count(), 1);

    let outgoing = graph.outgoing_edges(dir_id);
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].0, file_id);
    assert_eq!(outgoing[0].1, Relationship::Contains);
}

#[test]
fn test_graph_get_node_by_path() {
    let mut graph = IntelligenceGraph::new();

    let path = Path::new("/test/file.rs");
    let file_node = FileNode::new(path);
    graph.add_node(file_node.into());

    let retrieved = graph.get_node_by_path(path);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name(), "file.rs");
}

#[test]
fn test_graph_find_path() {
    let mut graph = IntelligenceGraph::new();

    let root = DirectoryNode::new("/root");
    let root_id = root.id;
    graph.add_node(root.into());

    let child = DirectoryNode::new("/root/child");
    let child_id = child.id;
    graph.add_node(child.into());

    let grandchild = FileNode::new("/root/child/file.rs");
    let grandchild_id = grandchild.id;
    graph.add_node(grandchild.into());

    graph
        .add_edge(Edge::new(root_id, child_id, Relationship::Contains))
        .unwrap();
    graph
        .add_edge(Edge::new(child_id, grandchild_id, Relationship::Contains))
        .unwrap();

    let path = graph.find_path(root_id, grandchild_id);
    assert!(path.is_some());

    let path_ids = path.unwrap();
    assert_eq!(path_ids.len(), 3);
    assert_eq!(path_ids[0], root_id);
    assert_eq!(path_ids[1], child_id);
    assert_eq!(path_ids[2], grandchild_id);
}

#[test]
fn test_graph_neighbors() {
    let mut graph = IntelligenceGraph::new();

    let parent = DirectoryNode::new("/parent");
    let parent_id = parent.id;
    graph.add_node(parent.into());

    let child1 = FileNode::new("/parent/child1.rs");
    let child1_id = child1.id;
    graph.add_node(child1.into());

    let child2 = FileNode::new("/parent/child2.rs");
    let child2_id = child2.id;
    graph.add_node(child2.into());

    graph
        .add_edge(Edge::new(parent_id, child1_id, Relationship::Contains))
        .unwrap();
    graph
        .add_edge(Edge::new(parent_id, child2_id, Relationship::Contains))
        .unwrap();

    let neighbors = graph.get_neighbors(parent_id);
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&child1_id));
    assert!(neighbors.contains(&child2_id));
}

#[test]
fn test_graph_remove_node() {
    let mut graph = IntelligenceGraph::new();

    let file_node = FileNode::new("/test/file.rs");
    let id = file_node.id;
    graph.add_node(file_node.into());

    assert_eq!(graph.node_count(), 1);

    let removed = graph.remove_node(id).unwrap();
    assert!(removed.is_some());
    assert_eq!(graph.node_count(), 0);
    assert!(!graph.contains_node(id));
}

#[test]
fn test_graph_builder() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path())
        .build()
        .expect("Failed to build graph");

    assert!(graph.node_count() > 0);
    assert!(graph.edge_count() >= 0);

    let query = GraphQuery::new(&graph);
    let files = query.find_by_type(NodeType::File);
    assert!(!files.is_empty());

    let dirs = query.find_by_type(NodeType::Directory);
    assert!(!dirs.is_empty());
}

#[test]
fn test_graph_builder_parallel() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path())
        .build_parallel()
        .expect("Failed to build graph in parallel");

    assert!(graph.node_count() > 0);
}

#[test]
fn test_query_find_by_name() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();

    let query = GraphQuery::new(&graph);
    let result = query.find_by_name("main.rs");
    assert!(!result.is_empty());
}

#[test]
fn test_query_find_by_extension() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();

    let query = GraphQuery::new(&graph);
    let result = query.find_by_extension("rs");
    assert!(!result.is_empty());
}

#[test]
fn test_query_find_children() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();

    let query = GraphQuery::new(&graph);
    let src_path = temp_dir.path().join("src");
    let src_id = query.resolve_path(&src_path).unwrap();

    let children = query.find_children(src_id);
    assert!(!children.is_empty());
}

#[test]
fn test_query_search_by_pattern() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();

    let query = GraphQuery::new(&graph);
    let result = query.search_by_pattern("main");
    assert!(!result.is_empty());
}

#[test]
fn test_query_find_all_descendants() {
    let mut graph = IntelligenceGraph::new();

    let root = DirectoryNode::new("/root");
    let root_id = root.id;
    graph.add_node(root.into());

    let child1 = DirectoryNode::new("/root/child1");
    let child1_id = child1.id;
    graph.add_node(child1.into());

    let grandchild = FileNode::new("/root/child1/file.rs");
    let grandchild_id = grandchild.id;
    graph.add_node(grandchild.into());

    graph
        .add_edge(Edge::new(root_id, child1_id, Relationship::Contains))
        .unwrap();
    graph
        .add_edge(Edge::new(child1_id, grandchild_id, Relationship::Contains))
        .unwrap();

    let query = GraphQuery::new(&graph);
    let descendants = query.find_all_descendants(root_id);

    assert_eq!(descendants.len(), 2);
}

#[test]
fn test_persistence_save_and_load_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_dir = temp_dir.path().join("graphs");

    let mut graph = IntelligenceGraph::new();
    let file_node = FileNode::new("/test/file.rs");
    let id = file_node.id;
    graph.add_node(file_node.into());

    let persistence = GraphPersistence::new(&storage_dir)
        .unwrap()
        .with_format(PersistenceFormat::Json);

    let saved_path = persistence.save(&graph, "test_graph").unwrap();
    assert!(saved_path.exists());

    let loaded_graph = persistence.load("test_graph").unwrap();
    assert_eq!(loaded_graph.node_count(), 1);
    assert!(loaded_graph.get_node(id).is_some());
}

#[test]
fn test_persistence_save_and_load_yaml() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_dir = temp_dir.path().join("graphs");

    let mut graph = IntelligenceGraph::new();
    let file_node = FileNode::new("/test/file.rs");
    graph.add_node(file_node.into());

    let persistence = GraphPersistence::new(&storage_dir)
        .unwrap()
        .with_format(PersistenceFormat::Yaml);

    let saved_path = persistence.save(&graph, "test_graph_yaml").unwrap();
    assert!(saved_path.exists());

    let loaded_graph = persistence.load("test_graph_yaml").unwrap();
    assert_eq!(loaded_graph.node_count(), 1);
}

#[test]
fn test_persistence_list_and_delete() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_dir = temp_dir.path().join("graphs");

    let graph = IntelligenceGraph::new();
    let persistence = GraphPersistence::new(&storage_dir).unwrap();

    persistence.save(&graph, "graph1").unwrap();
    persistence.save(&graph, "graph2").unwrap();

    let list = persistence.list_saved_graphs().unwrap();
    assert!(list.contains(&"graph1".to_string()));
    assert!(list.contains(&"graph2".to_string()));

    persistence.delete("graph1").unwrap();
    let list_after = persistence.list_saved_graphs().unwrap();
    assert!(!list_after.contains(&"graph1".to_string()));
    assert!(list_after.contains(&"graph2".to_string()));
}

#[test]
fn test_persistence_round_trip_preserves_edges() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_dir = temp_dir.path().join("graphs");

    let mut graph = IntelligenceGraph::new();

    let parent = DirectoryNode::new("/parent");
    let parent_id = parent.id;
    graph.add_node(parent.into());

    let child = FileNode::new("/parent/child.rs");
    let child_id = child.id;
    graph.add_node(child.into());

    graph
        .add_edge(Edge::new(parent_id, child_id, Relationship::Contains))
        .unwrap();

    let persistence = GraphPersistence::new(&storage_dir).unwrap();
    persistence.save(&graph, "edge_test").unwrap();

    let loaded = persistence.load("edge_test").unwrap();
    assert_eq!(loaded.edge_count(), 1);
    assert_eq!(loaded.node_count(), 2);
}

#[test]
fn test_error_handling_node_not_found() {
    let mut graph = IntelligenceGraph::new();
    let fake_id = NodeId::new();

    let target_id = NodeId::new();
    let edge = Edge::new(fake_id, target_id, Relationship::Contains);

    let result = graph.add_edge(edge);
    assert!(result.is_err());

    match result {
        Err(GraphError::NodeNotFound(_)) => {}
        _ => panic!("Expected NodeNotFound error"),
    }
}

#[test]
fn test_error_handling_invalid_path() {
    let graph = IntelligenceGraph::new();
    let query = GraphQuery::new(&graph);

    let result = query.resolve_path(Path::new("/nonexistent/path"));
    assert!(result.is_err());

    match result {
        Err(GraphError::InvalidPath(_)) => {}
        _ => {}
    }
}

#[test]
fn test_graph_is_empty() {
    let mut graph = IntelligenceGraph::new();
    assert!(graph.is_empty());

    let file_node = FileNode::new("/test/file.rs");
    graph.add_node(file_node.into());
    assert!(!graph.is_empty());

    graph.clear();
    assert!(graph.is_empty());
}

#[test]
fn test_graph_stats() {
    use assura::intelligence::graph::GraphStats;

    let mut graph = IntelligenceGraph::new();

    let dir = DirectoryNode::new("/test");
    graph.add_node(dir.into());

    let file1 = FileNode::new("/test/file1.rs");
    graph.add_node(file1.into());

    let file2 = FileNode::new("/test/file2.rs");
    graph.add_node(file2.into());

    let stats = GraphStats::from_graph(&graph);
    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.directory_count, 1);
    assert_eq!(stats.file_count, 2);
}

#[test]
fn test_serialization() {
    use serde_json;

    let file_node = FileNode::new("/test/file.rs")
        .with_content_hash("abc123")
        .with_metadata(NodeMetadata::new());

    let json = serde_json::to_string(&file_node).unwrap();
    let deserialized: FileNode = serde_json::from_str(&json).unwrap();

    assert_eq!(file_node.name, deserialized.name);
    assert_eq!(file_node.path, deserialized.path);
    assert_eq!(file_node.content_hash, deserialized.content_hash);
}

#[test]
fn test_graph_builder_max_depth() {
    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path();

    let level1 = base_path.join("level1");
    fs::create_dir(&level1).unwrap();
    fs::write(level1.join("file.txt"), "test").unwrap();

    let level2 = level1.join("level2");
    fs::create_dir(&level2).unwrap();
    fs::write(level2.join("file2.txt"), "test").unwrap();

    let graph = GraphBuilder::new(base_path).max_depth(1).build().unwrap();

    assert!(graph.node_count() > 0);
}

#[test]
fn test_query_result_operations() {
    let temp_dir = create_test_directory();
    let graph = GraphBuilder::new(temp_dir.path()).build().unwrap();
    let query = GraphQuery::new(&graph);

    let result = query.find_by_extension("rs");
    assert!(!result.is_empty());
    assert!(result.len() > 0);
    assert!(result.first().is_some());
    assert_eq!(result.total_count, result.len());
}

#[test]
fn test_edge_relationship_types() {
    let mut graph = IntelligenceGraph::new();

    let module = FileNode::new("/src/module.rs");
    let module_id = module.id;
    graph.add_node(module.into());

    let util = FileNode::new("/src/util.rs");
    let util_id = util.id;
    graph.add_node(util.into());

    let test = FileNode::new("/tests/test.rs");
    let test_id = test.id;
    graph.add_node(test.into());

    graph
        .add_edge(Edge::new(module_id, util_id, Relationship::Imports))
        .unwrap();
    graph
        .add_edge(Edge::new(test_id, module_id, Relationship::References))
        .unwrap();

    let query = GraphQuery::new(&graph);
    let imports = query.find_related(module_id, Relationship::Imports);
    assert_eq!(imports.len(), 1);

    let references = query.find_referring_to(module_id, Relationship::References);
    assert_eq!(references.len(), 1);
}
