---
id: goal-assura-beta-agent-nudge-integrations
type: goal
title: Assura beta agent nudge integrations
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-agent-nudge-mvp.md
  - ./assura-agent-daemon-awareness.md
  - ../../integrations/agents/codex/
  - ../../docs/support-policy.md
---

# Assura Beta Agent Nudge Integrations

## Objective

Provide beta-grade local agent nudges for Codex, OpenCode, Claude, and Pi
agents so each can use Assura efficiently without flooding context or breaking
caching. Nudges should appear only when repository feedback is likely to help
the next action.

## Current Gap

The stable feedback API is `assura check --format agent`, with Codex-specific
delivery available through `--agent codex`. Beta needs a broader integration
contract for multiple agents and event timings, while keeping the shared Assura
agent output as the product surface instead of one custom CLI per agent.

## Scope

- Define a shared nudge payload for structure, content, markdown, reference,
  daemon health, and performance-gate findings.
- Support Codex, OpenCode, Claude, and Pi agent adapters or documented hook
  recipes over the shared payload.
- Define event timing policy:
  - before tool calls when the next tool is likely to edit or inspect affected
    paths;
  - after tool calls when changed files create new Assura findings;
  - on session start only as a compact health summary;
  - never as broad repeated context injection.
- Keep nudges concise: affected paths, rule IDs, severity, one suggested
  command, and a pointer to deeper output.
- Respect caching: avoid volatile timestamps or large payloads in injected
  context unless the caller explicitly asks for diagnostics.
- Use daemon health and changed-path context when available; fall back to
  one-shot `assura check --format agent`.

## Non-Goals

- No hosted agent service.
- No mandatory MCP server.
- No per-agent validation logic that bypasses Assura's shared contracts.
- No automatic repair without explicit tool/user approval.

## Definition Of Done

- Each target agent has a documented local integration path and test fixture or
  executable smoke where practical.
- Nudge payloads are bounded and stable.
- Event timing avoids noisy repeated injection and preserves cacheability.
- Daemon-backed nudges and one-shot fallback use the same finding semantics.
- Support docs classify adapters as supported, experimental, or future.

## Validation Commands

```bash
cargo fmt --check
cargo test agent_feedback --quiet
cargo test daemon --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo xtask docs
cargo xtask evidence
git diff --check
```

Add adapter-specific package tests once OpenCode, Claude, and Pi agent adapter
packages or scripts exist.

## Review Tasks

- R1: Confirm no agent gets a private validation path.
- R2: Confirm nudges are compact enough for frequent agent events.
- R3: Confirm caching is not broken by volatile or oversized context.
- R4: Confirm fallback behavior works when the daemon is unavailable.

## Reviewer Blocking Criteria

Block if integrations duplicate Assura logic, require hosted services, inject
large diagnostics by default, run on every event without relevance filtering, or
revive per-agent CLI families instead of the shared agent output contract.
