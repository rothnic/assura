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

#[test]
fn fact_store_explains_changed_source_outbound_references() {
    let project = reference_project();
    let source_rel = Path::new("docs/note.md");
    let content = fs::read_to_string(project.path().join(source_rel)).unwrap();

    let mut ingestor = FactIngestor::new("refs-1");
    ingestor.ingest_markdown_links(project.path(), source_rel, &content);
    let store = InMemoryFactStore::load(ingestor.finish());

    let outbound = store
        .repository_references_from_path("docs/note.md")
        .into_iter()
        .map(|edge| edge.target_path.clone())
        .collect::<Vec<_>>();

    assert_eq!(outbound.len(), 3);
    assert!(outbound.contains(&PathBuf::from("docs/guide.md")));
    assert!(outbound.contains(&PathBuf::from("src/lib.rs")));
    assert!(outbound.contains(&PathBuf::from("docs/missing.md")));
}

#[test]
fn source_comments_and_strings_create_repository_reference_edges() {
    let project = reference_project();
    let source_rel = Path::new("src/lib.rs");
    let content = r#"
/// See docs/guide.md#install before changing this module.
const GUIDE_LINES: &str = "docs/guide.md:1-2";
const MISSING: &str = "docs/missing.md";
"#;

    let python_rel = Path::new("scripts/task.py");
    let python_content = r#"
def run():
    """See docs/guide.md#install before changing this task."""
    return "ok"
"#;

    let mut ingestor = FactIngestor::new("refs-1");
    ingestor.ingest_source_references(project.path(), source_rel, content);
    ingestor.ingest_source_references(project.path(), python_rel, python_content);
    let facts = ingestor.finish();

    let references = repository_references(&facts.edges);
    assert_eq!(references.len(), 4);
    assert!(references.iter().any(|edge| {
        edge.source_path == source_rel
            && edge.target_path == Path::new("docs/guide.md")
            && edge.target_anchor.as_deref() == Some("install")
            && edge.reference_kind == "doc_comment_reference"
            && edge.confidence == "medium"
            && edge.rule == "repository_reference_anchor"
    }));
    assert!(references.iter().any(|edge| {
        edge.source_path == python_rel
            && edge.target_path == Path::new("docs/guide.md")
            && edge.target_anchor.as_deref() == Some("install")
            && edge.reference_kind == "docstring_reference"
            && edge.confidence == "medium"
    }));
    assert!(references.iter().any(|edge| {
        edge.target_path == Path::new("docs/guide.md")
            && edge.target_line_start == Some(1)
            && edge.target_line_end == Some(2)
            && edge.reference_kind == "string_literal_reference"
            && edge.confidence == "low"
            && edge.rule == "repository_reference_line_anchor"
    }));
    assert!(references.iter().any(|edge| {
        edge.target_path == Path::new("docs/missing.md")
            && !edge.target_exists
            && edge.target_id.is_none()
            && edge.rule == "repository_reference_target"
    }));
    assert!(!facts.facts.iter().any(|fact| matches!(
        fact,
        assura::intelligence::ProjectFact::Diagnostic(diagnostic)
            if diagnostic.rule == "repository_reference_target"
    )));
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
