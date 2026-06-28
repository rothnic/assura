---
id: goal-assura-local-semantic-search
type: goal
title: Assura local semantic search
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-content-query-and-search-cli.md
  - docs/goals/assura-embedded-graph-search-store-spike.md
  - docs/goals/assura-project-intelligence-fact-model.md
---

# Assura Local Semantic Search

## Objective

Add optional local semantic search over selected Assura facts so agents can
find relevant model instances, Markdown sections, scenarios, diagnostics, and
code-symbol summaries by meaning, then expand results through deterministic
graph relationships.

## Current Gap

The product direction calls for local semantic search, but Assura does not yet
have a chunking contract, embedding record model, local embedding provider
decision, vector index decision, or query surface.

## Principle

Semantic search enriches context. It must not decide correctness. Validation,
required fields, relation resolution, stale-doc decisions, and safe fixes stay
deterministic.

## Scope

- Define which facts become semantic chunks:
  model instances, Markdown sections, requirements, scenarios, diagnostics,
  safe-fix summaries, and optional code-symbol summaries.
- Define stable chunk IDs, source locations, hashes, model versions, and
  embedding metadata.
- Evaluate a small local embedding model suitable for low-resource operation.
- Decide whether vector search lives in the selected graph/search backend or a
  separate local vector index.
- Add semantic search CLI output that returns candidate facts plus graph
  expansion and validation state.
- Add cache invalidation when source chunks change.
- Make semantic search opt-in and gracefully disabled when no local model or
  vector index is configured.

## Non-Goals

- No remote embedding dependency for core operation.
- No semantic validation or semantic safe-fix correctness claims.
- No provider-specific code intelligence requirement.
- No broad natural-language agent planner.

## Definition Of Done

- Semantic chunks and embedding records are represented in the fact model.
- A local embedding provider decision records size, speed, quality,
  portability, and licensing evidence.
- Search results include source facts, scores, graph-expanded context, and
  deterministic validation state.
- Changed chunks are re-embedded or invalidated by hash.
- Semantic search can be disabled without affecting `assura check`.
- Docs explain limitations and the difference between semantic candidates and
  validation truth.

## Validation Commands

```bash
cargo fmt --check
cargo test semantic_search --quiet
cargo run --quiet -- check --format json .
git diff --check
```

Add benchmark commands for embedding and vector lookup once the provider is
selected.

## Review Tasks

- R1: Confirm the embedding provider is local and optional.
- R2: Confirm result scores are not treated as correctness.
- R3: Confirm chunk invalidation uses stable hashes or generations.
- R4: Confirm disabled semantic search leaves core validation unaffected.

## Reviewer Blocking Criteria

Block if semantic search requires a remote service for normal operation,
changes validation truth, lacks invalidation, or cannot be disabled.
