---
id: goal-assura-reference-daemon-readiness
type: goal
title: Assura reference daemon readiness
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ../project-intelligence-facts.md
---

# Assura Reference Daemon Readiness

## Objective

Make the local daemon/session layer reliable enough to serve repeated
Markdown-reference checks, affected-path feedback, VS Code diagnostics, and
agent tools without requiring a hosted service.

## Scope

- Reuse one-shot validation and project-intelligence logic.
- Maintain prepared project state, config fingerprints, and reference graph
  generations.
- Track inbound and outbound reference edges so source or target changes can
  produce bounded affected-reference feedback.
- Handle file watcher events, missed events, config changes, restarts, and
  stale caches conservatively.
- Expose health states that CLI, VS Code, and agents can understand.

## Non-Goals

- No marketplace editor package in this goal.
- No MCP or remote daemon requirement.
- No silent automatic repair.

## Definition Of Done

- Daemon health distinguishes running, warming, stale, degraded, unavailable,
  and incompatible states.
- Changed-source and changed-target checks prove bounded affected-reference
  feedback.
- Config changes invalidate cached state.
- Logs and status metadata are stored in organized locations.
- One-shot `assura check` remains the fallback truth path.

## Validation Commands

```bash
cargo fmt --check
cargo test daemon --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm daemon results match one-shot check results for the same inputs.
- R2: Confirm stale cache and config fingerprint failures are visible.
- R3: Confirm changed-path feedback is bounded and correct.

## Reviewer Blocking Criteria

Block if the daemon can report clean results from stale state, requires remote
access, hides watcher misses, or cannot fall back to one-shot validation.

