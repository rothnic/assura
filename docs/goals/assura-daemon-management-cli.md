---
id: goal-assura-daemon-management-cli
type: goal
title: Assura daemon management CLI
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ./assura-reference-daemon-readiness.md
  - ../compatibility-and-surface.md
  - ../support-policy.md
  - ../../.assura/command-surface.yml
---

# Assura Daemon Management CLI

## Objective

Provide a future daemon command family as the shared control plane for humans,
editors, hooks, and agents.

## Scope

- Add JSON-first lifecycle commands for status, start, stop, restart, doctor,
  and logs.
- Use the same daemon client/protocol that editor and agent integrations use.
- Return machine-readable health, project root, config fingerprint, protocol
  version, pid/socket metadata, dirty paths, and remediation commands.
- Keep all state organized under an explicit daemon cache/log area.
- Update `.assura/command-surface.yml`, compatibility docs, support policy,
  and release notes so the new command family is classified as experimental or
  supported before it is advertised.

## Non-Goals

- No VS Code UI in this goal.
- No remote daemon manager.
- No hidden editor-only lifecycle behavior.

## Definition Of Done

- The future daemon status command is stable enough for agents in JSON mode.
- The future daemon doctor command reports actionable remediation in JSON mode.
- CLI start/restart/stop behavior is idempotent and safe for repeated calls.
- Failure modes include exact next commands or one-shot fallback guidance.
- `.assura/command-surface.yml` includes the daemon command family.
- `docs/compatibility-and-surface.md` and `docs/support-policy.md` classify
  the daemon CLI surface consistently.
- Target-state command-surface validation passes.

## Validation Commands

```bash
cargo fmt --check
cargo test daemon_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm all editor/agent lifecycle needs are available through CLI JSON.
- R2: Confirm commands are idempotent and safe in repeated agent workflows.
- R3: Confirm state files do not litter `.assura/` root.
- R4: Confirm command-surface config, compatibility docs, support policy, and
  release notes all classify the daemon CLI surface consistently.

## Reviewer Blocking Criteria

Block if the CLI omits health details needed by agents, exposes editor-only
daemon behavior, or requires users to inspect unstructured logs before a
machine-readable doctor result exists. Also block if command-surface docs or
support policy are not updated with the new public command family.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Started Epic 7 with a management-preview slice for `assura daemon status` and `assura daemon doctor`. The commands expose JSON-first health, protocol, process placeholder, management hints, and doctor remediation over `LocalDaemonCore` without claiming full start/stop/restart/logs lifecycle support yet. Added the carried-forward target-side `daemon references` parity proof. Independent review Pauli found no blocker or high-risk findings; idempotent start/stop/restart/logs lifecycle commands remain open. | `src/cli/daemon.rs`; `src/cli/daemon_management.rs`; `src/cli/daemon_text.rs`; `tests/daemon_cli_tests.rs`; `.assura/command-surface.yml`; `.assura/config.yml`; `docs/compatibility-and-surface.md`; `docs/support-policy.md`; `docs/release-notes.md`; `xtask/src/main.rs`; `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Pauli. |
| 2026-07-01 | Continued Epic 7 with idempotent runtime-metadata lifecycle commands. `daemon start`, `stop`, and `restart` now manage `.assura/daemon/status.json`, `daemon logs` returns bounded `.assura/daemon/daemon.log` lines, and `daemon status` reflects the runtime metadata while keeping `process.running = false` until a real long-running socket/process server exists. Independent review Averroes found no blocker or high-risk findings; suggested hardening added unavailable stop/logs, repeated restart, and log truncation coverage. | `src/cli/daemon.rs`; `src/cli/daemon_lifecycle.rs`; `src/cli/daemon_management.rs`; `tests/daemon_cli_tests.rs`; `.assura/command-surface.yml`; `.assura/config.yml`; `.trellis/spec/assura/daemon-management-cli.md`; `docs/compatibility-and-surface.md`; `docs/support-policy.md`; `docs/release-notes.md`; `xtask/src/main.rs`; `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Averroes. |
