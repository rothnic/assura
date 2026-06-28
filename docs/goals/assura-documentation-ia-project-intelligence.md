---
id: goal-assura-documentation-ia-project-intelligence
type: goal
title: Assura documentation IA project intelligence
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-runtime-program.md
  - website/src/content/docs/
  - website/astro.config.mjs
  - docs/content-runtime.md
  - docs/content-runtime-inspection.md
---

# Assura Documentation IA Project Intelligence

## Objective

Restructure the documentation site so Assura's layered product model is clear:
LS-Lint-like structure validation first, then Rust Markdown validation and
fixing, then modeled content collections, then graph/query/search, then
optional code intelligence and agent/editor surfaces.

## Current Gap

The current content runtime documentation exists, but the website presents it
as one example page. That does not explain the broader product or how content
runtime extends structure validation into Markdown, modeled collections,
relations, querying, and optional code intelligence.

## User Certainty Bar

A new user should be able to open the docs site and understand:

- what Assura does today;
- how it compares to LS-Lint at the base layer;
- when to use Markdown validation;
- when to use modeled content collections;
- how relations, queries, and search build on those models;
- which code-intelligence features are optional future enrichment.

## Scope

- Redesign the docs sidebar around product layers rather than scattered
  examples.
- Add or move pages for structure validation, Markdown validation, content
  models, collection relations, agent operations, query/search, and optional
  code intelligence.
- Keep current content runtime docs but promote them from example-only
  material into a first-class product area.
- Add a "what is implemented now" status table so docs do not overclaim.
- Add copy/paste command paths for current capabilities and planned commands
  for future goals only where clearly marked.
- Update docs tests that pin navigation, links, and implementation status.

## Non-Goals

- No marketing-only landing page.
- No claim that graph search, semantic search, or code intelligence is shipped
  before implementation.
- No hosted-docs deployment overhaul unless required to preview the IA.
- No runtime feature implementation.

## Definition Of Done

- The website sidebar has a coherent section for Assura's layered validation
  and project intelligence model.
- Content runtime documentation is reachable as a first-class product area, not
  only as an example.
- Docs distinguish shipped, experimental, and planned capabilities.
- Markdown validation docs explain the split between Rust linter/fixer,
  Assura-owned heading hierarchy, and content runtime frontmatter models.
- Query/search and code-intelligence pages clearly state implementation status.
- Link and docs tests cover the new navigation.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

If website files change substantially, also run the website build command used
by `cargo xtask docs` or record why the focused docs gate is sufficient.

## Review Tasks

- R1: Confirm the docs explain the full layered product without overclaiming.
- R2: Confirm current content runtime docs are discoverable.
- R3: Confirm planned capabilities are visibly marked as planned.
- R4: Confirm navigation and links are covered by tests.

## Reviewer Blocking Criteria

Block if content runtime remains buried as a single example, if docs claim
unimplemented graph/search/code features are shipped, or if the IA hides the
LS-Lint-like structure validation entry path.
