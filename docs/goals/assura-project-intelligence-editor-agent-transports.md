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
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - docs/goals/assura-project-intelligence-agent-cli-surface.md
  - docs/goals/assura-project-intelligence-lsp-editor-transport.md
  - .trellis/spec/assura/codex-agent-feedback.md
  - website/src/content/docs/product/agent-editor-surfaces.md
---

# Assura Project Intelligence Editor And Agent Transports

## Status

Superseded by two narrower executable goals:

- [Project Intelligence Agent CLI Surface](./assura-project-intelligence-agent-cli-surface.md)
- [Project Intelligence LSP Editor Transport](./assura-project-intelligence-lsp-editor-transport.md)

This document remains as the original umbrella framing, but agents should not
execute it directly.

## Objective

Expose diagnostics, safe-fix previews, graph/search queries, semantic
candidates, and code-symbol context through concrete editor and agent
transports that reuse the shared project-intelligence contracts.

## Current Gap

The CLI has generic agent envelopes, and docs classify daemon/editor sessions,
LSP, and optional protocol adapters as planned. After context-pack,
persistent-session, and safe-fix workflow goals define the shared contracts,
usability requires local agent/editor surfaces that prove those contracts are
not just isolated human-oriented commands.

## Scope

- Select the first supported local surface slice, with agent CLI ergonomics
  first and LSP diagnostics/code actions after that.
- Map transport requests to the same core APIs used by `assura check`,
  context packs, `assura content agent-query`, and safe-fix workflows.
- Reuse the persistent-session goal's state management when available.
- Document request/response schemas and support levels.
- Add transport integration tests that do not require a specific editor UI.
- Preserve `assura check --format agent --agent codex` as the Codex delivery
  path and avoid per-agent command families.

## Non-Goals

- No automatic agent orchestration.
- No hosted MCP server requirement.
- No MCP requirement for local agent usability.
- No editor plugin marketplace.
- No transport-specific validation behavior.
- No remote provider requirement.

## Definition Of Done

- At least one local agent surface and one editor surface expose diagnostics
  and project-intelligence queries through shared contracts.
- Safe-fix previews are exposed without applying changes automatically, and
  apply behavior is available only if the safe-fix workflow goal has completed
  explicit opt-in support.
- Tests prove CLI, editor, and agent surfaces agree on representative
  diagnostics/query output.
- Docs tell users when to use CLI, hook, editor, optional protocol adapters, or
  Codex delivery.
- Support policy rows match implemented behavior.

## Validation Commands

```bash
cargo fmt --check
cargo test agent_surface --quiet
cargo test lsp --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- content agent-query diagnostics --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm surface behavior is a wrapper over shared core contracts.
- R2: Confirm support policy and docs do not overstate editor or MCP support.
- R3: Confirm safe-fix behavior remains preview-only unless the safe-fix
  workflow goal has completed apply support.
- R4: Confirm no per-agent command family or per-agent output format appears.

## Reviewer Blocking Criteria

Block if surface code forks validation/query logic, if MCP or LSP behavior is
required for local agent usability, if Codex delivery is moved away from the
shared agent format, or if docs claim unsupported transports are supported.
