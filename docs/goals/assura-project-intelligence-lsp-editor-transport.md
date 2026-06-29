---
id: goal-assura-project-intelligence-lsp-editor-transport
type: goal
title: Assura project intelligence LSP editor transport
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - docs/goals/assura-project-intelligence-mcp-agent-transport.md
  - website/src/content/docs/product/agent-editor-surfaces.md
---

# Assura Project Intelligence LSP Editor Transport

## Objective

Expose project-intelligence diagnostics, object context, and safe-fix previews
inside editor workflows through a supported local editor protocol that reuses
the same validation and query contracts as the CLI.

## Current Gap

The website describes editor sessions and LSP behavior as future work. Users can
run CLI commands and agents can request context packs, but maintainers cannot
yet rely on a supported editor surface for diagnostics, related modeled
content, or safe-fix code actions.

## Scope

- Select and implement the first editor transport slice, with LSP diagnostics
  and code actions as the default candidate unless revalidation identifies a
  better fit.
- Map file diagnostics to existing Assura validation and Markdown/content
  diagnostic contracts.
- Expose project-intelligence context for the file or modeled object under
  edit without making semantic ranking a correctness source.
- Expose safe-fix previews as code actions, and apply fixes only if the
  safe-fix workflow has completed explicit opt-in apply support.
- Reuse persistent-session state when available, with conservative invalidation
  and full-check fallback.
- Add protocol-level tests that do not require a specific editor UI.

## Non-Goals

- No editor marketplace package.
- No hosted language server.
- No transport-specific validation behavior.
- No semantic content generation.
- No automatic repair without explicit approval.

## Definition Of Done

- The editor transport reports representative structure, Markdown, and
  project-intelligence diagnostics from the shared core.
- The editor transport can provide bounded project-intelligence context for the
  file or modeled object under edit.
- Safe-fix previews are exposed without writes, and apply behavior is gated by
  the safe-fix workflow's explicit opt-in contract.
- Tests prove editor protocol output agrees with CLI/context-pack output on
  representative Assura and Beacon CRM examples.
- Docs tell users how the editor transport relates to CLI, hooks, MCP, and
  Codex agent feedback.

## Validation Commands

```bash
cargo fmt --check
cargo test lsp --quiet
cargo test editor --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm editor diagnostics are wrappers over shared validation output.
- R2: Confirm invalidation is conservative and cannot hide changed config or
  content.
- R3: Confirm safe-fix code actions do not write unless the user explicitly
  accepts an apply-capable action.
- R4: Confirm docs do not advertise editor support beyond the implemented
  protocol slice.

## Reviewer Blocking Criteria

Block if the editor transport forks validation logic, relies on watcher events
as the only freshness signal, applies repairs implicitly, requires a hosted
service, or claims support for editor packaging that was not implemented.
