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

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Revalidated this goal after PR #115 merged the supported document-graph slice. The goal remains valid because live daemon lifecycle commands still managed runtime metadata rather than a long-running process, and tests asserted `process.running=false`. Started Trellis task `07-01-07-01-true-daemon-mode` on branch `codex/true-daemon-mode`. | `docs/goals/assura-post-beta-capabilities-program.md`; `.trellis/tasks/07-01-07-01-true-daemon-mode/prd.md`; `.trellis/spec/assura/roadmap.md`; `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_management.rs`; `tests/daemon_cli_tests.rs`. |
| 2026-07-01 | Added the first true-daemon implementation slice. `assura daemon start` now launches a hidden local `daemon serve` process, status probes versioned `assura.daemon.v1` IPC before reporting `running=true`, stop/restart manage the process idempotently, and JSON/YAML `daemon check-path` uses daemon IPC when available while preserving one-shot fallback. The slice keeps broader editor, agent, and content-reference daemon workflows experimental follow-up work. | `src/cli/daemon_process.rs`; `src/cli/daemon.rs`; `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_management.rs`; `tests/daemon_cli_tests.rs`; `.trellis/spec/assura/daemon-management-cli.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `docs/data/release-surfaces.json`; `cargo test --test daemon_cli_tests --quiet`; `cargo xtask target-state`. |
| 2026-07-01 | Closed independent-review blockers for stale-state safety and process ownership. Daemon health probes now force config freshness before reporting `running`, stop/restart only signal identity-verified daemon processes, start/restart return runtime errors on failed spawn, and regression tests cover stale config, crashed processes, stale PID metadata, and process cleanup/replacement. | Review agent `019f2030-a4ac-77f0-b5a4-df30ce68e50e`; `src/daemon/mod.rs`; `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_process.rs`; `src/cli/daemon_transport.rs`; `tests/daemon_cli_tests.rs`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`. |
| 2026-07-01 | Kept Windows and tarpaulin coverage to compile, installer, installable-adoption smoke, and non-subprocess daemon tests while marking managed subprocess lifecycle CLI regressions as normal Unix CI coverage. Windows full-suite and tarpaulin runs repeatedly hung after daemon subprocess tests even though Windows install smoke and Unix/macOS lifecycle tests passed, so Windows-specific daemon lifecycle hardening remains a follow-up instead of blocking this Unix-proven true-daemon slice. | `Cargo.toml`; `.github/workflows/ci.yml`; `tests/daemon_cli_tests.rs`; PR #116 checks; `Windows Installer Smoke`; `Installable Adoption Smoke (windows-x86_64)`; `Code Coverage`. |
