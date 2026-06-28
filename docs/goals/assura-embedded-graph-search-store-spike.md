---
id: goal-assura-embedded-graph-search-store-spike
type: goal
title: Assura embedded graph search store spike
status: completed
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-fact-model.md
  - docs/analysis/2026-06-28-content-runtime-index-performance.md
  - benches/
---

# Assura Embedded Graph Search Store Spike

## Objective

Select or reject an embedded graph/search storage approach for Assura project
intelligence using measured evidence on Assura fact-model workloads.

## Current Gap

The content runtime currently uses an ephemeral single-walk in-memory index for
validation. That is appropriate for `assura check`, but it does not answer the
next product need: local graph traversal, repeated queries, text search,
optional vector search, incremental fact replacement, and low-resource
persistence.

## Candidate Direction

Treat Grafeo as the first serious candidate to verify because the research
context identified it as promising for embedded graph/search. Treat redb or
SQLite plus Tantivy and Assura-owned in-memory indexes as the lean fallback.

All candidate claims must be reverified against current upstream state during
execution.

## Scope

- Build a project intelligence fixture repository with modeled collections,
  Markdown sections, diagnostics, broken links, BDD/scenario-style records,
  code-symbol references, and optional embedding records.
- Load the normalized fact model into Grafeo if current APIs support the
  required embedded shape.
- Load the same facts into a lean fallback prototype using redb or SQLite for
  durable facts and Tantivy for full-text search.
- Measure memory, ingestion time, incremental replacement, relation traversal,
  path-scope queries, text search, and optional vector or hybrid search.
- Compare query ergonomics and operational complexity.
- Record a decision: adopt, defer, reject, or use fallback.

## Non-Goals

- No production graph store integration without measured evidence.
- No standalone service requirement for normal Assura operation.
- No semantic correctness claims from vector search.
- No code-intelligence provider implementation.
- No docs-site IA work.

## Definition Of Done

- A decision record compares Grafeo and the lean fallback using the same fact
  fixture and benchmark scenarios.
- Benchmarks include cold load, warm query, incremental update, missing-target
  traversal, path-scope query, text search, and memory footprint.
- The selected path states what is production-ready now, what remains
  experimental, and what must be rechecked before release.
- If no backend is selected, the fallback path remains explicit and executable.
- The decision does not make a database the canonical source of repository
  truth; checked files remain canonical.

## Validation Commands

```bash
cargo fmt --check
cargo test project_intelligence --quiet
cargo bench --bench project_intelligence -- --noplot
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R1: Confirm every candidate used the same fixture and query set.
- R2: Confirm memory and incremental update behavior were measured, not
  inferred.
- R3: Confirm optional vector/search features are not required for validation.
- R4: Confirm the selected path is local and embedded for core use.

## Reviewer Blocking Criteria

Block if the spike selects a backend without benchmark evidence, requires a
standalone service for normal use, stores canonical repository state outside
the repo, or skips the lean fallback comparison.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-28 | Started as the fifth Project Intelligence Runtime successor after the fact-model goal completed and archived. Created Trellis task `.trellis/tasks/06-28-embedded-graph-search-store-spike`, refreshed roadmap routing, and recorded current candidate-surface research for Grafeo, redb, SQLite/rusqlite, and Tantivy. | `.trellis/tasks/06-28-embedded-graph-search-store-spike/prd.md`; `.trellis/tasks/06-28-embedded-graph-search-store-spike/research/candidate-store-surfaces.md`; `.trellis/spec/assura/roadmap.md`; `python3 ./.trellis/scripts/task.py validate 06-28-embedded-graph-search-store-spike`; `cargo run --quiet -- check --format json .`. |
| 2026-06-28 | Implemented and measured the executable Assura-owned in-memory fact-store fallback for graph/search query development. Current Grafeo, redb, and Tantivy releases exceed Assura's declared Rust 1.70 MSRV, so external embedded backend adoption is deferred while the in-memory fallback remains available for the next query/search CLI goal. | `src/intelligence/store.rs`; `tests/project_intelligence_store_spike_tests.rs`; `benches/project_intelligence.rs`; `docs/analysis/2026-06-28-project-intelligence-store-spike.md`; `cargo bench --bench project_intelligence -- --noplot`. |
| 2026-06-28 | Closed review findings by making missing relationship targets generation-aware and index-backed, pinning the benchmark-shaped serialized footprint, clarifying that external candidates were screened at release gates instead of fixture-benchmarked, and completing independent review with no blockers. | `cargo fmt --check`; `cargo test --test project_intelligence_store_spike_tests --quiet`; `cargo test project_intelligence --quiet`; `cargo bench --bench project_intelligence -- --noplot`; `cargo run --quiet -- check --format json .`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`; review agent `019f1056-1fa3-75c2-8710-46b80e7a955f`. |
