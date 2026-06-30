//! Local semantic-search primitives for project intelligence facts.

use super::facts::{EmbeddingRecord, FactId, FactOrigin, SearchChunk};
use crate::stable_hash::stable_hash;

/// Built-in no-dependency embedding baseline for local candidate retrieval.
pub const LOCAL_HASH_EMBEDDING_PROVIDER: &str = "local-hash-embedding-v1";

/// Vector dimensions for the local hash embedding baseline.
pub const LOCAL_HASH_EMBEDDING_DIMENSIONS: usize = 64;

/// Build an embedding record for a search chunk using the local hash baseline.
pub fn local_hash_embedding_record(chunk: &SearchChunk) -> EmbeddingRecord {
    let text_hash = semantic_text_hash(&chunk.text);
    EmbeddingRecord {
        id: FactId::from_parts(
            "embedding",
            &format!("{}:{LOCAL_HASH_EMBEDDING_PROVIDER}:{text_hash}", chunk.id),
        ),
        generation: chunk.generation.clone(),
        origin: FactOrigin::Derived,
        chunk_id: chunk.id.clone(),
        provider: LOCAL_HASH_EMBEDDING_PROVIDER.to_string(),
        text_hash,
        dimensions: LOCAL_HASH_EMBEDDING_DIMENSIONS,
        vector: local_hash_embedding(&chunk.text),
    }
}

/// Deterministic text hash used to invalidate stale embedding records.
pub fn semantic_text_hash(text: &str) -> String {
    format!("{:016x}", stable_hash(text.as_bytes()))
}

/// Embed text with a deterministic token hashing baseline.
pub fn local_hash_embedding(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0; LOCAL_HASH_EMBEDDING_DIMENSIONS];
    for token in semantic_tokens(text) {
        let hash = stable_hash(token.as_bytes());
        let index = (hash as usize) % LOCAL_HASH_EMBEDDING_DIMENSIONS;
        let sign = if hash & 1 == 0 { 1.0 } else { -1.0 };
        vector[index] += sign;
    }
    normalize_vector(vector)
}

/// Cosine similarity for normalized or unnormalized vectors.
pub fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    let len = left.len().min(right.len());
    if len == 0 {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut left_norm = 0.0;
    let mut right_norm = 0.0;
    for index in 0..len {
        dot += left[index] * right[index];
        left_norm += left[index] * left[index];
        right_norm += right[index] * right[index];
    }

    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm.sqrt() * right_norm.sqrt())
    }
}

fn semantic_tokens(text: &str) -> Vec<String> {
    text.split(|ch: char| !ch.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn normalize_vector(mut vector: Vec<f32>) -> Vec<f32> {
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intelligence::{FactGeneration, FactOrigin};

    #[test]
    fn local_hash_embedding_is_deterministic_and_normalized() {
        let first = local_hash_embedding("Portable structure policy");
        let second = local_hash_embedding("Portable structure policy");

        assert_eq!(first, second);
        assert_eq!(first.len(), LOCAL_HASH_EMBEDDING_DIMENSIONS);
        let norm = first.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.0001);
    }

    #[test]
    fn embedding_record_tracks_source_hash_and_provider() {
        let chunk = SearchChunk {
            id: FactId::from_parts("search_chunk", "goal:portable"),
            generation: FactGeneration::new("test"),
            origin: FactOrigin::Derived,
            source_id: FactId::from_parts("instance", "goals:portable"),
            text: "Portable structure policy".to_string(),
        };

        let record = local_hash_embedding_record(&chunk);

        assert_eq!(record.chunk_id, chunk.id);
        assert_eq!(record.provider, LOCAL_HASH_EMBEDDING_PROVIDER);
        assert_eq!(record.text_hash, semantic_text_hash(&chunk.text));
        assert_eq!(record.dimensions, LOCAL_HASH_EMBEDDING_DIMENSIONS);
    }
}
