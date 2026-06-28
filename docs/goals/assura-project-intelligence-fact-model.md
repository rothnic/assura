---
id: goal-assura-project-intelligence-fact-model
type: goal
title: Assura project intelligence fact model
status: planned
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
