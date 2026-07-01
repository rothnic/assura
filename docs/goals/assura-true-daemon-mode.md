---
id: goal-assura-true-daemon-mode
type: goal
title: Assura true daemon mode
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-reference-daemon-readiness.md
  - ./assura-daemon-management-cli.md
---

# Assura True Daemon Mode

## Objective

Replace the current experimental runtime-metadata daemon preview with a real
local daemon process that provides warm, incremental validation over a stable
IPC contract for humans, editors, hooks, and agents.

## Current Gap

`v0.2.0` proves daemon-ready core state and management-preview commands, but it
does not claim a long-running socket/process server. The next goal must close
that gap without weakening one-shot `assura check` truth.

## Scope

- Define process lifecycle, IPC transport, protocol versioning, and project
  discovery.
- Implement start/stop/restart/status/doctor/logs against the real process.
- Preserve one-shot fallback when the daemon is unavailable or stale.
- Prove changed-path structure checks and reference queries match one-shot
  truth.
- Add editor/agent-friendly JSON responses with bounded payloads.
- Add stale-config, deleted-target, moved-target, dirty-worktree, crash, and
  lockfile recovery tests.

## Non-Goals

- No hosted daemon.
- No remote telemetry.
- No daemon-only validation semantics that cannot be reproduced by one-shot
  commands.

## Definition Of Done

- Daemon process and IPC protocol are implemented and documented.
- Warm daemon checks are faster than cold one-shot checks on representative
  changed-path cases without stale-state false freshness.
- `assura daemon doctor` can explain unavailable, stale, crashed, or mismatched
  daemon states.
- VS Code and agent integration docs route through the shared daemon protocol.
- Independent review accepts process lifecycle, cache invalidation, and fallback
  behavior.

## Validation Commands

```bash
cargo fmt --check
cargo test --test daemon_core_tests --quiet
cargo test --test daemon_cli_tests --quiet
cargo test --test editor_surface_cli --quiet
cargo test --test agent_surface_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
```

## Reviewer Blocking Criteria

Block if the implementation is still only runtime metadata, can report stale
state as fresh, lacks IPC versioning, lacks one-shot fallback, omits crash or
config-stale tests, or creates editor/agent-specific validation paths.
