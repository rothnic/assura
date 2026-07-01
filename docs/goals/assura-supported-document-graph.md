---
id: goal-assura-supported-document-graph
type: goal
title: Assura supported document graph
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-beta-content-collections-querying.md
  - ./assura-code-doc-reference-validation.md
  - ./assura-project-intelligence-runtime-program.md
  - ./assura-project-intelligence-usability-program.md
  - ../project-intelligence-facts.md
---

# Assura Supported Document Graph

## Objective

Make document graph support fully supported across modeled content validation,
repository facts, searching, querying, relation checks, and bounded agent
context so users can rely on Assura as a local document intelligence layer.

## Current Gap

Beta proved the core pieces: content models, collection validation, Markdown
sections, repository-reference facts, content search, graph expansion,
affected-reference queries, and agent context packs. Post-beta needs one
support-grade contract that ties these pieces together, removes ambiguity
between supported graph behavior and experimental candidate enrichment, and
adds real-project proof for the complete workflow.

## Scope

- Define the supported document graph contract for nodes, edges, diagnostics,
  content model instances, Markdown sections, repository references, and
  query/search outputs.
- Validate frontmatter, IDs, relations, path scopes, missing relation targets,
  duplicate/ambiguous references, and cyclic relation behavior through the
  content runtime.
- Support deterministic content search, collection queries, relation queries,
  bounded graph expansion, affected-source and affected-target reference
  queries, and agent context packs.
- Keep semantic and code-symbol results as optional candidate enrichment unless
  a later goal promotes a specific surface with proof.
- Add support docs, fixtures, and target-state checks that prevent unsupported
  graph claims from drifting back into public docs.
- Prove the full workflow on Assura and at least one realistic non-Assura
  fixture package.

## Non-Goals

- No hosted graph service.
- No remote embedding requirement.
- No semantic search correctness claim.
- No editor or daemon-specific behavior beyond shared contracts and examples.

## Definition Of Done

- Public docs describe one supported document graph workflow from model
  definition through validation, search, query, graph expansion, and affected
  references.
- CLI JSON schemas and text output are stable enough for local agents.
- Real-project proof covers valid graph queries, invalid content diagnostics,
  stale references, relation drift, and bounded context packs.
- Support policy and release surfaces classify supported graph behavior
  separately from experimental semantic/code-symbol enrichment.
- Target-state checks catch unsupported graph, search, or content validation
  claims.
- Independent review finds no unsupported dependency on hosted services,
  editor plugins, daemon state, semantic ranking, or code-symbol providers.

## Validation Commands

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test project_intelligence --quiet
cargo test --test content_query_cli --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the supported graph contract covers validation, search, query,
  graph expansion, relation diagnostics, and affected-reference questions.
- R2: Confirm optional candidate enrichment cannot decide validation truth.
- R3: Confirm agents can consume bounded graph context without unbounded docs.
- R4: Confirm docs and support policy do not imply hosted, daemon, or editor
  prerequisites.

## Reviewer Blocking Criteria

Block if document graph support is only a bundle of demos, if search ranking
decides correctness, if content-model diagnostics do not connect to query
outputs, if repository-reference facts are missing from graph context, or if a
remote service/provider becomes required for the supported path.
