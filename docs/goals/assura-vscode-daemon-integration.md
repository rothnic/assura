---
id: goal-assura-vscode-daemon-integration
type: goal
title: Assura VS Code daemon integration
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ./assura-daemon-management-cli.md
  - ./assura-reference-daemon-readiness.md
---

# Assura VS Code Daemon Integration

## Objective

Build the first editor integration around the shared daemon CLI/client
contracts, with VS Code diagnostics and daemon health visible in the editor.

## Scope

- Use the shared daemon client or future daemon command JSON contracts.
- Create the first VS Code extension package under
  `integrations/editors/vscode/` unless implementation research records a
  better repo-native package path before coding starts.
- Report validation and reference findings through the VS Code Problems panel.
- Show daemon health in the status bar.
- Provide command-palette actions for start, stop, restart, doctor, and logs.
- Surface safe-fix previews without applying writes implicitly.

## Non-Goals

- No Zed or JetBrains implementation in this goal.
- No marketplace release until install/update/support policy is defined.
- No editor-specific scanner that bypasses Assura CLI/core behavior.

## Definition Of Done

- VS Code can start or connect to the daemon for a workspace.
- Diagnostics update from daemon or one-shot fallback output.
- A stopped or unhealthy daemon produces actionable editor guidance.
- The extension reuses the same JSON/protocol contracts tested by CLI.
- `integrations/editors/vscode/package.json` defines the test and build
  commands used by this goal.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cd integrations/editors/vscode && pnpm test
cd integrations/editors/vscode && pnpm run build
git diff --check
```

## Review Tasks

- R1: Confirm VS Code does not implement separate validation logic.
- R2: Confirm daemon failures are visible and recoverable from the UI.
- R3: Confirm safe fixes require explicit user action.

## Reviewer Blocking Criteria

Block if the extension hides daemon failure, applies fixes implicitly, bypasses
shared CLI/client contracts, or claims support for Zed/JetBrains before those
follow-up goals exist.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Started Epic 9 with the first experimental VS Code extension package under `integrations/editors/vscode/`. The package contributes diagnostics, daemon status/lifecycle/doctor/logs, and safe-fix preview commands over shared Assura CLI JSON contracts; support docs classify it as experimental and not marketplace-ready. Independent review Leibniz found a high-risk non-zero JSON handling gap, which was fixed by parsing stdout JSON before rejecting process errors and adding regression tests for blocking diagnostics/remediation payloads. | `integrations/editors/vscode/package.json`; `integrations/editors/vscode/src/assura-client.js`; `integrations/editors/vscode/src/extension.js`; `integrations/editors/vscode/tests/assura-client.test.js`; `.assura/config.yml`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `docs/data/release-surfaces.json`; `docs/release-notes.md`; independent review Leibniz; `pnpm test`; `pnpm run build`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `git diff --check`. |
| 2026-07-01 | Continued Epic 9 with changed-document daemon/editor diagnostics. The extension now refreshes saved and active workspace files with `assura daemon check-path` plus `assura editor session` diagnostics, preserves LSP diagnostic ranges/severity, ignores non-file and outside-workspace documents before shelling out, and keeps full-project `assura check --format json` as refresh fallback. Independent review Helmholtz's event-scope blocker was fixed before commit. | `integrations/editors/vscode/src/assura-client.js`; `integrations/editors/vscode/src/extension.js`; `integrations/editors/vscode/tests/assura-client.test.js`; `integrations/editors/vscode/README.md`; independent review Helmholtz; `pnpm test`; `pnpm run build`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `git diff --check`. |
