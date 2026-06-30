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
