use assura::cli::{StructureCheckReport, StructureViolation};
use assura::config::config::ContentRelationConfig;
use assura::config::loader::ConfigLoader;
use assura::content_repository::{AdapterKind, CollectionSpec, ContentRepository, RepositoryModel};
use assura::intelligence::{
    model_instance_id, resource_id, EdgeId, FactGeneration, FactId, FactIngestor, FactOrigin,
    FactSet, ProjectEdge, ProjectFact, Resource, SymbolRef,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const FIXTURE_ROOT: &str = "tests/fixtures/content_runtime";

#[test]
fn project_intelligence_ingests_content_runtime_facts_deterministically() {
    let first = ingest_fixture("references/valid", "snapshot-1");
    let second = ingest_fixture("references/valid", "snapshot-1");
    let next_generation = ingest_fixture("references/valid", "snapshot-2");

    assert_eq!(first, second);
    assert_eq!(fact_ids(&first), fact_ids(&next_generation));
    assert_eq!(edge_ids(&first), edge_ids(&next_generation));
    assert_eq!(first.count_kind("ModelDefinition"), 4);
    assert!(first.count_kind("FieldDefinition") >= 10);
    assert_eq!(first.count_kind("RelationshipDefinition"), 5);
    assert_eq!(first.count_kind("PathScope"), 4);
    assert_eq!(first.count_kind("Resource"), 4);
    assert_eq!(first.count_kind("ModelInstance"), 4);
    assert_eq!(first.count_kind("MarkdownDocument"), 1);
    assert_eq!(first.count_kind("MarkdownSection"), 1);
    assert!(first.count_kind("SearchChunk") >= 5);

    let relationship_edges = first
        .edges
        .iter()
        .filter(|edge| matches!(edge, ProjectEdge::Relationship(_)))
        .collect::<Vec<_>>();
    assert_eq!(relationship_edges.len(), 4);
    assert!(relationship_edges.iter().any(|edge| match edge {
        ProjectEdge::Relationship(edge) => {
            edge.field == "specs"
                && edge.target_id.is_some()
                && edge.target_instance_id == "spec-portable-structure"
        }
        ProjectEdge::SymbolRef(_) => false,
    }));
}

#[test]
fn project_intelligence_keeps_model_definitions_distinct_by_collection() {
    let model = RepositoryModel {
        collections: vec![
            CollectionSpec {
                name: "active_goals".to_string(),
                object_type: "Goal".to_string(),
                schema_class: None,
                path_pattern: "active/*.json".to_string(),
                adapter: AdapterKind::JsonRecord,
                id_field: "id".to_string(),
                fields: Vec::new(),
                references: Vec::new(),
            },
            CollectionSpec {
                name: "archived_goals".to_string(),
                object_type: "Goal".to_string(),
                schema_class: None,
                path_pattern: "archive/*.json".to_string(),
                adapter: AdapterKind::JsonRecord,
                id_field: "id".to_string(),
                fields: Vec::new(),
                references: Vec::new(),
            },
        ],
        placements: Vec::new(),
        schema_artifact_path: None,
        schema_artifact: None,
    };
    let mut ingestor = FactIngestor::new("model-1");
    ingestor.ingest_repository_model(&model);
    let facts = ingestor.finish();
    let models = facts
        .facts
        .iter()
        .filter_map(|fact| match fact {
            ProjectFact::ModelDefinition(model) => Some(model),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(models.len(), 2);
    assert_eq!(facts.count_kind("PathScope"), 2);
    assert!(models
        .iter()
        .any(|model| model.collection == "active_goals"));
    assert!(models
        .iter()
        .any(|model| model.collection == "archived_goals"));
    assert_ne!(models[0].id, models[1].id);
}

#[test]
fn project_intelligence_ingests_diagnostics_and_safe_fixes() {
    let report = StructureCheckReport {
        success: false,
        project_root: PathBuf::from("/repo"),
        config_path: PathBuf::from("/repo/.assura/config.yml"),
        checked_path: PathBuf::from("/repo"),
        files_checked: 1,
        dirs_checked: 0,
        violations: vec![StructureViolation {
            path: PathBuf::from("docs/note.md"),
            rule: "markdown_trailing_spaces".to_string(),
            message: "Markdown file 'docs/note.md' has 3 trailing whitespace character(s) on blank line 4, column 1".to_string(),
            severity: "low".to_string(),
            corrective_context: "Run `assura fix markdown`.".to_string(),
        }],
    };

    let mut ingestor = FactIngestor::new("check-1");
    ingestor.ingest_check_report(&report);
    let facts = ingestor.finish();

    assert_eq!(facts.count_kind("Diagnostic"), 1);
    assert_eq!(facts.count_kind("SafeFix"), 1);
    let diagnostic = facts
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic) => Some(diagnostic),
            _ => None,
        })
        .expect("diagnostic fact");
    assert_eq!(diagnostic.target_id, Some(resource_id("docs/note.md")));
    assert_eq!(
        diagnostic
            .location
            .as_ref()
            .and_then(|location| location.line),
        Some(4)
    );
    let safe_fix = facts
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::SafeFix(safe_fix) => Some(safe_fix),
            _ => None,
        })
        .expect("safe fix fact");
    assert_eq!(safe_fix.diagnostic_id, diagnostic.id);
    assert_eq!(safe_fix.target_id, Some(resource_id("docs/note.md")));
    assert_eq!(safe_fix.operation, "remove_blank_line_trailing_spaces");
    assert_eq!(
        safe_fix
            .location
            .as_ref()
            .and_then(|location| location.line),
        Some(4)
    );
    assert_eq!(
        safe_fix
            .location
            .as_ref()
            .and_then(|location| location.column),
        Some(1)
    );
}

