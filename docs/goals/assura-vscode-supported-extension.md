---
id: goal-assura-vscode-supported-extension
type: goal
title: Assura VS Code supported extension
status: completed
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-vscode-daemon-integration.md
  - ./assura-true-daemon-mode.md
  - ../../integrations/editors/vscode/
---

# Assura VS Code Supported Extension

## Objective

Move the existing experimental VS Code integration toward a supported extension
surface that can be installed, updated, tested, and documented without
duplicating Assura validation logic.

## Current Gap

The beta VS Code work proved diagnostics, daemon status/lifecycle commands,
changed-document checks, and preview-only safe-fix commands over Assura CLI
contracts. Post-beta support needs packaging, install/update guidance,
compatibility policy, daemon-process integration, diagnostics reliability, and
release evidence suitable for real users.

## User-Specific Verification Picture

Nick is maintaining a documentation-heavy project on a branch that renames an
architecture document, moves related implementation files, and edits several
Markdown pages before merge. In VS Code, the supported Assura extension should
let him stay inside the editor while proving the same branch-safety story that
the CLI, daemon, agent hooks, and final release gates prove.

The expected workflow is:

1. Nick opens the repository in VS Code with the local Assura extension loaded
   from `integrations/editors/vscode`.
2. The extension connects to the shared Assura daemon when it is healthy and
   asks the same contract-backed checks that `assura check`,
   `assura daemon check-path`, `assura editor session`, and
   `assura fix markdown --dry-run` expose outside the editor.
3. When he edits a Markdown file, diagnostics appear for structure, Markdown,
   headings, frontmatter/content-model, document graph, and repository-reference
   findings that Assura already knows how to report. The extension must not own
   a private validation engine or produce editor-only rule IDs.
4. If the daemon is stopped, stale, or unhealthy, VS Code makes that visible and
   falls back to one-shot CLI diagnostics instead of silently pretending the
   daemon path succeeded.
5. Before applying repairs, Nick previews deterministic Markdown safe fixes.
   The extension may show the preview path, but writes require explicit user
   action and must stay aligned with the CLI safe-fix contract.
6. Agent hooks can still inject compact nudges before or after relevant tool
   events without bloating editor context, because the editor surface consumes
   the same findings and daemon/session contracts rather than inventing a second
   integration model.
7. The local package can be installed for development, updated, removed, and
   diagnosed using documented commands. The support policy is honest that this
   milestone is a supported beta local package, not a marketplace extension or
   full LSP server.

Final verification should demonstrate that VS Code helps Nick decide whether
the branch is mergeable: diagnostics, daemon recovery, safe-fix preview, and
support/package evidence all agree with the broader parent goal's CLI, daemon,
agent, Markdown, document graph, and LS-Lint performance gates.

## Scope

- Define the supported VS Code extension API surface and version compatibility
  with Assura CLI/daemon protocols.
- Reuse the true daemon process when available and fall back to one-shot CLI
  output without hiding daemon health failures.
- Surface structure, Markdown, content model, document graph, reference, and
  performance-gate findings through VS Code diagnostics and commands.
- Keep safe fixes preview-first and explicit before writes.
- Add extension build/test/package validation and documented install/update/
  remove/doctor workflows.
- Explicitly defer marketplace publication and full LSP framing for this
  milestone with support-policy evidence.

## Non-Goals

- No editor-specific validation engine.
- No Zed or JetBrains extension implementation in this goal.
- No automatic repair without explicit user action.
- No supported marketplace claim unless packaging and release evidence prove it.

## Definition Of Done

- VS Code diagnostics and commands use shared Assura CLI/daemon contracts.
- Extension package tests and build commands run in CI or an equivalent release
  gate.
- Install, update, remove, doctor, daemon recovery, and fallback workflows are
  documented.
- Support policy and release surfaces classify the extension accurately.
- The implemented surface satisfies the user-specific verification picture
  above, especially daemon visibility, shared diagnostics, safe-fix preview, and
  honest local-package support boundaries.
- Independent review confirms the extension does not hide daemon failures,
  duplicate validation logic, or apply safe fixes implicitly.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cd integrations/editors/vscode && pnpm test
cd integrations/editors/vscode && pnpm run build
cd integrations/editors/vscode && pnpm run doctor
cd integrations/editors/vscode && pnpm run package
git diff --check
```

## Review Tasks

- R1: Confirm diagnostics come from shared Assura outputs.
- R2: Confirm daemon health and fallback behavior are visible to users.
- R3: Confirm package/install/update/remove docs match the implemented surface.
- R4: Confirm safe fixes require explicit approval.

## Reviewer Blocking Criteria

Block if VS Code implements private validators, if daemon errors are swallowed,
if package docs imply unsupported marketplace availability, if extension tests
are not executable, or if safe fixes can write without explicit user action.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Started the supported VS Code extension slice from `origin/master` after the agent installer lifecycle merged. The goal now carries the user-specific editor verification picture so package, daemon, diagnostic, safe-fix, and support-policy work is judged against the actual maintainer workflow. | `.trellis/tasks/archive/2026-07/07-02-vscode-supported-extension/prd.md`; branch `codex/vscode-supported-extension`; [Post-beta capabilities program](./assura-post-beta-capabilities-program.md). |
| 2026-07-02 | Completed the supported beta local VS Code package slice. The extension remains a local package, keeps marketplace publication and full LSP framing deferred, uses shared CLI/daemon/editor-session contracts for diagnostics and safe-fix previews, warns on daemon check-path fallback, and now has executable test/build/doctor/package smoke commands. | `integrations/editors/vscode/package.json`; `integrations/editors/vscode/README.md`; `integrations/editors/vscode/src/assura-client.js`; `integrations/editors/vscode/src/extension.js`; `integrations/editors/vscode/tests/assura-client.test.js`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `docs/data/release-surfaces.json`; `pnpm test`; `pnpm run build`; `pnpm run doctor`; `pnpm run package`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
