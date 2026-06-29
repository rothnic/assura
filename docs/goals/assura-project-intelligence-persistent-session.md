---
id: goal-assura-project-intelligence-persistent-session
type: goal
title: Assura project intelligence persistent session
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
  - docs/goals/assura-project-intelligence-context-pack.md
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
project-intelligence commands are cold CLI invocations. After onboarding and
context-pack work, repeated checks should reuse the same content model,
validation, graph/search, and safe-fix preview state instead of rebuilding it
for every editor or agent request. `assura watch` is still experimental, and
docs say future editor or agent integrations should reuse prepared checks or a
hot daemon state.

## Scope

- Decide whether the first usable surface is watch hardening, an explicit
  session command, a daemon, or an internal API promoted for wrappers.
- Reuse prepared validation, content model loading, graph facts, search state,
  context-pack assembly, and safe-fix preview inputs when safe.
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
cargo test --test project_intelligence_session --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo test --test content_query_cli --quiet
cargo test --test content_runtime_dx_docs project_intelligence_demo_is_discoverable_and_covers_adoption_commands --quiet
cargo bench --bench project_intelligence -- --sample-size 10 session_reuse
cargo test project_intelligence_store --quiet
cargo bench --bench project_intelligence
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
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

## Progress Log

- 2026-06-29: Completed locally on task
  `.trellis/tasks/06-29-project-intelligence-persistent-session`. Added
  `assura content session`, a local JSON-line request/response loop that keeps
  one project-intelligence context loaded for repeated diagnostics,
  context-pack, collection, search, graph expansion, missing-relations, and
  safe-fix preview requests. Each response uses
  `assura.project-intelligence.session.response.v1` and reports reload state
  as `initial_load`, `reused`, `reloaded`, `reload_failed`, or `not_checked`.
  The session scans a conservative project fingerprint before every request
  and reloads before answering when project files change, so correctness does
  not depend on watcher events. `assura watch` remains experimental.
- 2026-06-29: Focused proof added in
  `tests/project_intelligence_session.rs` for repeated context reuse, invalid
  request recovery, reload after modeled frontmatter changes, and reload before
  the first response when files change after startup. Live smoke verified two
  requests through `cargo run --quiet -- content session .`: diagnostics
  returned `initial_load`; a context-pack request returned `reused`. Benchmark
  evidence from
  `cargo bench --bench project_intelligence -- --sample-size 10 session_reuse`
  compared cold fact loading to warm reusable fact-store query operations on
  the Assura repo and Beacon CRM fixture. The public JSON-line parsing,
  serialization, reload, and error envelope are covered by integration tests
  and live smoke rather than the Criterion microbenchmark.
- 2026-06-29: Independent review found two blocking correctness risks: changed
  files could be missed before the first request, and same-length writes could
  evade timestamp/length fingerprints. Both were fixed by checking freshness
  before the first response and hashing file contents in the project
  fingerprint.