#[test]
fn project_intelligence_targets_content_diagnostics_to_model_instances() {
    let facts = ingest_fixture_allowing_findings("invalid_shape", "snapshot-1");

    let diagnostic = facts
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic)
                if diagnostic.rule == "content_runtime:invalid_object_shape"
                    && diagnostic
                        .location
                        .as_ref()
                        .and_then(|location| location.field.as_deref())
                        == Some("status")
                    && diagnostic.message.contains("Goal") =>
            {
                Some(diagnostic)
            }
            _ => None,
        })
        .expect("goal shape diagnostic");

    assert_eq!(
        diagnostic.target_id,
        Some(model_instance_id("goals", "goal-portable-structure"))
    );
    assert_eq!(
        diagnostic
            .location
            .as_ref()
            .map(|location| location.path.as_path()),
        Some(Path::new("docs/goals/goal_portable_structure.md"))
    );
}

#[test]
fn project_intelligence_leaves_ambiguous_relationship_edges_unresolved() {
    let facts = ingest_fixture_allowing_findings("references/ambiguous", "snapshot-1");

    let edge = facts
        .edges
        .iter()
        .find_map(|edge| match edge {
            ProjectEdge::Relationship(edge)
                if edge.field == "related" && edge.target_instance_id == "shared" =>
            {
                Some(edge)
            }
            _ => None,
        })
        .expect("ambiguous relationship edge");

    assert_eq!(edge.target_collections, vec!["goals", "specs"]);
    assert_eq!(edge.target_id, None);
}

#[test]
fn project_intelligence_resolves_inferred_relationship_targets_when_unique() {
    let root = PathBuf::from(FIXTURE_ROOT).join("references/valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config.relations.insert(
        "events.related".to_string(),
        ContentRelationConfig {
            target: None,
            targets: Vec::new(),
            many: false,
            required: false,
            acyclic: false,
        },
    );
    let model = RepositoryModel::from_config(&root, &config).expect("repository model compiles");
    let repository =
        ContentRepository::from_config(&root, &config).expect("content repository compiles");
    let validation = repository.validate(&root);
    assert_eq!(validation.findings, Vec::new());
    let mut ingestor = FactIngestor::new("snapshot-1");
    ingestor.ingest_repository_model(&model);
    ingestor.ingest_repository_validation(&validation);
    let facts = ingestor.finish();

    let edge = facts
        .edges
        .iter()
        .find_map(|edge| match edge {
            ProjectEdge::Relationship(edge)
                if edge.field == "related"
                    && edge.target_instance_id == "goal-portable-structure" =>
            {
                Some(edge)
            }
            _ => None,
        })
        .expect("inferred relationship edge");

    assert_eq!(
        edge.target_id,
        Some(model_instance_id("goals", "goal-portable-structure"))
    );
    assert_eq!(edge.target_collections, vec!["decisions", "goals", "specs"]);
}

