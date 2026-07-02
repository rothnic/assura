---
title: Supported document graph closure
status: active
priority: P0
---

# Supported Document Graph Closure

## Goal

Close the `docs/goals/assura-supported-document-graph.md` child goal for the
post-beta parent program only if current evidence proves the supported document
graph contract end to end.

This is not new semantic search, hosted graph service, daemon, or editor work.
It is a support-grade closure pass: align the goal, roadmap, support docs,
target-state checks, and final verification story with the actual local graph
surfaces that are already implemented.

## Current Evidence

- The archived task `.trellis/tasks/archive/2026-07/07-01-supported-document-graph`
  completed bounded repository-reference context packs and promoted
  `assura content references` into the supported graph contract.
- `docs/support-policy.md` and `docs/compatibility-and-surface.md` classify
  modeled content queries, repository-reference facts, context packs, and
  sessions as supported local surfaces while keeping semantic/code-symbol
  outputs experimental candidate enrichment.
- `docs/content-runtime.md` documents the supported graph workflow from content
  models through validation, query/search, relation diagnostics, bounded graph
  expansion, repository references, and context packs.
- Existing fixture tests cover content query/search, missing relations,
  repository-reference queries, object-mode context packs, and realistic
  project-intelligence workflows.

## Requirements

- Re-audit the child goal definition of done against current files and command
  output before changing status.
- Tie the document graph closure back to the parent goal's user-specific
  verification criteria: the graph must help a maintainer decide which goals,
  ADRs, analysis notes, generated references, source files, tests, headings,
  benchmark rows, and release docs are affected by a renamed page or moved
  module.
- Keep supported graph behavior local and deterministic: no hosted services,
  no daemon prerequisite, no editor prerequisite, no semantic ranking as
  validation truth, and no code-symbol provider requirement.
- Ensure target-state checks or docs prevent unsupported graph/search/content
  validation claims from drifting into release surfaces.
- Update roadmap and parent progress so the next child goal can start from a
  clean handoff.
- Archive the completed Markdown engine task before continuing the parent
  program, because PR #129 intentionally left active task archival to Trellis.

## Acceptance Criteria

- [ ] The supported document graph goal status and progress log reflect
      current evidence without overclaiming semantic/code-symbol behavior.
- [ ] The parent goal includes user-specific criteria for the final major
      increment, and this closure records the document graph portion of that
      branch-safety story.
- [ ] Public support docs still separate supported graph behavior from
      experimental candidate enrichment.
- [ ] Target-state evidence catches unsupported hosted, semantic, code-symbol,
      or dependency-graph claims in supported surfaces.
- [ ] Current validation proves content validation, content query/search,
      missing relation diagnostics, repository-reference graph queries,
      object-mode context packs, and realistic project proof.
- [ ] The roadmap routes to the next incomplete post-beta child goal after the
      document graph closure.
- [ ] Independent review confirms the child closure is support-grade and does
      not depend on a hosted service, daemon, editor plugin, semantic search,
      or code-symbol provider.

## Validation

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test project_intelligence --quiet
cargo test --test content_query_cli --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
python3 ./.trellis/scripts/task.py validate 07-02-supported-document-graph-closure
git diff --check
```

## Reviewer Blocking Criteria

Block if the goal is closed without current command evidence, if docs imply
semantic search or code-symbol providers decide validation truth, if graph
support requires daemon/editor/hosted infrastructure, if repository-reference
facts disappear from bounded context packs, or if roadmap routing keeps agents
on completed Markdown candidate research.
