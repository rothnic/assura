---
title: Supported document graph
status: active
---

# Supported Document Graph

## Goal

Execute the second child goal from
`docs/goals/assura-post-beta-capabilities-program.md`: promote Assura's local
document graph from a set of adjacent beta surfaces into one support-grade
workflow for content validation, querying, graph expansion, repository
references, and bounded agent context.

## Current State

- The self-config child goal is merged in PR #114 as `3aa17ea`.
- `assura content` already supports modeled collections, instances, show,
  search, missing relations, graph expansion, context packs, sessions, and
  repository-reference queries.
- Public support docs still classify repository-reference facts and
  `assura content references` as experimental.
- `assura content context-pack` includes diagnostics, model relations, search,
  missing relations, and safe-fix previews, but does not include direct
  repository-reference context for the object being handed to an agent.

## Requirements

- Preserve the parent program's final verification use case: users should be
  able to validate and query a local repository knowledge graph without a
  hosted service, daemon, editor extension, semantic ranking, or code-symbol
  provider.
- Add repository-reference context to bounded context packs so agents can see
  inbound and outbound doc/code references for a modeled object.
- Promote the supported graph contract in public docs while keeping semantic
  search and code-symbol surfaces classified as optional candidate enrichment.
- Prove the behavior on existing realistic fixtures with deterministic CLI
  tests.
- Update parent/child progress logs and roadmap routing.

## Acceptance Criteria

- [x] `assura content context-pack` JSON includes bounded inbound and outbound
      repository-reference context for object-mode packs.
- [x] Context-pack bounds report truncation/omission for repository-reference
      fields consistently with other pack sections.
- [x] Public support docs classify repository-reference facts and
      `assura content references` as supported parts of the document graph
      contract.
- [x] Semantic search and code-symbol outputs remain documented as experimental
      candidate enrichment and do not decide validation truth.
- [x] Validation passes:
      `cargo fmt --check`, `cargo test --test project_intelligence_context_pack --quiet`,
      `cargo test --test content_query_cli --quiet`,
      `cargo test --test repository_reference_graph_tests --quiet`,
      `cargo run --quiet -- check --format json .`, `cargo xtask target-state`,
      `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check`.
- [x] Independent review finds no unsupported dependency on hosted services,
      editor plugins, daemon state, semantic ranking, or code-symbol providers.

## Out Of Scope

- Implementing the true daemon process.
- Implementing VS Code marketplace support.
- Promoting semantic search or code-symbol candidate enrichment to validation
  truth.
- Replacing the existing content runtime or fact store.