#[test]
fn project_intelligence_falls_back_to_resource_for_ambiguous_jsonl_diagnostics() {
    let project = jsonl_project_with_multiple_goal_records_and_one_invalid();
    let facts = ingest_root_allowing_findings(project.path(), "snapshot-1");

    let diagnostic = facts
        .facts
        .iter()
        .find_map(|fact| match fact {
            ProjectFact::Diagnostic(diagnostic)
                if diagnostic.rule == "content_runtime:invalid_object_shape"
                    && diagnostic
                        .location
                        .as_ref()
                        .and_then(|location| location.field.as_deref())
                        == Some("status")
                    && diagnostic.location.as_ref().is_some_and(|location| {
                        location.path == Path::new("goals/goals.jsonl")
                    }) =>
            {
                Some(diagnostic)
            }
            _ => None,
        })
        .expect("ambiguous JSONL diagnostic");

    assert_eq!(diagnostic.target_id, Some(resource_id("goals/goals.jsonl")));
}

#[test]
fn project_intelligence_replaces_generation_without_storage_backend() {
    let mut facts = ingest_fixture("valid", "snapshot-1");
    assert!(facts.count_kind("ModelInstance") > 0);

    let mut replacement = FactSet::default();
    replacement.upsert_fact(ProjectFact::Resource(Resource {
        id: resource_id("docs/replacement.md"),
        generation: FactGeneration::new("snapshot-1"),
        origin: FactOrigin::Source,
        path: PathBuf::from("docs/replacement.md"),
        extension: Some("md".to_string()),
    }));

    facts.replace_generation("snapshot-1", replacement);

    assert_eq!(facts.facts.len(), 1);
    assert_eq!(facts.edges.len(), 0);
    assert!(matches!(
        &facts.facts[0],
        ProjectFact::Resource(resource) if resource.path == Path::new("docs/replacement.md")
    ));
}

#[test]
fn project_intelligence_preserves_overlapping_fact_ids_from_other_generations() {
    let mut facts = FactSet::default();
    facts.upsert_fact(resource_fact("docs/note.md", "snapshot-1"));
    facts.upsert_fact(resource_fact("docs/note.md", "snapshot-2"));
    facts.upsert_edge(symbol_ref_edge("snapshot-1"));
    facts.upsert_edge(symbol_ref_edge("snapshot-2"));
    assert_eq!(facts.facts.len(), 2);
    assert_eq!(facts.edges.len(), 2);

    let mut replacement = FactSet::default();
    replacement.upsert_fact(resource_fact("docs/replacement.md", "snapshot-1"));
    facts.replace_generation("snapshot-1", replacement);

    assert_eq!(facts.facts.len(), 2);
    assert_eq!(facts.edges.len(), 1);
    assert!(facts.facts.iter().any(|fact| matches!(
        fact,
        ProjectFact::Resource(resource)
            if resource.path == Path::new("docs/note.md")
                && resource.generation.id == "snapshot-2"
    )));
    assert!(facts.facts.iter().any(|fact| matches!(
        fact,
        ProjectFact::Resource(resource)
            if resource.path == Path::new("docs/replacement.md")
                && resource.generation.id == "snapshot-1"
    )));
    assert!(facts.edges.iter().any(|edge| matches!(
        edge,
        ProjectEdge::SymbolRef(edge) if edge.generation.id == "snapshot-2"
    )));
}

