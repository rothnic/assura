---
id: goal-assura-project-intelligence-adoption-blueprint
type: goal
title: Assura project intelligence adoption blueprint
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-runtime-program.md
  - website/src/content/docs/product/content-models.md
  - website/src/content/docs/product/query-search.md
  - website/src/content/docs/product/agent-editor-surfaces.md
---

# Assura Project Intelligence Adoption Blueprint

## Objective

Create the first-run path that gets a maintainer from an ordinary repository to
useful project-intelligence feedback: modeled content, Markdown diagnostics,
graph/search queries, and agent envelopes.

## Current Gap

The runtime commands exist, but adoption still expects users or agents to infer
how `assura init`, content models, Markdown scopes, query commands, safe-fix
dry-runs, and agent envelopes fit together. That is not a usable product path.

## Scope

- Define one recommended first-run workflow for project intelligence.
- Add or update config templates, examples, docs, or command guidance so the
  workflow is copyable.
- Ensure `assura status --format json` or another supported summary tells users
  whether project-intelligence pieces are configured.
- Include a minimal content model, relation, Markdown check, search query,
  missing-relation query, graph expansion, and agent-query example.
- Keep the path local and source-control friendly.

## Non-Goals

- No daemon, LSP, or MCP implementation.
- No automatic repo inference that writes broad policies without review.
- No remote provider setup.
- No replacement for the existing structure-first adoption path.

## Definition Of Done

- A clean fixture or sample repo can follow the documented path from init to a
  passing check and a useful `assura content` query.
- Invalid sample states demonstrate missing frontmatter/model fields, broken
  relations, Markdown lint, and at least one search or graph query.
- Agent wrappers can discover the same capabilities through
  `assura content agent-context` and `assura content agent-query`.
- Docs tell users what to do next when no content model is configured.
- The workflow is linked from discoverable website navigation.

## Validation Commands

```bash
cargo fmt --check
cargo test --test content_query_cli --quiet
cargo test --test content_runtime_check_cli --quiet
cargo run --quiet -- check --format json .
cargo run --quiet -- content agent-context --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Follow the workflow from a fresh fixture and confirm the commands work in
  order.
- R2: Confirm the workflow starts with supported local surfaces only.
- R3: Confirm examples distinguish structure validation, typed content
  validation, graph/search context, and agent envelopes.
- R4: Confirm docs do not imply semantic search or code symbols are required.

## Reviewer Blocking Criteria

Block if a new user still has to infer the setup path from scattered docs, if
examples use unsupported commands, if the workflow requires remote services, or
if project-intelligence adoption breaks the existing structure-first init path.
