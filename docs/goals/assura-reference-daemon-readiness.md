---
id: goal-assura-reference-daemon-readiness
type: goal
title: Assura reference daemon readiness
status: completed
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-code-doc-reference-validation.md
  - ./assura-markdown-reference-intelligence-program.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ./assura-daemon-management-cli.md
  - ../project-intelligence-facts.md
  - ../../.trellis/tasks/07-01-07-01-reference-daemon-readiness/prd.md
---

# Assura Reference Daemon Readiness

## Objective

Make the local daemon/session layer reliable enough to serve repeated
Markdown-reference checks, affected-path feedback, VS Code diagnostics, and
agent tools without requiring a hosted service.

## Current Gap

Reference Graph is complete for beta: one-shot checks and content queries can
produce repository-reference facts, inbound/outbound affected context, and
opt-in broken-reference diagnostics. The remaining beta gap is that repeated
agent/editor/hook workflows still need to invoke cold one-shot checks or
content-query sessions. A daemon-ready core must hold warm project state,
detect when that state is stale, and provide bounded affected-path feedback
that stays consistent with one-shot truth.

## User Certainty Bar

A user or agent should be able to ask the daemon for current project health and
changed-path feedback and know whether the answer came from fresh state,
warming state, stale state, degraded state, or a required one-shot fallback:

- changed-source feedback should name affected outbound reference targets;
- changed-target feedback should name inbound source references before a move
  or delete;
- config changes should invalidate cached state instead of reporting stale
  success;
- watcher misses and unreadable state should be visible and actionable;
- every daemon answer should preserve the command needed to fall back to
  one-shot `assura check`.

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
- No daemon management CLI command family beyond what is needed to prove core
  state contracts; full status/start/stop/restart/doctor/logs belongs to
  `assura-daemon-management-cli.md`.
- No MCP or remote daemon requirement.
- No per-agent validation logic.
- No silent automatic repair.

## Definition Of Done

- Daemon health distinguishes running, warming, stale, degraded, unavailable,
  and incompatible states.
- Changed-source and changed-target daemon/session checks prove bounded
  affected-reference feedback and agree with `assura content references`.
- Config changes invalidate cached state.
- Logs and status metadata are stored in organized locations when a runtime
  process is introduced; earlier core slices must at least define those paths.
- One-shot `assura check` remains the fallback truth path.
- Agent/editor-facing outputs are JSON-capable and do not require remote
  services.

## Validation Commands

```bash
cargo fmt --check
cargo test --test daemon_core_tests --quiet
cargo test --test daemon_cli_tests --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo test --test repository_reference_check_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm daemon results match one-shot check results for the same inputs.
- R2: Confirm stale cache and config fingerprint failures are visible.
- R3: Confirm changed-path feedback is bounded and correct.
- R4: Confirm daemon/core outputs remain shared across CLI, editor, hooks, and
  agents without per-agent validation branches.

## Reviewer Blocking Criteria

Block if the daemon can report clean results from stale state, requires remote
access, hides watcher misses, or cannot fall back to one-shot validation.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Revalidated Epic 6 after completing Reference Graph. The goal remains valid: Reference Graph provides one-shot facts, queries, and diagnostics, while beta still lacks a warm daemon-ready core with freshness states, affected-path feedback, and explicit one-shot fallback contracts. Created Trellis task `07-01-07-01-reference-daemon-readiness` for execution. | [assura-code-doc-reference-validation.md](./assura-code-doc-reference-validation.md); [assura-beta-code-agnostic-capabilities-program.md](./assura-beta-code-agnostic-capabilities-program.md); `.trellis/tasks/07-01-07-01-reference-daemon-readiness/prd.md`; `python3 ./.trellis/scripts/workflow_gate.py --platform codex`; `git status --short --branch`. |
| 2026-07-01 | Added the first daemon-ready core contract as shared Rust state, not a managed background process. `LocalDaemonCore` now exposes JSON-serializable health states, project-local status/log paths, one-shot fallback commands, prepared changed-path structure checks, bounded source/target repository-reference responses, config-change stale failures, degraded target-delete feedback from the prior warm graph, and runtime-file fingerprint exclusions. | `src/daemon/mod.rs`; `src/daemon/fingerprint.rs`; `tests/daemon_core_tests.rs`; `cargo test --test daemon_core_tests --quiet`; `cargo test --lib daemon::tests::health_states_are_serialized_for_clients --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo fmt --check`. |
| 2026-07-01 | Hardened the daemon-ready core after independent review. Explicit config fallback now preserves `--config`, configs outside `.assura/` keep the checked project root, missing config returns structured stale health, source mutation output is compared against `assura content references`, target moves have old/new path context, and observable warming/unavailable/incompatible health responses are tested. Reviewer Bacon found no remaining blocker or high-risk findings. | `src/cli/content_query/context.rs`; `src/daemon/types.rs`; `tests/daemon_core_tests.rs`; `cargo test --test daemon_core_tests --quiet`; `cargo test --lib daemon::tests::health_states_are_serialized_for_clients --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Bacon. |
| 2026-07-01 | Added a narrow JSON-capable `assura daemon` probe surface over `LocalDaemonCore` without introducing process management. The probe exposes `daemon health`, `daemon check-path`, and `daemon references` for source, target, and moved-target context; command-surface and support-matrix metadata classify the commands as experimental while keeping full daemon mode on the roadmap for Epic 7. Galileo review found and confirmed fixes for pre-load reference-flag validation, structured JSON unavailable health output, and replacing the broad daemon test filter with explicit daemon tests. | `src/cli/daemon.rs`; `src/cli/content_args.rs`; `.assura/command-surface.yml`; `docs/data/release-surfaces.json`; `tests/daemon_cli_tests.rs`; `cargo fmt --check`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test daemon_core_tests --quiet`; `cargo test --test cli_command_surface_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `cargo xtask docs`; `cargo check --workspace --all-targets --quiet`; `git diff --check`; independent review Galileo. |
| 2026-07-01 | Completed Epic 6 after closure review. Reviewer Locke found no blockers or high-risk gaps and confirmed daemon-ready health states, config-stale behavior, bounded affected-reference feedback, runtime path definitions, one-shot fallback metadata, local JSON-capable probe outputs, and the absence of full process-management commands. The only residual low-risk follow-up is to add explicit `daemon references --target` versus `content references --target` CLI parity coverage during Epic 7. | `cargo fmt --check`; `cargo test --test daemon_core_tests --quiet`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test repository_reference_graph_tests --quiet`; `cargo test --test repository_reference_check_tests --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask evidence`; `git diff --check`; independent closure review Locke. |
