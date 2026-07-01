use assura::intelligence::{
    resource_id, FactIngestor, InMemoryFactStore, ProjectEdge, RepositoryReferenceEdge,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn markdown_links_create_repository_reference_edges() {
    let project = reference_project();
    let source_rel = Path::new("docs/note.md");
    let content = fs::read_to_string(project.path().join(source_rel)).unwrap();

    let mut ingestor = FactIngestor::new("refs-1");
    ingestor.ingest_markdown_links(project.path(), source_rel, &content);
    let facts = ingestor.finish();

    let references = repository_references(&facts.edges);
    assert_eq!(references.len(), 3);
    assert!(references.iter().any(|edge| {
        edge.source_path == Path::new("docs/note.md")
            && edge.source_line == Some(3)
            && edge.target_path == Path::new("docs/guide.md")
            && edge.target_anchor.as_deref() == Some("install")
            && edge.target_exists
            && edge.target_id == Some(resource_id("docs/guide.md"))
            && edge.reference_kind == "markdown_link"
            && edge.confidence == "exact"
            && edge.rule == "markdown_link_heading_anchor"
    }));
    assert!(references.iter().any(|edge| {
        edge.target_path == Path::new("src/lib.rs")
            && edge.target_line_start == Some(1)
            && edge.target_line_end == Some(2)
            && edge.target_id == Some(resource_id("src/lib.rs"))
            && edge.rule == "markdown_link_line_anchor"
    }));
    assert!(references.iter().any(|edge| {
        edge.target_path == Path::new("docs/missing.md")
            && !edge.target_exists
            && edge.target_id.is_none()
            && edge.rule == "markdown_link_target"
    }));
}

#[test]
fn fact_store_indexes_inbound_repository_references_by_target() {
    let project = reference_project();
    let source_rel = Path::new("docs/note.md");
    let content = fs::read_to_string(project.path().join(source_rel)).unwrap();

    let mut ingestor = FactIngestor::new("refs-1");
    ingestor.ingest_markdown_links(project.path(), source_rel, &content);
    let store = InMemoryFactStore::load(ingestor.finish());

    let guide_refs = store.repository_references_to(&resource_id("docs/guide.md"));
    assert_eq!(guide_refs.len(), 1);
    assert_eq!(guide_refs[0].source_path, PathBuf::from("docs/note.md"));
    assert_eq!(guide_refs[0].target_anchor.as_deref(), Some("install"));

    let code_refs = store.repository_references_to(&resource_id("src/lib.rs"));
    assert_eq!(code_refs.len(), 1);
    assert_eq!(code_refs[0].target_line_start, Some(1));
    assert!(store
        .repository_references_to(&resource_id("docs/missing.md"))
        .is_empty());

    let stats = store.stats();
    assert_eq!(stats.repository_reference_target_count, 2);
    assert_eq!(stats.edge_count, 3);
}

fn reference_project() -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nSee [guide](guide.md#install), [code](../src/lib.rs#L1-L2), and [missing](missing.md).\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/guide.md"),
        "# Guide\n\n## Install\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();
    project
}

fn repository_references(edges: &[ProjectEdge]) -> Vec<&RepositoryReferenceEdge> {
    edges
        .iter()
        .filter_map(|edge| match edge {
            ProjectEdge::RepositoryReference(edge) => Some(edge),
            _ => None,
        })
        .collect()
}
