# Project Intelligence Fact Model

## Goal

Execute `docs/goals/assura-project-intelligence-fact-model.md` as the fourth
successor goal in the Project Intelligence Runtime program. The task defines a
storage-independent fact contract and fixture-backed ingestion proof that turns
current models, repository resources, Markdown documents, model instances,
diagnostics, safe fixes, relation edges, and optional code-symbol references
into deterministic graph-shaped facts.

## Requirements

- Define public fact model types for the required node and edge categories:
  model definitions, fields, relationships, path scopes, resources, Markdown
  documents, Markdown sections, model instances, diagnostics, safe fixes,
  relationship edges, search chunks, embedding records, code symbols, and symbol
  references.
- Keep facts storage-independent. No Grafeo, redb, SQLite, Tantivy, vector
  store, daemon, or external provider may be required by this slice.
- Provide deterministic fact IDs and edge IDs that stay stable across unchanged
  runs.
- Provide generation or snapshot semantics so a later ingest can replace facts
  from the same source without relying on a database.
- Ingest existing content runtime fixture data into source facts for resources,
  Markdown documents, Markdown sections, model instances, model metadata, path
  scopes, relationship definitions, and relationship edges.
- Ingest validation output into diagnostic facts, and produce safe-fix facts for
  at least the supported Markdown trailing-spaces fix.
- Represent unresolved code-symbol references without requiring a code-symbol
  provider.
- Document source facts versus derived facts in repository docs.

## Acceptance Criteria

- [ ] `cargo test project_intelligence --quiet` covers deterministic fact IDs,
  replacement semantics, model/resource/Markdown/instance/diagnostic/safe-fix
  ingestion, relation edges, and unresolved code-symbol references.
- [ ] `cargo test --test content_runtime_validation --quiet` still passes.
- [ ] `cargo run --quiet -- check --format json .` remains clean.
- [ ] Public fact model structs/enums have rustdoc.
- [ ] The fact model can be used without selecting a storage backend.
- [ ] Documentation explains source facts, derived facts, and what later
  graph/search goals are expected to add.

## Definition Of Done

- The goal file has progress evidence for planning, implementation, review, and
  validation.
- An independent review checks deterministic IDs, source/derived separation,
  optional code-symbol facts, and precise diagnostic/safe-fix targets.
- Focused tests and self-check pass.
- The Trellis task can be archived after the goal is marked completed.

## Technical Approach

Use a small `src/intelligence` module or equivalent library surface rather than
threading fact behavior into CLI formatting. Keep the model as typed Rust data
with stable string IDs and simple ingest builders. Build ingestion from existing
content runtime and check-report surfaces where possible, but keep the
interface narrow enough that later graph/search storage can consume the facts
without changing validation behavior.

## Decision (ADR-lite)

Context: Later Project Intelligence Runtime goals need a common graph-shaped
contract before backend, query, search, semantic, code-symbol, daemon, LSP, or
MCP work can safely proceed.

Decision: Implement a storage-independent fact contract and fixture ingestion
proof first. Treat search chunks, embeddings, code symbols, and symbol
references as fact categories now, but do not require providers or stores.

Consequences: This may duplicate a small amount of projection logic from
existing validation/reporting paths, but it prevents premature database
coupling and gives later goals deterministic IDs to index.

## Out Of Scope

- Backend selection or benchmark decision.
- Public query/search CLI.
- Semantic embedding model integration.
- Code-symbol provider integration.
- Long-running daemon, LSP, MCP, or editor surface implementation.

## Technical Notes

- Active goal: `docs/goals/assura-project-intelligence-fact-model.md`
- Master program: `docs/goals/assura-project-intelligence-runtime-program.md`
- Current content runtime modules: `src/content_repository/`
- Current Markdown validation/fix modules: `src/cli/check/markdown.rs` and
  `src/cli/check/markdown_fix.rs`
- Validation commands from the goal:
  - `cargo fmt --check`
  - `cargo test project_intelligence --quiet`
  - `cargo test --test content_runtime_validation --quiet`
  - `cargo run --quiet -- check --format json .`
  - `git diff --check`
