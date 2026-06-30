use assura::intelligence::{
    local_hash_embedding, local_hash_embedding_record, model_instance_id, FactGeneration, FactId,
    FactIngestor, FactOrigin, FactSet, InMemoryFactStore, ProjectFact, SearchChunk,
    LOCAL_HASH_EMBEDDING_PROVIDER,
};

#[test]
fn semantic_embeddings_are_optional_until_ingested() {
    let facts = semantic_fixture(false);
    let store = InMemoryFactStore::load(facts);

    let query = local_hash_embedding("portable policy");

    assert_eq!(store.stats().embedding_record_count, 0);
    assert!(store
        .semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 10)
        .is_empty());
}

#[test]
fn semantic_search_returns_deterministic_ranked_candidates() {
    let facts = semantic_fixture(true);
    let store = InMemoryFactStore::load(facts);

    let query = local_hash_embedding("portable structure policy");
    let hits = store.semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 2);

    assert_eq!(hits.len(), 2);
    assert!(hits[0].score >= hits[1].score);
    assert_eq!(
        hits[0].chunk.source_id,
        model_instance_id("goals", "portable")
    );
    assert_eq!(hits[0].embedding.provider, LOCAL_HASH_EMBEDDING_PROVIDER);
    assert_eq!(hits[0].embedding.text_hash.len(), 16);
}

#[test]
fn semantic_search_limit_zero_returns_no_candidates() {
    let facts = semantic_fixture(true);
    let store = InMemoryFactStore::load(facts);
    let query = local_hash_embedding("portable policy");

    assert!(store
        .semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 0)
        .is_empty());
}

#[test]
fn semantic_search_drops_zero_score_candidates() {
    let facts = semantic_fixture(true);
    let store = InMemoryFactStore::load(facts);
    let query = local_hash_embedding("");

    assert!(store
        .semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 10)
        .is_empty());
}

#[test]
fn semantic_embedding_hash_changes_when_chunk_text_changes() {
    let generation = FactGeneration::new("semantic-test");
    let mut original = test_chunk(&generation, "portable", "Portable structure policy");
    let original_record = local_hash_embedding_record(&original);

    original.text = "Portable structure policy with updated diagnostics".to_string();
    let changed_record = local_hash_embedding_record(&original);

    assert_ne!(original_record.id, changed_record.id);
    assert_ne!(original_record.text_hash, changed_record.text_hash);
}

#[test]
fn semantic_search_ignores_stale_embedding_records() {
    let generation = FactGeneration::new("semantic-test");
    let mut changed_chunk = test_chunk(&generation, "portable", "Portable structure policy");
    let stale_record = local_hash_embedding_record(&changed_chunk);
    changed_chunk.text = "Release contract checks for archive evidence".to_string();

    let mut facts = FactSet::default();
    facts.upsert_fact(ProjectFact::SearchChunk(changed_chunk));
    facts.upsert_fact(ProjectFact::EmbeddingRecord(stale_record));
    let store = InMemoryFactStore::load(facts);
    let query = local_hash_embedding("portable structure policy");

    assert!(store
        .semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 10)
        .is_empty());
}

#[test]
fn semantic_search_matches_embedding_records_by_generation() {
    let old_generation = FactGeneration::new("snapshot-1");
    let new_generation = FactGeneration::new("snapshot-2");
    let old_chunk = test_chunk(&old_generation, "portable", "Archived release policy");
    let new_chunk = test_chunk(&new_generation, "portable", "Portable structure policy");
    let new_record = local_hash_embedding_record(&new_chunk);

    let mut facts = FactSet::default();
    facts.upsert_fact(ProjectFact::SearchChunk(old_chunk));
    facts.upsert_fact(ProjectFact::SearchChunk(new_chunk));
    facts.upsert_fact(ProjectFact::EmbeddingRecord(new_record));
    let store = InMemoryFactStore::load(facts);
    let query = local_hash_embedding("portable structure policy");
    let hits = store.semantic_search(&query, LOCAL_HASH_EMBEDDING_PROVIDER, 10);

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].chunk.generation.id, "snapshot-2");
}

fn semantic_fixture(with_embeddings: bool) -> FactSet {
    let mut ingestor = FactIngestor::new("semantic-test");
    let generation = FactGeneration::new("semantic-test");
    let mut facts = FactSet::default();
    add_chunk(
        &mut facts,
        &generation,
        "portable",
        "Portable structure policy for local project intelligence",
    );
    add_chunk(
        &mut facts,
        &generation,
        "release",
        "Release policy checks for archive evidence",
    );

    for fact in facts.facts {
        if let ProjectFact::SearchChunk(chunk) = fact {
            ingestor.add_search_chunk(chunk);
        }
    }

    if with_embeddings {
        ingestor.ingest_local_semantic_embeddings();
    }
    ingestor.finish()
}

fn add_chunk(facts: &mut FactSet, generation: &FactGeneration, id: &str, text: &str) {
    facts.upsert_fact(ProjectFact::SearchChunk(test_chunk(generation, id, text)));
}

fn test_chunk(generation: &FactGeneration, id: &str, text: &str) -> SearchChunk {
    SearchChunk {
        id: FactId::from_parts("search_chunk", &format!("goal:{id}")),
        generation: generation.clone(),
        origin: FactOrigin::Derived,
        source_id: model_instance_id("goals", id),
        text: text.to_string(),
    }
}
