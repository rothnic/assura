---
id: goal-assura-agent-daemon-awareness
type: goal
title: Assura agent daemon awareness
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ./assura-daemon-management-cli.md
  - ./assura-reference-daemon-readiness.md
  - ../support-policy.md
  - ../../.trellis/spec/assura/codex-agent-feedback.md
---

# Assura Agent Daemon Awareness

## Objective

Make local agents aware of daemon health and recovery paths without requiring
MCP, remote access, or broad context injection.

## Scope

- Define a compact daemon-health JSON contract for agents.
- Prefer tool calls or CLI calls for detailed diagnostics instead of injecting
  full daemon state into every prompt.
- Add hook/context snippets that show daemon state, dirty paths, and exact
  remediation commands.
- Support fallback to Assura's existing agent-format check when the daemon is absent
  or unhealthy.
- Document plugin wrappers as optional adapters over the same CLI/protocol.

## Non-Goals

- No per-agent daemon command family.
- No required MCP transport.
- No remote service dependency.
- No automatic daemon mutation unless the user or host policy allows it.

## Definition Of Done

- Agents can detect healthy, unavailable, stale, degraded, and incompatible
  daemon states.
- Agent guidance includes exact recovery commands once the daemon command
  family exists.
- Hook/context output remains bounded and does not exceed existing agent
  context-size expectations.
- Tool/plugin integrations reuse the CLI/protocol contract.

## Validation Commands

```bash
cargo fmt --check
cargo test agent_daemon --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm agents can recover from daemon-down and daemon-stale states.
- R2: Confirm context injection is bounded and tool calls fetch detail on
  demand.
- R3: Confirm no per-agent CLI or required MCP path is introduced.

## Reviewer Blocking Criteria

Block if the integration requires MCP for local use, creates one daemon command
per agent, injects unbounded context, or gives agents no deterministic fallback
when the daemon is down.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Added daemon-aware agent nudge output through the shared `assura agent nudge` command. Nudges include compact daemon health, status/doctor/fallback commands, changed-path daemon checks when paths are supplied, and deterministic fallback output for unavailable projects without requiring MCP or a per-agent daemon command family. | `src/cli/agent_nudge.rs`; `tests/agent_surface_cli.rs`; `cargo test --test agent_surface_cli --quiet`; `cargo test --test daemon_cli_tests --quiet`; `cargo run --quiet -- check --format agent --agent codex .`. |
