use assura::intelligence::{FactId, FactIngestor, FactSet, MarkdownLink, ProjectFact};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn project_intelligence_ingests_markdown_link_facts_deterministically() {
    let project = markdown_link_project();
    let source_rel = Path::new("docs/note.md");
    let content = fs::read_to_string(project.path().join(source_rel)).unwrap();

    let first = ingest_links(project.path(), source_rel, &content, "links-1");
    let second = ingest_links(project.path(), source_rel, &content, "links-1");
    let next_generation = ingest_links(project.path(), source_rel, &content, "links-2");

    assert_eq!(first, second);
    assert_eq!(fact_ids(&first), fact_ids(&next_generation));
    assert_eq!(first.count_kind("MarkdownDocument"), 1);
    assert_eq!(first.count_kind("MarkdownLink"), 5);

    let links = markdown_links(&first);
    let doc = links
        .iter()
        .find(|link| link.raw_target == "target.md#install-steps")
        .expect("heading link fact");
    assert_eq!(doc.source_path, PathBuf::from("docs/note.md"));
    assert_eq!(doc.source_line, 3);
    assert!(doc.source_column > 0);
    assert_eq!(doc.target_path, PathBuf::from("docs/target.md"));
    assert_eq!(doc.target_anchor.as_deref(), Some("install-steps"));
    assert_eq!(doc.target_line_start, None);
    assert_eq!(doc.target_line_end, None);
    assert!(doc.target_exists);
    assert_eq!(doc.rule, "markdown_link_heading_anchor");

    let code = links
        .iter()
        .find(|link| link.raw_target == "../src/lib.rs#L1-L2")
        .expect("line-range link fact");
    assert_eq!(code.target_path, PathBuf::from("src/lib.rs"));
    assert_eq!(code.target_line_start, Some(1));
    assert_eq!(code.target_line_end, Some(2));
    assert!(code.target_exists);
    assert_eq!(code.rule, "markdown_link_line_anchor");

    let missing = links
        .iter()
        .find(|link| link.raw_target == "missing.md")
        .expect("missing target link fact");
    assert_eq!(missing.target_path, PathBuf::from("docs/missing.md"));
    assert!(!missing.target_exists);
    assert_eq!(missing.rule, "markdown_link_target");

    let missing_anchor = links
        .iter()
        .find(|link| link.raw_target == "missing.md#gone")
        .expect("missing anchored target link fact");
    assert_eq!(missing_anchor.target_path, PathBuf::from("docs/missing.md"));
    assert_eq!(missing_anchor.target_anchor.as_deref(), Some("gone"));
    assert!(!missing_anchor.target_exists);
    assert_eq!(missing_anchor.rule, "markdown_link_target");

    let unicode = links
        .iter()
        .find(|link| link.raw_target == "target.md")
        .expect("unicode-prefix link fact");
    assert_eq!(unicode.source_line, 4);
    assert_eq!(unicode.source_column, 6);
}

fn markdown_link_project() -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nSee [doc](target.md#install-steps), [code](../src/lib.rs#L1-L2), [missing](missing.md), and [missing anchor](missing.md#gone).\nCafé [unicode](target.md).\nUse `[ignored](ignored.md)` as text.\n![image](missing.png)\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/target.md"),
        "# Target\n\n## Install Steps\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();
    project
}

fn ingest_links(root: &Path, source_rel: &Path, content: &str, generation: &str) -> FactSet {
    let mut ingestor = FactIngestor::new(generation);
    ingestor.ingest_markdown_links(root, source_rel, content);
    ingestor.finish()
}

fn markdown_links(facts: &FactSet) -> Vec<&MarkdownLink> {
    facts
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::MarkdownLink(link) => Some(link),
            _ => None,
        })
        .collect()
}

fn fact_ids(facts: &FactSet) -> Vec<FactId> {
    facts.facts.iter().map(|fact| fact.id().clone()).collect()
}
