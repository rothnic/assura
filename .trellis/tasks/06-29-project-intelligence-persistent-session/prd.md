---
title: Project intelligence persistent session
status: in_progress
---

# Project Intelligence Persistent Session

## Objective

Execute `docs/goals/assura-project-intelligence-persistent-session.md` by
promoting a local warm-session workflow for repeated project-intelligence
checks and queries.

## Product Slice

Add a public `assura content session` surface that keeps one
project-intelligence context loaded, accepts JSON-line requests on stdin, emits
JSON-line responses on stdout, and reloads conservatively when project
configuration or modeled content changes.

This is the smallest useful session surface for later MCP/LSP transport goals:
it avoids editor-specific protocol decisions while giving wrappers a stable
local process that can reuse content-model loading, graph/search facts,
context-pack assembly, and safe-fix preview inputs.

## Scope

- Add a content-session CLI command and checked command-surface metadata.
- Support at least diagnostics, context-pack, keyword search, graph expansion,
  missing-relations, agent-context, and safe-fix preview request types.
- Reuse existing content-query and context-pack functions instead of forking
  validation/query behavior.
- Define the JSON request/response schema, reload metadata, and error shape.
- Reload on config/schema/content/source mtime changes before serving a
  request; do not rely on filesystem watcher delivery for correctness.
- Document that `assura watch` remains experimental for this goal.
- Add focused tests and docs proving Assura-local and Beacon CRM request flows.

## Non-Goals

- No hosted daemon.
- No editor protocol, MCP server, or plugin packaging.
- No automatic safe-fix apply.
- No semantic correctness claims.
- No cache that can hide config or content changes.

## Acceptance Criteria

- [x] `assura content session` is discoverable in CLI help and command-surface
      docs.
- [x] The session returns deterministic JSON responses for representative
      repeated requests without restarting the CLI process.
- [x] Reload metadata distinguishes initial load, reused context, and
      conservative reload after changed config/content.
- [x] Tests cover request parsing, repeated reuse, invalid request errors, and
      reload after content changes.
- [x] Docs explain when to use one-shot `assura content ...` commands versus
      the session surface.
- [x] Goal, roadmap, support policy, and website docs route the next successor
      to safe-fix workflow.

## Validation

```bash
cargo fmt --check
cargo test --test project_intelligence_session --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```
