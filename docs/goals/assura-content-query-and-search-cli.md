---
id: goal-assura-content-query-and-search-cli
type: goal
title: Assura content query and search CLI
status: completed
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-fact-model.md
  - docs/goals/assura-embedded-graph-search-store-spike.md
  - src/cli/
---

# Assura Content Query And Search CLI

## Objective

Expose modeled collection queries, relation queries, keyword search, and graph
expansion through public Assura CLI commands backed by the project
intelligence fact model.

## Current Gap

`assura check` validates modeled collections, but users and agents do not yet
have public commands to query detected collections, find related objects, search
docs/specs/diagnostics, or inspect graph relationships.

## User Certainty Bar

An agent should be able to ask Assura questions such as "which requirements
lack scenarios?", "which goals reference this spec?", "find authentication
timeout", and "show related diagnostics and files" without scraping raw files
manually.

## Scope

- Define stable CLI commands for querying collection instances.
- Add keyword search over model instances, Markdown sections, diagnostics, and
  selected resource metadata.
- Add graph expansion from a selected node to related instances, documents,
  sections, diagnostics, and unresolved code-symbol references.
- Add relation-focused queries for missing references, ambiguous references,
  unresolved symbols, and required-doc gaps.
- Support JSON output for agents and concise text output for humans.
- Add fixtures that exercise collections, Markdown sections, diagnostics,
  relation edges, and search chunks.
- Keep semantic search and code provider enrichment out of the first query CLI.

## Non-Goals

- No vector embeddings in this goal.
- No LSP, MCP, or daemon API.
- No code provider implementation beyond facts already present.
- No arbitrary user query language until stable canned queries exist.

## Definition Of Done

- CLI commands can list collections and collection instances.
- CLI commands can show one instance with its source path, model type,
  outgoing relations, incoming relations, diagnostics, and related sections.
- Keyword search returns matching model instances, Markdown sections, and
  diagnostics with enough context for an agent to choose next steps.
- Graph expansion returns bounded related context with deterministic ordering.
- JSON output is covered by snapshot or structural tests.
- Docs include copy/paste examples for agent use.

## Validation Commands

```bash
cargo fmt --check
cargo test content_query --quiet
cargo run --quiet -- check --format json .
git diff --check
```

Add command-specific CLI fixture checks as commands are introduced.

## Review Tasks

- R1: Confirm query commands use the fact model rather than ad hoc file scans.
- R2: Confirm JSON output is stable enough for agents.
- R3: Confirm graph expansion is bounded and deterministic.
- R4: Confirm keyword search does not claim semantic meaning.

## Reviewer Blocking Criteria

Block if commands scrape files outside the fact ingestion path, if output is
not stable for agents, if graph expansion is unbounded, or if this goal
implements vector search before keyword search works.

## Progress Log

- 2026-06-28: Implemented `assura content` commands for collection listing,
  instance listing, instance show, keyword search, missing relations, and
  bounded graph expansion. The command path builds facts through
  `RepositoryModel`, `ContentRepository`, `FactIngestor`, and
  `InMemoryFactStore`.
- 2026-06-28: Added diagnostic search chunks for content runtime and structure
  diagnostics so keyword search can return diagnostic matches without semantic
  search or vector ranking.
- 2026-06-28: Added structural CLI fixture tests covering collections,
  instances, show, expand, search, missing relations, and diagnostic search.
- 2026-06-28: Independent review agent `019f10a5-8ca4-7a03-8cfb-55f8b10ca563`
  found one blocker: website examples used `.` with fixture IDs that do not
  exist in the root config. Updated docs to use copy/pasteable content runtime
  fixture paths; reviewer found no other blockers.
- 2026-06-28: Registered `assura content` and its first-slice subcommands in
  the checked command-surface contract, support matrix, support policy,
  compatibility matrix, and release notes.
- 2026-06-28: Validation passed:
  `cargo fmt --check`;
  `cargo test --test content_query_cli --quiet`;
  `cargo test content_query --quiet`;
  `cargo test project_intelligence --quiet`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo run --quiet -- check --format json .`;
  `cargo xtask docs`;
  `cargo xtask evidence`;
  `git diff --check`.
