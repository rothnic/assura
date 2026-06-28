---
id: goal-assura-project-intelligence-fact-model
type: goal
title: Assura project intelligence fact model
status: completed
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/goals/assura-rust-markdown-validation-and-fixing.md
  - src/content_repository/
  - src/markdown/
---

# Assura Project Intelligence Fact Model

## Objective

Define the normalized fact model and ingestion contract that converts
LinkML-style collection models, repository resources, Markdown documents,
frontmatter records, model instances, diagnostics, safe fixes, and relations
into a stable graph-shaped representation.

## Current Gap

Content runtime validation already creates object snapshots and relation edges,
but there is no product-level fact model that can power graph queries, search,
semantic indexing, code-symbol enrichment, daemon surfaces, or agent APIs.

## User Certainty Bar

A developer or agent should be able to ask Assura what facts it knows about a
repo and get stable node and edge identities, source locations, validation
state, and relationships without reverse-engineering internal validation
structures.

## Scope

- Define core fact types:
  `ModelDefinition`, `FieldDefinition`, `RelationshipDefinition`,
  `PathScope`, `Resource`, `MarkdownDocument`, `MarkdownSection`,
  `ModelInstance`, `Diagnostic`, `SafeFix`, `RelationshipEdge`,
  `SearchChunk`, `EmbeddingRecord`, `CodeSymbol`, and `SymbolRef`.
- Define deterministic IDs for facts and edges.
- Define generation or snapshot replacement semantics for incremental updates.
- Ingest model metadata from runtime schema artifacts and content runtime
  config.
- Ingest instance facts from Markdown frontmatter, JSON, YAML, and JSONL
  collections.
- Ingest Markdown document and section facts from the Markdown validation path.
- Ingest diagnostics and safe-fix proposals from validation output.
- Materialize only high-value derived edges required by common queries.
- Keep storage backend choices out of this goal except for interface needs.

## Non-Goals

- No Grafeo, redb, SQLite, Tantivy, or vector backend selection.
- No public query CLI beyond debug/inspection output needed for proof.
- No code provider integration except placeholder fact types.
- No semantic embedding model integration.

## Definition Of Done

- A fact model module or equivalent contract defines all required node and edge
  types with rustdoc.
- Fixture ingestion produces deterministic facts for models, resources,
  Markdown sections, model instances, relation edges, diagnostics, and safe
  fixes.
- Tests prove stable IDs across unchanged runs and replacement of facts by
  generation or snapshot.
- The fact model can represent unresolved code-symbol references without a code
  provider.
- Docs explain which facts are source facts and which are derived facts.
- No storage backend is made mandatory by this contract.

## Validation Commands

```bash
cargo fmt --check
cargo test project_intelligence --quiet
cargo test --test content_runtime_validation --quiet
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R1: Confirm fact IDs are deterministic and stable enough for incremental
  indexing.
- R2: Confirm the model separates source facts from derived facts.
- R3: Confirm optional code-symbol facts do not require a provider.
- R4: Confirm diagnostics and safe fixes can target precise resources or model
  instances.

## Reviewer Blocking Criteria

Block if the fact model is tied to one database, if IDs are unstable without a
documented reason, if code intelligence is mandatory, or if diagnostics cannot
be attached back to source files and fields.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-28 | Started as the fourth Project Intelligence Runtime successor goal after Documentation IA completed local review. Created and activated Trellis task `.trellis/tasks/06-28-project-intelligence-fact-model`, refreshed roadmap routing to this task, and added the PRD/context files needed before implementation. | `python3 ./.trellis/scripts/workflow_gate.py --platform codex`; `.trellis/tasks/06-28-project-intelligence-fact-model/prd.md`; `.trellis/spec/assura/roadmap.md`; `cargo run --quiet -- check --format json .`; `git diff --check`. |
| 2026-06-28 | Implemented the storage-independent project intelligence fact contract with deterministic fact and edge IDs, generation replacement, content-runtime model/resource/Markdown/instance/relation ingestion, diagnostics, safe fixes, search chunks, and unresolved symbol references. Independent review found diagnostic targeting and safe-fix proof gaps; follow-up fixes now target model instances only when unambiguous, fall back to resources for multi-record ambiguity, assert safe-fix diagnostic/location linkage, prove IDs are stable across generation labels, preserve same fact/edge IDs across different generations, keep model definitions distinct by collection/class binding, expose inferred relation candidate collections, and leave ambiguous multi-target relation edges unresolved. | Review agents `019f0fb7-1933-7c20-b422-23899f9a1566` and `019f0ff1-b758-70c2-b8d6-cf2ea5b4db2f`; `cargo fmt --check`; `cargo test --test project_intelligence_fact_model_tests --quiet`; `cargo test project_intelligence --quiet`; `cargo test --test content_runtime_validation --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `git diff --check`. |
