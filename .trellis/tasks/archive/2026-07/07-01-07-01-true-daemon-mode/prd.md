---
title: True daemon mode
status: completed
priority: P0
---

# True Daemon Mode

## Objective

Turn `assura daemon` from lifecycle metadata into a real local process with a
versioned IPC health/check/reference contract, while preserving one-shot CLI
fallback and the existing `LocalDaemonCore` truth source.

## Current Gap

The current daemon commands manage project-local runtime files and expose
daemon-ready health, check-path, and repository-reference responses, but
`daemon start` does not launch a managed process and `daemon status` honestly
reports `process.running=false`.

The first implementation slice should make lifecycle commands prove a real
process and IPC endpoint without claiming the full parent goal is complete.

## Scope

- Add a hidden daemon server entrypoint used by `daemon start`.
- Start a background local process with a loopback TCP IPC address by default;
  keep Unix socket transport support available for explicit daemon addresses.
- Write runtime JSON with PID, IPC address, protocol version, and socket path
  when applicable.
- Make `daemon status`, `daemon doctor`, `daemon stop`, and `daemon restart`
  probe and manage the real process.
- Serve a minimal versioned IPC protocol for health, changed-path structure
  checks, and bounded repository-reference queries.
- Keep one-shot fallback behavior for unavailable, stale, or incompatible
  daemon states.
- Update docs, support surface notes, and goal progress logs without implying
  hosted daemon, public plugin API, or editor marketplace support.

## Non-Goals

- No hosted daemon or remote telemetry.
- No separate per-agent daemon protocol.
- No daemon-only validation semantics.
- No claim that all broader content queries are served over IPC in this slice
  unless implemented and tested here.

## Acceptance Criteria

- [x] `assura daemon start <project> --format json` launches a process and
      returns runtime metadata with `running=true`, `pid`, and protocol
      version.
- [x] `assura daemon status <project> --format json` distinguishes not-started,
      running, stopped, crashed, and unavailable states without reporting stale
      process metadata as fresh.
- [x] `assura daemon stop` terminates the managed process idempotently.
- [x] `assura daemon restart` replaces the managed process and keeps runtime
      metadata current.
- [x] `assura daemon doctor` reports actionable remediation for unavailable,
      stale/crashed, and running states.
- [x] `assura daemon check-path` can use the running daemon IPC path for a
      changed-path structure check and falls back to one-shot local state when
      no daemon is available.
- [x] `assura daemon references` can use the running daemon IPC path for
      source, target, and moved-target repository-reference context while
      preserving one-shot fallback.
- [x] Representative warm daemon changed-path and target-reference queries are
      faster than cold one-shot rows, or any miss is attributed and fixed before
      this child goal is claimed complete.
- [x] Tests cover lifecycle, IPC health/check-path/references, stale config
      safety, crash detection, and fallback behavior.
- [x] Public docs classify true daemon mode accurately for this slice.

## Completion Evidence

Completed in PR #117, merged on 2026-07-02 as commit
`745455215d757e49fb4614a170e48f046cb829ad`. All GitHub checks passed,
including Rustfmt, Check, Clippy, Code Coverage, Evidence Gates, Performance
Report, Release Bundle Smoke, Linux/macOS/Windows test suites, installable
adoption smokes, Windows Installer Smoke, documentation, and security scope.

## Validation

```bash
cargo fmt --check
cargo test --test daemon_cli_tests --quiet
cargo test --test daemon_reference_cli_tests --quiet
cargo test --test daemon_core_tests --quiet
cargo test --test editor_surface_cli --quiet
cargo test --test agent_surface_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
python3 ./.trellis/scripts/task.py validate 07-01-07-01-true-daemon-mode
```

## Reviewer Blocking Criteria

Block if `daemon start` still only writes metadata, if status can report stale
or dead daemon state as fresh, if the IPC contract lacks protocol versioning,
if one-shot fallback disappears, if lifecycle tests leave child processes
behind, or if docs imply that editor/agent daemon support is fully complete
before the corresponding child goals prove it.
