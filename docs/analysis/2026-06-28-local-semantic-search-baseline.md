---
id: analysis-2026-06-28-local-semantic-search-baseline
title: Local Semantic Search Baseline
status: active
created: 2026-06-28
related:
  - docs/goals/assura-local-semantic-search.md
  - docs/goals/assura-content-query-and-search-cli.md
---

# Local Semantic Search Baseline

This note records the first local semantic-search implementation slice.

## Decision

Use `local-hash-embedding-v1` as Assura's no-dependency baseline provider for
early local candidate retrieval. It creates deterministic 64-dimensional
vectors from normalized token hashes and stores them as `EmbeddingRecord`
facts attached to existing `SearchChunk` facts.

This is a baseline, not a final quality model. It proves the fact contract,
text-hash invalidation metadata, local vector indexing, ranking behavior, and
disabled-by-default posture before adding a heavier local embedding model.

## Evidence

| Criterion | Baseline |
| --- | --- |
| Size | No model artifact and no new dependency. |
| Speed | One pass over chunk tokens; no network or model startup. |
| Portability | Pure Rust, deterministic, platform-independent stable hashes. |
| Licensing | No external model license. |
| Update behavior | `EmbeddingRecord.text_hash` changes when source chunk text changes. |
| Quality | Lexical hash-vector candidate retrieval only; richer local embeddings remain future work. |

## Boundaries

- Scores are candidate ranking signals only.
- `assura check` validation truth is unchanged.
- Remote embedding services are not required.
- Public semantic CLI output is still a follow-up slice.

## Validation

```bash
cargo fmt --check
cargo test --test semantic_search_tests --quiet
cargo test semantic_search --quiet
cargo test project_intelligence --quiet
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
git diff --check
```
