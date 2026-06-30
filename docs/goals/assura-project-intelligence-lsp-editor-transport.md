---
id: goal-assura-project-intelligence-lsp-editor-transport
type: goal
title: Assura project intelligence LSP editor transport
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - docs/goals/assura-project-intelligence-agent-cli-surface.md
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

- Select and implement the first editor transport slice as a local
  LSP-shaped JSON-line editor session. Full LSP server framing and editor
  plugin packaging are deferred to a later packaging goal.
- Expose LSP-style diagnostics and code actions through the local session.
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
- No full LSP `Content-Length` framed server in this goal.
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
- Docs tell users how the editor transport relates to CLI, hooks, optional MCP
  adapters, and Codex agent feedback.

## Validation Commands

```bash
cargo fmt --check
cargo test -p assura --test editor_surface_cli --quiet
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

## Progress Log

- 2026-06-29: Revalidated after the CLI-first agent surface completed. The
  first editor slice will be a local `assura editor session` JSON-line protocol
  with LSP-shaped methods and responses. This keeps the implementation local
  and testable without claiming full LSP server framing, marketplace packaging,
  or a hosted language server.
- 2026-06-29: Implemented `assura editor session` as a local JSON-line protocol
  with `textDocument/diagnostics`, `textDocument/context`, and
  `textDocument/codeAction` methods over the shared content-query facts,
  context-pack output, safe-fix audit metadata, and conservative session
  fingerprint reload behavior. Updated support matrices, release notes, API
  docs, agent/editor surface docs, and the project-intelligence demo. Focused
  validation passed: `cargo fmt --check`,
  `cargo test -p assura --test editor_surface_cli --quiet`,
  `cargo test --test project_intelligence_context_pack --quiet`,
  `cargo test -p assura --test cli_command_surface_tests --quiet`,
  `cargo run --quiet -- check --format json .`, `cargo xtask docs`,
  `cargo xtask evidence`, and `git diff --check`.
- 2026-06-29: Independent review agent
  `019f148d-2646-7921-a827-195512b082d9` found and fixed absolute
  `file://` URI matching for relative session roots and strengthened tests to
  prove parity with content diagnostics, context packs, safe-fix preview
  output, dry-run audit IDs, and no implicit LSP `command` or `edit` writes.
  Re-ran the validation chain after review; all commands passed.
