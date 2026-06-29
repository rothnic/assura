use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentRepository, RepositoryModel};
use assura::intelligence::{
    model_instance_id, resource_id, EdgeId, FactGeneration, FactId, FactIngestor, FactOrigin,
    FactSet, InMemoryFactStore, ModelInstance, PathScope, ProjectEdge, ProjectFact,
    RelationshipEdge, Resource, SearchChunk,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde_json::Map;
use std::path::{Path, PathBuf};
use std::time::Duration;

const GOALS: usize = 240;

fn project_intelligence_benchmarks(c: &mut Criterion) {
    let fixture = SpikeFixture::new(GOALS);

    let mut group = c.benchmark_group("project_intelligence/store_spike");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.throughput(Throughput::Elements(GOALS as u64));

    group.bench_function("assura_in_memory/cold_load", |b| {
        b.iter(|| {
            let store = InMemoryFactStore::load(fixture.snapshot_1.clone());
            black_box(store.stats())
        })
    });

    let store = InMemoryFactStore::load(fixture.snapshot_1.clone());
    group.bench_function("assura_in_memory/warm_missing_target_traversal", |b| {
        b.iter(|| black_box(store.missing_relationship_targets().len()))
    });
    group.bench_function("assura_in_memory/warm_path_scope_query", |b| {
        b.iter(|| {
            black_box(
                store
                    .path_scopes_for_path(Path::new("docs/goals/goal-0120.md"))
                    .len(),
            )
        })
    });
    group.bench_function("assura_in_memory/warm_text_search", |b| {
        b.iter(|| black_box(store.keyword_search("runtime goal 0120").len()))
    });
    group.bench_function("assura_in_memory/incremental_replace_generation", |b| {
        b.iter(|| {
            let mut store = InMemoryFactStore::load(fixture.snapshot_1.clone());
            store.replace_generation("snapshot-1", fixture.snapshot_2.clone());
            black_box(store.stats())
        })
    });
    group.bench_function("assura_in_memory/serialized_footprint_bytes", |b| {
        b.iter(|| black_box(store.stats().serialized_bytes))
    });

    group.finish();

    project_intelligence_session_benchmarks(c);
}

fn project_intelligence_session_benchmarks(c: &mut Criterion) {
    let cases = [
        (
            "assura_repo",
            Path::new("."),
            "Project Intelligence Usability",
        ),
        (
            "beacon_crm",
            Path::new("tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid"),
            "checkout",
        ),
    ];

    let mut group = c.benchmark_group("project_intelligence/session_reuse");
    group.sample_size(10);
    group.warm_up_time(Duration::from_millis(250));

    for (name, root, query) in cases {
        group.bench_function(format!("{name}/cold_fact_load"), |b| {
            b.iter(|| black_box(load_project_intelligence_store(root)))
        });

        let store = load_project_intelligence_store(root);
        group.bench_function(format!("{name}/warm_keyword_search"), |b| {
            b.iter(|| black_box(store.keyword_search(query).len()))
        });
        group.bench_function(format!("{name}/warm_missing_relations"), |b| {
            b.iter(|| black_box(store.missing_relationship_targets().len()))
        });
    }

    group.finish();
}

fn load_project_intelligence_store(root: &Path) -> InMemoryFactStore {
    let config_path = root.join(".assura/config.yml");
    let config = ConfigLoader::load(&config_path).expect("benchmark config loads");
    let model = RepositoryModel::from_config(root, &config).expect("benchmark model loads");
    let repository = ContentRepository::try_new(model.clone()).expect("benchmark repository loads");
    let validation = repository.validate(root);

    let mut ingestor = FactIngestor::new("project-intelligence-session-bench");
    ingestor.ingest_repository_model(&model);
    ingestor.ingest_repository_validation(&validation);
    InMemoryFactStore::load(ingestor.finish())
}

struct SpikeFixture {
    snapshot_1: FactSet,
    snapshot_2: FactSet,
}

impl SpikeFixture {
    fn new(goals: usize) -> Self {
        Self {
            snapshot_1: fixture_facts("snapshot-1", goals),
            snapshot_2: fixture_facts("snapshot-2", goals),
        }
    }
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
        add_search_chunk(&mut facts, &generation, index, &goal_id);
        add_relationship(&mut facts, &generation, index, goals, &goal_id, &spec_id);
        add_symbol_ref(&mut facts, &generation, &goal_id, index);
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

fn add_search_chunk(facts: &mut FactSet, generation: &FactGeneration, index: usize, goal_id: &str) {
    facts.upsert_fact(ProjectFact::SearchChunk(SearchChunk {
        id: FactId::from_parts("search_chunk", &format!("goal:{goal_id}")),
        generation: generation.clone(),
        origin: FactOrigin::Derived,
        source_id: model_instance_id("goals", goal_id),
        text: format!(
            "Runtime goal {index:04} validates graph search behavior with diagnostics and scenarios"
        ),
    }));
}

fn add_relationship(
    facts: &mut FactSet,
    generation: &FactGeneration,
    index: usize,
    goals: usize,
    goal_id: &str,
    spec_id: &str,
) {
    facts.upsert_edge(ProjectEdge::Relationship(RelationshipEdge {
        id: EdgeId::from_parts("relationship", &format!("{goal_id}:specs:{spec_id}")),
        generation: generation.clone(),
        origin: FactOrigin::Derived,
        source_id: model_instance_id("goals", goal_id),
        target_id: if index + 1 == goals {
            None
        } else {
            Some(model_instance_id("specs", spec_id))
        },
        field: "specs".to_string(),
        target_collections: vec!["specs".to_string()],
        target_instance_id: spec_id.to_string(),
    }));
}

fn add_symbol_ref(facts: &mut FactSet, generation: &FactGeneration, goal_id: &str, index: usize) {
    let source_id = model_instance_id("goals", goal_id);
    facts.upsert_edge(ProjectEdge::SymbolRef(assura::intelligence::SymbolRef {
        id: EdgeId::from_parts(
            "symbol_ref",
            &format!("{source_id}:-:crate::runtime::Goal{index:04}"),
        ),
        generation: generation.clone(),
        origin: FactOrigin::Derived,
        source_id,
        symbol: format!("crate::runtime::Goal{index:04}"),
        field: Some("implementation".to_string()),
        target_id: None,
        provider: None,
    }));
}

criterion_group!(project_intelligence, project_intelligence_benchmarks);
criterion_main!(project_intelligence);
