---
id: goal-assura-vscode-supported-extension
type: goal
title: Assura VS Code supported extension
status: planned
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
- Decide whether marketplace publication is in scope for the first supported
  milestone or explicitly defer it with a support-policy note.

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
- Independent review confirms the extension does not hide daemon failures,
  duplicate validation logic, or apply safe fixes implicitly.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cd integrations/editors/vscode && pnpm test
cd integrations/editors/vscode && pnpm run build
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
