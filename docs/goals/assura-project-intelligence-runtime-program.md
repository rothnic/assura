---
id: goal-assura-project-intelligence-runtime-program
type: goal
title: Assura project intelligence runtime program
status: completed
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/goals/assura-content-model-source-of-truth.md
  - docs/goals/assura-rust-markdown-validation-and-fixing.md
  - docs/goals/assura-project-intelligence-fact-model.md
  - docs/goals/assura-embedded-graph-search-store-spike.md
  - docs/goals/assura-content-query-and-search-cli.md
  - docs/goals/assura-local-semantic-search.md
  - docs/goals/assura-code-symbol-enrichment.md
  - docs/goals/assura-project-intelligence-agent-surfaces.md
  - docs/goals/assura-documentation-ia-project-intelligence.md
---

# Assura Project Intelligence Runtime Program

## Objective

Move Assura from a structure-first validator with a repo-native content runtime
into a Rust-native project intelligence and validation runtime that agents can
use to validate, query, update, and understand repository knowledge.

This program keeps Assura's core local and embedded. External code
intelligence tools, standalone services, and heavyweight databases may enrich
the product later, but the core must remain useful with ordinary repository
files, Rust validation, modeled collections, Markdown validation, local graph
facts, and local search.

## Product Shape

The end state is layered:

1. Structure validation comparable to LS-Lint for repository shape.
2. Rust-native Markdown linting, formatting, frontmatter extraction, and
   Assura-owned heading hierarchy validation.
3. LinkML-style modeled repository collections with one source of truth for
   required fields, optional fields, IDs, relations, and path scopes.
4. A normalized project fact graph covering models, resources, Markdown
   documents, sections, instances, diagnostics, safe fixes, and relations.
5. Embedded graph, text, and optional vector search over modeled facts.
6. Optional code-symbol enrichment from native or provider-backed sources.
7. CLI, daemon, LSP, and MCP surfaces that reuse the same core APIs.

## Current Gap

The repo-native content runtime goal completed the modeling and validation
foundation, but the broader project intelligence layer is not yet defined as an
execution program. The missing pieces are:

- one correct source of truth for frontmatter required fields;
- Rust-native Markdown lint and fix integration;
- a documented docs-site information architecture for the layered product;
- a normalized project fact graph;
- an embedded graph/search backend decision;
- public query and search commands;
- local semantic search;
- optional code-symbol enrichment;
- long-running agent/editor surfaces that reuse the same query and validation
  APIs.

## Execution Sequence

Execute these goals in order unless a goal records a refreshed dependency
decision with evidence.

1. [Content Model Source Of Truth](./assura-content-model-source-of-truth.md)
   makes content runtime models own frontmatter required fields and removes
   duplicate Markdown-frontmatter validation paths. Exit when existing
   frontmatter checks route through modeled collections or are removed with
   docs and tests updated.
2. [Rust Markdown Validation And Fixing](./assura-rust-markdown-validation-and-fixing.md)
   selects and integrates Rust Markdown lint/fix tooling while keeping heading
   hierarchy model-aware. Exit when `assura check` reports Markdown lint
   diagnostics and a supported safe-fix operation applies deterministic
   Markdown fixes.
3. [Documentation IA Project Intelligence](./assura-documentation-ia-project-intelligence.md)
   presents the product as a layered validation and intelligence system instead
   of one buried example. Exit when website navigation and docs explain
   structure, Markdown, collections, graph/search, and optional code
   intelligence.
4. [Project Intelligence Fact Model](./assura-project-intelligence-fact-model.md)
   defines normalized facts and graph ingestion from models, repository files,
   Markdown, diagnostics, and safe fixes. Exit when fixtures prove resources,
   model instances, Markdown sections, diagnostics, and relation edges are
   generated deterministically.
5. [Embedded Graph Search Store Spike](./assura-embedded-graph-search-store-spike.md)
   compares Grafeo against a lean redb/SQLite plus Tantivy fallback under
   Assura workloads. Exit when a decision record selects or rejects a backend
   using measured memory, update, traversal, text, and vector evidence.
6. [Content Query And Search CLI](./assura-content-query-and-search-cli.md)
   exposes collection queries, relation queries, keyword search, and graph
   expansion. Exit when users can query modeled collections and find related
   docs, instances, and diagnostics from CLI output.
