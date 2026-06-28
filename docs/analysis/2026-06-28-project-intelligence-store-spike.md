---
title: Project Intelligence Store Spike
status: completed
---

# Project Intelligence Store Spike

## Decision

Defer Grafeo, redb, and Tantivy as direct Assura dependencies for the current
Project Intelligence Runtime branch. Keep an Assura-owned in-memory fact store
as the executable fallback for the next query/search slice, and re-evaluate
external embedded storage after Assura either raises MSRV or identifies
compatible releases.

This decision does not make any store canonical. Repository files remain the
source of truth; `FactSet` generations remain rebuildable from checked files
and validation output.

## Candidate Evidence

The external candidates were screened against the same intended fact fixture
and query set, but current direct-dependency adoption stopped at release gates
before loader implementation: the current Grafeo, redb, and Tantivy releases
all exceed Assura's declared MSRV, and SQLite/rusqlite needs a separate native
dependency and packaging review. The executable benchmark therefore measures
the Assura-owned fallback as the baseline all future candidates must beat with
the same scenarios.

| Candidate | Current upstream evidence | Decision |
| --- | --- | --- |
| Grafeo `0.5.42` | `cargo info grafeo` reports an embedded graph database package; `cargo info grafeo-engine@0.5.42` reports `rust-version: 1.91.1`. | Deferred. It may be a strong future candidate, but it is not compatible with Assura's declared `rust-version = "1.70.0"` today. |
| redb `4.1.0` | `cargo info redb@4.1.0` reports a pure-Rust embedded database with `rust-version: 1.89`. | Deferred as the durable fact-table fallback until MSRV changes or an older acceptable release is selected with explicit risk. |
| Tantivy `0.26.1` | `cargo info tantivy@0.26.1` reports `rust-version: 1.86`. | Deferred as the text-search component until MSRV changes or an older acceptable release is selected. |
| SQLite / rusqlite `0.40.1` | `cargo info rusqlite@0.40.1 --verbose` shows a current SQLite wrapper with many feature options and native/bundled dependency choices. | Not selected in this slice. It remains a fallback candidate, but selecting it needs a separate native dependency and packaging review. |
| Assura in-memory fact store | Implemented in `src/intelligence/store.rs` and benchmarked in `benches/project_intelligence.rs`. | Accepted as the executable fallback for local graph/search query development while external embedded stores are deferred. |

## Benchmark Fixture

The benchmark fixture uses the normalized fact model and generates:

- 240 modeled goal instances;
- 240 spec instances;
- 240 scenario-style records;
- path scopes for goals, specs, and scenarios;
- search chunks for goal text search;
- relationship edges, including one unresolved target;
- unresolved code-symbol references.

The 240-goal benchmark fixture serializes to 496,203 bytes through
`InMemoryFactStore::stats()`. The focused test fixture uses the same shape at
smaller scale and adds an extra stale-target regression edge.

## Local Benchmark Evidence

Command:

```bash
cargo bench --bench project_intelligence -- --noplot
```

Local environment:

```text
rustc 1.94.1 (e408947bf 2026-03-25)
cargo 1.94.1 (29ea6fb6a 2026-03-24)
```

Benchmark group:
`project_intelligence/store_spike` with 240 generated goal/spec/scenario
records and 20 Criterion samples.

| Scenario | Median estimate |
| --- | ---: |
| `assura_in_memory/cold_load` | 3.4682 ms |
| `assura_in_memory/warm_missing_target_traversal` | 62.124 ns |
| `assura_in_memory/warm_path_scope_query` | 3.1883 us |
| `assura_in_memory/warm_text_search` | 26.055 us |
| `assura_in_memory/incremental_replace_generation` | 36.079 ms |
| `assura_in_memory/serialized_footprint_bytes` | 585.96 us to recompute the serialized footprint byte count |

## Interpretation

The in-memory fallback is good enough to unblock the next CLI/query successor:
load, traversal, path-scope matching, keyword search, and generation replacement
are all executable with the completed fact model and no new runtime dependency.

It is not a durable graph/search backend. It should be treated as a baseline
and product integration surface, not as a final storage decision. Missing-target
traversal is precomputed during index rebuild so repeated queries stay cheap.
The expensive row is incremental replacement because the fallback rebuilds all
indexes after `FactSet::replace_generation`; later durable candidates should
specifically measure partial index updates against this baseline.

`serialized_footprint_bytes` is not a resident-memory measurement. It is the
deterministic serialized size of the retained fact set plus the cost of
recomputing that size through JSON serialization. Future durable backend
comparisons should add RSS or allocator-level memory measurement when selecting
production storage.

## Release Recheck Requirements

Before selecting an external backend:

1. Re-run `cargo info` for Grafeo, redb, Tantivy, and rusqlite.
2. Verify MSRV compatibility against `Cargo.toml`.
3. Compile a candidate loader behind a dev-only or experimental feature.
4. Run the same benchmark scenarios from `benches/project_intelligence.rs`.
5. Document native dependency, binary-size, and packaging impact.

## Validation

```bash
cargo fmt --check
cargo test --test project_intelligence_store_spike_tests --quiet
cargo test project_intelligence --quiet
cargo bench --bench project_intelligence -- --noplot
cargo run --quiet -- check --format json .
cargo clippy --workspace --all-targets --all-features -- -D warnings
git diff --check
```

Independent review agent `019f1056-1fa3-75c2-8710-46b80e7a955f` reported no
blockers after checking the dependency-gate decision, in-memory fallback
surface, missing-target generation handling, benchmark coverage, and validation
evidence.
