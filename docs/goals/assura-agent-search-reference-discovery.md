---
id: goal-assura-agent-search-reference-discovery
type: goal
title: Assura agent search and reference discovery
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-supported-document-graph.md
  - ./assura-content-query-and-search-cli.md
---

# Assura Agent Search And Reference Discovery

## Objective

Make Assura useful for discovery before perfect modeling by adding raw search
fallback, frontmatter repository references, all/unresolved reference listing,
and discoverable agent-query capabilities.

## Scope

- Add raw repository text search as a fallback when modeled content has no
  chunks or is inactive.
- Add configurable frontmatter reference extraction for fields such as source
  documents, related records, evidence, and requirements.
- Include frontmatter references in repository reference graph output, context
  packs, affected-reference answers, and doctor summaries.
- Add all-reference, unresolved-reference, and summary discovery outputs so
  agents can enumerate the edges they need to fix.
- Add discoverable agent-query capabilities for gaps, next actions, and
  unresolved references.

## Non-Goals

- No semantic search correctness claim.
- No natural-language agent-query parser as the primary API.
- No requirement that content models be perfect before raw discovery works.

## Definition Of Done

- A repo with inactive content models can still find raw text matches through
  Assura.
- Frontmatter paths produce repository-reference diagnostics and graph facts.
- Agent context can enumerate unresolved references directly.
- Agent-query discovery lists available deterministic capabilities and
  follow-up surfaces.

## Validation Commands

```bash
cargo fmt --check
cargo test --test content_query_cli --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if agents must fall back to external grep for day-one discovery, if
frontmatter references are invisible to the graph, or if unresolved-reference
counts cannot be enumerated.
