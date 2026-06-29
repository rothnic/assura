---
id: goal-assura-project-intelligence-persistent-session
type: goal
title: Assura project intelligence persistent session
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-real-repo-proof.md
  - website/src/content/docs/reference/agent-feedback.md
  - docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md
---

# Assura Project Intelligence Persistent Session

## Objective

Promote a measured warm-session or watch-backed workflow that makes repeated
agent/editor checks and project-intelligence queries fast enough to use during
normal editing.

## Current Gap

Lower-level prepared-check and hot-session concepts exist, but public
project-intelligence commands are cold CLI invocations. `assura watch` is still
experimental, and docs say future editor or agent integrations should reuse
prepared checks or a hot daemon state.

## Scope

- Decide whether the first usable surface is watch hardening, an explicit
  session command, a daemon, or an internal API promoted for wrappers.
- Reuse prepared validation, content model loading, graph facts, and search
  state when safe.
- Define config and filesystem invalidation rules.
- Provide changed-path or changed-content behavior with fallback to full checks.
- Measure cold CLI, warm session, changed-path, and dirty-project rows.
- Keep session state local and disposable.

## Non-Goals

- No hosted daemon.
- No editor-specific protocol in this goal.
- No correctness dependency on filesystem watcher delivery alone.
- No cache that can hide config or content changes.

## Definition Of Done

- A public or documented integration surface reuses validation/query state for
  repeated local workflows.
- Config changes, model changes, content changes, and ambiguous watcher events
  invalidate or fall back conservatively.
- Benchmarks compare cold and warm paths on Assura and the real-repo proof
  package.
- `assura watch` support status is either promoted with tests/docs or remains
  explicitly experimental with a documented reason.
- Agent/editor transport goals can call this surface without duplicating state
  management.

## Validation Commands

```bash
cargo fmt --check
cargo test watch --quiet
cargo test prepared_check --quiet
cargo test project_intelligence_store --quiet
cargo bench --bench project_intelligence
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R1: Confirm warm-session correctness falls back safely when freshness is
  uncertain.
- R2: Confirm benchmark rows compare the right user-visible workflows.
- R3: Confirm no session state is required for ordinary one-shot CLI checks.
- R4: Confirm docs accurately classify watch/session support.

## Reviewer Blocking Criteria

Block if cached or session results can mask changed config/content, if watcher
events are treated as the only correctness signal, if performance evidence is
missing, or if the goal promotes a daemon before the reuse benefit is measured.
