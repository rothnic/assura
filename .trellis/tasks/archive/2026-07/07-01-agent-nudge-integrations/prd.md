# Agent Nudge Integrations

## Problem

The beta program now has daemon-ready local state and daemon management CLI
contracts, but agents still need bounded event-aware guidance that uses those
contracts without duplicating validation logic or injecting broad context on
every event.

## Goal

Execute `docs/goals/assura-beta-agent-nudge-integrations.md` as Epic 8 of the
beta program, with `docs/goals/assura-agent-daemon-awareness.md` as the daemon
health companion goal. Deliver concise local nudge contracts and documented
Codex, OpenCode, Claude, and Pi integration paths over shared Assura CLI output.

## Scope

- Define the shared nudge payload for structure, content, Markdown, repository
  references, daemon health, and performance-gate findings.
- Reuse `assura check --format agent`, `assura daemon status/doctor`, and
  changed-path daemon commands instead of adding per-agent validation paths.
- Document local hook/plugin recipes for Codex, OpenCode, Claude, and Pi.
- Keep nudge payloads bounded: affected paths, rule IDs, severity, one
  suggested command, and a pointer to deeper output.
- Preserve cacheability by avoiding volatile timestamps and large diagnostics
  in default injected context.
- Classify the adapter surfaces in support and compatibility docs.

## Non-Goals

- No hosted service.
- No required MCP server.
- No per-agent CLI families.
- No automatic repair or daemon mutation without explicit host/user policy.

## Validation

Use narrow checks while iterating:

```bash
cargo fmt --check
cargo test agent_feedback --quiet
cargo test --test daemon_cli_tests --quiet
cargo run --quiet -- check --format agent --agent codex .
git diff --check
```

Before completion, also run:

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
```

## Review

Complex implementation slices require independent review. Ask reviewers to
focus on bounded context, event relevance, cacheability, deterministic daemon
fallbacks, and whether any agent integration bypasses the shared Assura
finding contracts.