#[test]
fn project_intelligence_allows_unresolved_code_symbol_refs() {
    let source_id = resource_id("docs/goals/goal_portable_structure.md");
    let mut ingestor = FactIngestor::new("symbols-1");

    ingestor.add_symbol_ref(source_id.clone(), "crate::config::Config", None, None);
    let facts = ingestor.finish();

    let symbol_ref = facts
        .edges
        .iter()
        .find_map(|edge| match edge {
            ProjectEdge::SymbolRef(edge) => Some(edge),
            _ => None,
        })
        .expect("symbol ref edge");
    assert_eq!(symbol_ref.source_id, source_id);
    assert_eq!(symbol_ref.symbol, "crate::config::Config");
    assert_eq!(symbol_ref.target_id, None);
    assert_eq!(symbol_ref.provider, None);
}

fn ingest_fixture(name: &str, generation: &str) -> FactSet {
    let (facts, finding_count) = ingest_fixture_with_finding_count(name, generation);
    assert_eq!(finding_count, 0);
    facts
}

fn ingest_fixture_allowing_findings(name: &str, generation: &str) -> FactSet {
    ingest_fixture_with_finding_count(name, generation).0
}

fn ingest_fixture_with_finding_count(name: &str, generation: &str) -> (FactSet, usize) {
    let root = PathBuf::from(FIXTURE_ROOT).join(name);
    ingest_root_with_finding_count(&root, generation)
}

fn ingest_root_allowing_findings(root: &Path, generation: &str) -> FactSet {
    ingest_root_with_finding_count(root, generation).0
}

fn ingest_root_with_finding_count(root: &Path, generation: &str) -> (FactSet, usize) {
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    let model = RepositoryModel::from_config(root, &config).expect("repository model compiles");
    let repository =
        ContentRepository::from_config(root, &config).expect("content repository compiles");
    let validation = repository.validate(root);
    let finding_count = validation.findings.len();

    let mut ingestor = FactIngestor::new(generation);
    ingestor.ingest_repository_model(&model);
    ingestor.ingest_repository_validation(&validation);
    (ingestor.finish(), finding_count)
}

fn jsonl_project_with_multiple_goal_records_and_one_invalid() -> TempDir {
    let source = PathBuf::from(FIXTURE_ROOT).join("adapters/jsonl/valid");
    let project = TempDir::new().expect("temp project");
    for dir in [".assura", "goals", "schemas", "specs"] {
        fs::create_dir_all(project.path().join(dir)).expect("fixture dir");
    }
    for path in [
        ".assura/config.yml",
        "schemas/content_runtime.schema.json",
        "specs/specs.jsonl",
    ] {
        fs::copy(source.join(path), project.path().join(path)).expect("copy fixture file");
    }
    let goals = fs::read_to_string(source.join("goals/goals.jsonl")).expect("goals fixture");
    fs::write(
        project.path().join("goals/goals.jsonl"),
        format!(
            "{goals}{{\"id\":\"goal-invalid-status\",\"title\":\"Invalid status\",\"status\":\"unknown\",\"specs\":[\"spec-portable-structure\"]}}\n"
        ),
    )
    .expect("write ambiguous goals");
    project
}

fn fact_ids(facts: &FactSet) -> Vec<FactId> {
    facts.facts.iter().map(|fact| fact.id().clone()).collect()
}

fn edge_ids(facts: &FactSet) -> Vec<EdgeId> {
    facts.edges.iter().map(|edge| edge.id().clone()).collect()
}

fn resource_fact(path: &str, generation: &str) -> ProjectFact {
    ProjectFact::Resource(Resource {
        id: resource_id(path),
        generation: FactGeneration::new(generation),
        origin: FactOrigin::Source,
        path: PathBuf::from(path),
        extension: Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(ToOwned::to_owned),
    })
}

fn symbol_ref_edge(generation: &str) -> ProjectEdge {
    let source_id = resource_id("docs/note.md");
    ProjectEdge::SymbolRef(SymbolRef {
        id: EdgeId::from_parts(
            "symbol_ref",
            &format!("{}:-:crate::config::Config", source_id),
        ),
        generation: FactGeneration::new(generation),
        origin: FactOrigin::Derived,
        source_id,
        symbol: "crate::config::Config".to_string(),
        target_id: None,
        provider: None,
    })
}
