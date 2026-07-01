# Daemon Management CLI

## Problem

Epic 6 proved daemon-ready local state and a narrow `assura daemon` probe
surface, but beta still lacks the human/editor/hook/agent control plane for a
managed local daemon lifecycle.

## Goal

Execute `docs/goals/assura-daemon-management-cli.md` as Epic 7 of the beta
program. Build JSON-first status, doctor, start, stop, restart, and logs
commands over the same local daemon/client contracts introduced in Epic 6.

## Scope

- Reuse `LocalDaemonCore`, daemon health metadata, fallback commands, runtime
  paths, and repository-reference contracts instead of adding per-client logic.
- Add idempotent lifecycle commands that are safe for repeated agent runs.
- Keep daemon state under `.assura/daemon/` or another explicit daemon runtime
  directory; do not litter `.assura/` root.
- Add machine-readable doctor output with exact next commands and one-shot
  fallback guidance.
- Update command-surface, support policy, compatibility docs, release notes,
  and release-surface metadata consistently.
- Add explicit `assura daemon references --target` versus
  `assura content references --target` CLI parity coverage carried forward from
  Epic 6 closure review.

## Non-Goals

- No VS Code UI in this task.
- No remote daemon manager or hosted service.
- No editor-only daemon lifecycle behavior.
- No per-agent validation branches.

## Validation

Use narrow checks while iterating:

```bash
cargo fmt --check
cargo test --test daemon_cli_tests --quiet
cargo run --quiet -- check --format json .
git diff --check
```

Before completion, also run:

```bash
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
```

## Review

Complex implementation slices require independent review. Ask reviewers to
focus on lifecycle idempotence, JSON contract completeness, state-file
organization, exact fallback guidance, and whether editor/hook/agent callers
can use the same CLI/client contract.
