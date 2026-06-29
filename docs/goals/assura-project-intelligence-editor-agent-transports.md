---
id: goal-assura-project-intelligence-editor-agent-transports
type: goal
title: Assura project intelligence editor and agent transports
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - .trellis/spec/assura/codex-agent-feedback.md
  - website/src/content/docs/product/agent-editor-surfaces.md
---

# Assura Project Intelligence Editor And Agent Transports

## Objective

Expose diagnostics, safe-fix previews, graph/search queries, semantic
candidates, and code-symbol context through concrete editor and agent
transports that reuse the shared project-intelligence contracts.

## Current Gap

The CLI has generic agent envelopes, and docs classify daemon/editor sessions,
LSP, and MCP as planned. Usability requires at least one real editor or agent
transport that proves the shared contracts are not just command-line wrappers.

## Scope

- Select the first supported transport slice, with LSP diagnostics/code actions
  and MCP tools as the likely pair.
- Map transport requests to the same core APIs used by `assura check`,
  `assura content agent-query`, and safe-fix dry-runs.
- Reuse the persistent-session goal's state management when available.
- Document request/response schemas and support levels.
- Add transport integration tests that do not require a specific editor UI.
- Preserve `assura check --format agent --agent codex` as the Codex delivery
  path and avoid per-agent command families.

## Non-Goals

- No automatic agent orchestration.
- No hosted MCP server requirement.
- No editor plugin marketplace.
- No transport-specific validation behavior.
- No remote provider requirement.

## Definition Of Done

- At least one editor transport and one agent transport expose diagnostics and
  project-intelligence queries through shared contracts.
- Safe-fix previews are exposed without applying changes automatically.
- Tests prove CLI, editor, and agent transports agree on representative
  diagnostics/query output.
- Docs tell users when to use CLI, hook, editor, MCP, or Codex delivery.
- Support policy rows match implemented behavior.

## Validation Commands

```bash
cargo fmt --check
cargo test agent_surface --quiet
cargo test lsp --quiet
cargo test mcp --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- content agent-query diagnostics --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm transport behavior is a wrapper over shared core contracts.
- R2: Confirm support policy and docs do not overstate editor or MCP support.
- R3: Confirm safe-fix behavior remains preview-only unless the safe-fix
  workflow goal has completed apply support.
- R4: Confirm no per-agent command family or per-agent output format appears.

## Reviewer Blocking Criteria

Block if transport code forks validation/query logic, if MCP or LSP behavior
requires a hosted service, if Codex delivery is moved away from the shared
agent format, or if docs claim unsupported transports are supported.
