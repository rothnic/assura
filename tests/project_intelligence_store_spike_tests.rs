use assura::intelligence::{
    model_instance_id, resource_id, EdgeId, FactGeneration, FactId, FactOrigin, FactSet,
    InMemoryFactStore, ModelInstance, PathScope, ProjectEdge, ProjectFact, RelationshipEdge,
    Resource, SearchChunk, SymbolRef,
};
use serde_json::Map;
use std::path::{Path, PathBuf};

#[test]
fn project_intelligence_store_supports_required_query_shapes() {
    let mut facts = fixture_facts("snapshot-1", 8);
    facts.upsert_edge(ProjectEdge::Relationship(RelationshipEdge {
        id: EdgeId::from_parts("relationship", "goal-0002:specs:missing"),
        generation: FactGeneration::new("snapshot-1"),
        origin: FactOrigin::Derived,
        source_id: model_instance_id("goals", "goal-0002"),
        target_id: Some(model_instance_id("specs", "missing")),
        field: "specs".to_string(),
        target_collections: vec!["specs".to_string()],
        target_instance_id: "missing".to_string(),
    }));
    let store = InMemoryFactStore::load(facts);
    let stats = store.stats();

    assert_eq!(stats.fact_count, 59);
    assert_eq!(stats.edge_count, 17);
    assert_eq!(stats.search_chunk_count, 8);
    assert_eq!(stats.path_scope_count, 3);
    assert!(stats.serialized_bytes > 1_000);

    assert_eq!(
        store
            .path_scopes_for_path(Path::new("docs/goals/goal-0003.md"))
            .len(),
        1
    );
    assert_eq!(store.keyword_search("runtime goal 0003").len(), 1);
    assert_eq!(store.missing_relationship_targets().len(), 2);

    let source_id = model_instance_id("goals", "goal-0004");
    let edges = store.edges_from(&source_id);
    assert_eq!(edges.len(), 2);
    assert!(edges.iter().any(|edge| matches!(
        edge,
        ProjectEdge::Relationship(edge)
            if edge.target_id == Some(model_instance_id("specs", "spec-0004"))
    )));
}

#[test]
fn project_intelligence_store_replaces_generation_without_losing_other_generations() {
    let mut store = InMemoryFactStore::load(fixture_facts("snapshot-1", 4));
    store.replace_generation("snapshot-2", fixture_facts("snapshot-2", 4));
    assert_eq!(store.facts().facts.len(), 62);
    assert_eq!(store.facts().edges.len(), 16);
    assert_eq!(store.missing_relationship_targets().len(), 2);

    let replacement = fixture_facts("snapshot-1", 2);
    store.replace_generation("snapshot-1", replacement);

    assert_eq!(store.facts().facts.len(), 48);
    assert_eq!(store.facts().edges.len(), 12);
    assert_eq!(store.missing_relationship_targets().len(), 2);
    assert_eq!(
        store
            .facts_by_id(&model_instance_id("goals", "goal-0003"))
            .len(),
        1
    );
    assert_eq!(store.keyword_search("runtime goal 0001").len(), 2);
}

#[test]
fn project_intelligence_store_reports_benchmark_fixture_footprint() {
    let store = InMemoryFactStore::load(fixture_facts("snapshot-1", 240));
    let stats = store.stats();

    assert_eq!(stats.fact_count, 1_683);
    assert_eq!(stats.edge_count, 480);
    assert_eq!(stats.search_chunk_count, 240);
    assert_eq!(stats.serialized_bytes, 496_203);
}

fn fixture_facts(generation: &str, goals: usize) -> FactSet {
    let generation = FactGeneration::new(generation);
    let mut facts = FactSet::default();
    add_path_scope(&mut facts, &generation, "goals", "docs/goals/*.md");
    add_path_scope(&mut facts, &generation, "specs", "specs/*.json");
    add_path_scope(&mut facts, &generation, "scenarios", "bdd/*.feature");

    for index in 0..goals {
        let goal_id = format!("goal-{index:04}");
        let spec_id = format!("spec-{index:04}");
        add_resource(&mut facts, &generation, &format!("docs/goals/{goal_id}.md"));
        add_resource(&mut facts, &generation, &format!("specs/{spec_id}.json"));
        add_resource(&mut facts, &generation, &format!("bdd/{goal_id}.feature"));
        add_instance(&mut facts, &generation, "goals", &goal_id);
        add_instance(&mut facts, &generation, "specs", &spec_id);
        add_instance(
            &mut facts,
            &generation,
            "scenarios",
            &format!("scenario-{index:04}"),
        );
        facts.upsert_fact(ProjectFact::SearchChunk(SearchChunk {
            id: FactId::from_parts("search_chunk", &format!("goal:{goal_id}")),
            generation: generation.clone(),
            origin: FactOrigin::Derived,
            source_id: model_instance_id("goals", &goal_id),
            text: format!("Runtime goal {index:04} validates graph search behavior"),
        }));
        facts.upsert_edge(ProjectEdge::Relationship(RelationshipEdge {
            id: EdgeId::from_parts("relationship", &format!("{goal_id}:specs:{spec_id}")),
            generation: generation.clone(),
            origin: FactOrigin::Derived,
            source_id: model_instance_id("goals", &goal_id),
            target_id: if index == goals - 1 {
                None
            } else {
                Some(model_instance_id("specs", &spec_id))
            },
            field: "specs".to_string(),
            target_collections: vec!["specs".to_string()],
            target_instance_id: spec_id,
        }));
        facts.upsert_edge(ProjectEdge::SymbolRef(SymbolRef {
            id: EdgeId::from_parts(
                "symbol_ref",
                &format!(
                    "{}:-:crate::runtime::Goal{index:04}",
                    model_instance_id("goals", &goal_id)
                ),
            ),
            generation: generation.clone(),
            origin: FactOrigin::Derived,
            source_id: model_instance_id("goals", &goal_id),
            symbol: format!("crate::runtime::Goal{index:04}"),
            target_id: None,
            provider: None,
        }));
    }
    facts
}

fn add_path_scope(
    facts: &mut FactSet,
    generation: &FactGeneration,
    collection: &str,
    pattern: &str,
) {
    facts.upsert_fact(ProjectFact::PathScope(PathScope {
        id: FactId::from_parts("path_scope", collection),
        generation: generation.clone(),
        origin: FactOrigin::Source,
        model_id: FactId::from_parts("model", collection),
        collection: collection.to_string(),
        pattern: pattern.to_string(),
    }));
}

fn add_resource(facts: &mut FactSet, generation: &FactGeneration, path: &str) {
    facts.upsert_fact(ProjectFact::Resource(Resource {
        id: resource_id(path),
        generation: generation.clone(),
        origin: FactOrigin::Source,
        path: PathBuf::from(path),
        extension: Path::new(path)
            .extension()
            .and_then(|extension| extension.to_str())
            .map(ToOwned::to_owned),
    }));
}

fn add_instance(facts: &mut FactSet, generation: &FactGeneration, collection: &str, id: &str) {
    let path = match collection {
        "goals" => format!("docs/goals/{id}.md"),
        "specs" => format!("specs/{id}.json"),
        _ => format!("bdd/{id}.feature"),
    };
    facts.upsert_fact(ProjectFact::ModelInstance(ModelInstance {
        id: model_instance_id(collection, id),
        generation: generation.clone(),
        origin: FactOrigin::Source,
        model_id: FactId::from_parts("model", collection),
        resource_id: resource_id(path),
        collection: collection.to_string(),
        object_type: collection.to_string(),
        instance_id: id.to_string(),
        data: Map::new(),
    }));
}
