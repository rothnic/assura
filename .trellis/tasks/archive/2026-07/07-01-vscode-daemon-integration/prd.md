# VS Code Daemon Integration

## Problem

The beta program now has daemon-ready local state, daemon management commands,
and agent nudge recipes, but the editor integration epic has no executable task.
VS Code needs a first local extension that surfaces Assura diagnostics and daemon
health through the shared CLI/client contracts instead of building an
editor-specific scanner.

## Goal

Execute `docs/goals/assura-vscode-daemon-integration.md` as Epic 9 of the beta
program. Deliver the first VS Code extension package under
`integrations/editors/vscode/` unless implementation research records a better
repo-native path before coding starts.

## Scope

- Create the VS Code extension package and package metadata.
- Reuse `assura daemon status/doctor/start/stop/restart/logs --format json`,
  changed-path daemon commands, `assura editor session`, or one-shot
  `assura check --format json` fallback output.
- Report validation/reference findings through the VS Code Problems panel.
- Show daemon health in the status bar.
- Provide command-palette actions for daemon status, start, stop, restart,
  doctor, logs, and safe-fix preview.
- Keep safe fixes preview-only unless the user explicitly invokes an Assura CLI
  apply command outside the extension's automatic flow.
- Register new package paths in `.assura/config.yml` and support/surface docs
  only when the implementation exists.

## Non-Goals

- No Zed, JetBrains, or generic LSP package.
- No marketplace release.
- No editor-specific scanner or validation engine.
- No implicit writes or automatic repair.
- No daemon socket/process claim beyond the current runtime-metadata preview
  unless a later daemon goal implements and tests it first.

## Validation

Use narrow checks while iterating:

```bash
cargo run --quiet -- check --format json .
cd integrations/editors/vscode && pnpm test
cd integrations/editors/vscode && pnpm run build
git diff --check
```

Before completion, also run:

```bash
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
```

## Review

Complex implementation requires independent review before commit or PR. Ask the
reviewer to focus on whether VS Code reuses shared Assura CLI/protocol
contracts, daemon failures are visible and recoverable, diagnostics are bounded,
and safe fixes cannot apply writes implicitly.
