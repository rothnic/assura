# Embedded Graph Search Store Spike

## Goal

Execute `docs/goals/assura-embedded-graph-search-store-spike.md` as the fifth
successor in the Project Intelligence Runtime program. The task must decide,
with measured evidence, whether Assura should adopt Grafeo, use a lean embedded
fallback such as redb or SQLite plus Tantivy, or defer backend selection for
project intelligence graph/search workloads.

## What I Already Know

- The fact-model successor is complete and committed on
  `codex/project-intelligence-fact-model`.
- The current content runtime performance decision accepts an ephemeral
  single-walk in-memory file index for normal `assura check` and explicitly
  avoids a persistent cache/database for that release slice.
- The next product gap is repeated local graph traversal, text search,
  optional vector search, incremental fact replacement, and low-resource
  persistence over normalized project intelligence facts.
- The goal requires current upstream/API verification before treating Grafeo,
  redb, SQLite, or Tantivy claims as true.

## Revalidation Result

`valid`: no current Assura artifact selects a storage/search backend for the new
project intelligence fact model. Existing benchmarks cover content runtime and
older in-memory graph behavior, not fact-model load/update/query/search
workloads.

## Requirements

- Build or reuse a project intelligence fact fixture that includes modeled
  collections, Markdown documents/sections, diagnostics, safe fixes,
  relationship edges, missing/ambiguous targets, search chunks, optional
  embedding records, and unresolved code-symbol references.
- Research current Grafeo APIs and constraints before implementing a candidate
  loader.
- Implement a comparable lean fallback prototype using local embedded pieces
  only, such as redb or SQLite for durable facts plus Tantivy for text search,
  when the current dependency/API evidence supports it.
- Measure the same scenarios for every candidate: cold load, warm query,
  incremental generation replacement, missing-target traversal, path-scope
  query, keyword/text search, and memory footprint.
- Record a decision document with adopt/defer/reject/fallback outcome and the
  exact evidence behind it.
- Keep repository files as canonical state; no candidate may become the source
  of truth for modeled content.

## Acceptance Criteria

- [x] A checked decision record screens Grafeo and lean embedded fallback
  candidates against the same intended fixture/query set, records why external
  candidates stop before loader implementation, and benchmarks the executable
  Assura fallback against that fixture.
- [x] Benchmarks or benchmark-like tests measure cold load, warm query,
  incremental replacement, traversal, path-scope query, text search, and memory.
- [x] The selected or deferred path states production readiness, experimental
  gaps, and release recheck requirements.
- [x] Optional vector or semantic search remains non-authoritative for
  validation correctness.
- [x] `cargo fmt --check`, `cargo test project_intelligence --quiet`,
  `cargo bench --bench project_intelligence -- --noplot`,
  `cargo run --quiet -- check --format json .`, and `git diff --check` pass or
  have explicit documented blockers.

## Out Of Scope

- Productionizing a graph/search backend without benchmark evidence.
- Requiring a standalone service for normal Assura use.
- Implementing semantic model integration or code-symbol providers.
- Changing the fact model contract unless the spike exposes a blocker that must
  be fixed first.

## Technical Notes

- Active goal: `docs/goals/assura-embedded-graph-search-store-spike.md`.
- Master program:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Completed dependency:
  `docs/goals/assura-project-intelligence-fact-model.md`.
- Existing fact module: `src/intelligence/facts/`.
- Existing benchmarks: `benches/content_runtime.rs`,
  `benches/graph_benchmarks.rs`, and benchmark guidance in
  `benches/README.md`.
- Existing performance decision:
  `docs/analysis/2026-06-28-content-runtime-index-performance.md`.

## Research References

- `research/candidate-store-surfaces.md` records current upstream/API surfaces
  for Grafeo, redb, SQLite/rusqlite, and Tantivy and recommends the initial
  spike shape.
