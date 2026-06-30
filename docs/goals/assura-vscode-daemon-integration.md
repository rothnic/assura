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

## Validation Commands

```bash
cargo run --quiet -- check --format json .
pnpm test
pnpm run build
git diff --check
```

Adjust Node commands to the extension package once the package location exists.

## Review Tasks

- R1: Confirm VS Code does not implement separate validation logic.
- R2: Confirm daemon failures are visible and recoverable from the UI.
- R3: Confirm safe fixes require explicit user action.

## Reviewer Blocking Criteria

Block if the extension hides daemon failure, applies fixes implicitly, bypasses
shared CLI/client contracts, or claims support for Zed/JetBrains before those
follow-up goals exist.