7. [Local Semantic Search](./assura-local-semantic-search.md) adds optional
   local embeddings over selected chunks without making semantic search
   correctness-critical. Exit when semantic search returns candidate nodes with
   graph expansion and deterministic validation state.
8. [Code Symbol Enrichment](./assura-code-symbol-enrichment.md) adds optional
   code facts and unresolved or resolved symbol edges without requiring
   external providers. Exit when modeled instances can link to code symbols from
   native baseline or imported provider facts.
9. [Project Intelligence Agent Surfaces](./assura-project-intelligence-agent-surfaces.md)
   layers daemon, LSP, MCP, and agent APIs on the same local validation and
   query core. Exit when agents and editors can request diagnostics, safe fixes,
   graph queries, and search through shared contracts.

## Program Definition Of Done

- Assura has exactly one supported way to model frontmatter fields for typed
  content collections.
- Markdown lint and fix behavior is Rust-native, benchmarked, and documented.
- Heading hierarchy remains Assura-owned and can express required and optional
  nested headings.
- Modeled collection instances become graph facts with stable node and edge
  IDs.
- The selected graph/search storage approach is justified by benchmark evidence
  on Assura fixtures and real-ish repositories.
- Query commands can answer missing-target, relation, path-scope, diagnostic,
  keyword, and graph-expansion questions.
- Semantic search is optional and only contributes candidate context; it does
  not decide correctness.
- Code intelligence is optional and provider-based; Assura works without CKB,
  LIP, Codanna, CQS, Glean, SCIP, LSP, or a standalone service.
- The docs site teaches the layered product path from LS-Lint-like validation
  through project intelligence.
- Each goal has independent review evidence and clear validation commands.

## Testing Strategy

Use smart staged validation:

- run the narrow test or fixture for the changed behavior during development;
- run docs checks when only docs changed;
- run benchmark and performance gates only for runtime, indexing, search, or
  notation cost changes;
- run broad workspace tests, docs, evidence, and self-check at PR or program
  readiness boundaries.

Do not spend time rerunning the whole suite after every small doc or fixture
edit unless the change touches shared runtime behavior.

## Validation Commands

Planning-only changes to this program should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

Implementation goals add their own narrow proof gates. The final program gate
must include the complete validation chain from every completed goal.

## Review Tasks

- R1: Confirm each successor goal is independently executable and ordered by
  dependency.
- R2: Confirm no goal makes an optional provider required for core Assura.
- R3: Confirm frontmatter model ownership removes duplicate validation paths.
- R4: Confirm graph/search work is benchmark-driven and not a premature
  database commitment.
- R5: Confirm the docs IA goal explains the layered product coherently.

## Reviewer Blocking Criteria

Block execution if the program keeps duplicate frontmatter validation surfaces,
requires a standalone service for normal validation, treats semantic search as
validation truth, or starts code intelligence before the core fact graph and
query path exist.

## Progress Log

- 2026-06-29: Final handoff review. Iteration count: final completion audit
  after nine successor goals. Context health: goal tracker reports 6,715,062
  tokens used with no remaining budget exposed; current context is compacted
  but includes the pending audit state, prior validation results, review
  findings, and commit list. Reviewed the master goal, final audit task,
  roadmap, dirty tree, and validation evidence. No new project skill is needed:
  the existing `assura-goal-execution`, `assura-goal-validation`,
  `assura-local-build`, and `assura-performance-reporting` skills cover the
  reusable workflow surfaces discovered during the program.

## Completion Evidence

| Date | Evidence |
| --- | --- |
| 2026-06-29 | Completed final audit in `.trellis/tasks/06-29-06-29-project-intelligence-runtime-completion-audit`. All nine successors in the execution sequence have completed goal evidence and archived Trellis tasks. The audit fixed stale `status: planned` metadata in `docs/goals/assura-content-model-source-of-truth.md` and mapped every Program Definition Of Done item to current tests, docs, support matrices, command output, benchmark evidence, and review records. Final validation included `cargo test --workspace --all-targets --all-features --quiet`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`, `git diff --check`, `cargo run --quiet -- check --format json .`, `cargo run --quiet -- check --format agent --agent codex .`, `cargo xtask docs`, and `cargo xtask evidence`. |
