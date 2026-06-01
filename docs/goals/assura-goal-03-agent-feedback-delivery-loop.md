---
id: goal-assura-roadmap-03-agent-feedback-delivery-loop
type: goal
title: Assura roadmap 03 agent feedback delivery loop
status: active
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/codex-agent-feedback.md
  - docs/goals/assura-real-project-policy-proof.md
---

# Goal 03: Agent Feedback Delivery Loop

## Objective

Turn stable agent feedback output into a complete local delivery loop that
agents can run, inspect, and measure without adding per-agent command surfaces.

This is a two-week team chunk for CLI, Codex integration, tests, and docs.

## Scope

- Keep `assura check --format agent` as the stable structured feedback output.
- Keep Codex delivery under `assura check --format agent --agent codex`.
- Add or harden examples for generic agent JSON, Codex hook JSON, advisory
  mode, blocking mode, severity filtering, and max-issue filtering.
- Improve same-turn observation evidence so feedback usefulness is reproducible
  from checked reports.
- Document append-only Codex hook configuration examples without mutating user
  settings.
- Add review artifacts showing an agent received feedback, fixed drift, and
  reran the check.

## Non-Goals

- No `assura-codex-feedback` binary.
- No `assura check --format codex-hook`.
- No automatic Codex `/hooks` approval or user-level config mutation.
- No hosted telemetry.

## Definition Of Done

- Generic agent JSON has a stable schema, tests, and docs.
- Codex delivery emits valid `UserPromptSubmit` hook JSON with bounded
  `additionalContext`.
- Advisory and blocking exit behavior are covered by integration tests.
- Feedback evidence records violation class, feedback count, fixed-before-new
  turn status, usefulness, remaining violations, and repeat feedback count.
- The checked same-turn feedback proof includes at least 10 seeded policy
  violations across three real-project scenarios: generic agent JSON, Codex hook
  JSON, and advisory mode. Every Critical and High violation must appear in the
  first feedback response, at least 80% of Medium violations must appear before
  truncation, and all included entries must identify rule id, file path, severity,
  message, and one actionable remediation hint.
- Codex `additionalContext` stays under 24 KiB for the seeded proof and remains
  deterministic across two consecutive runs with the same inputs.
- Usefulness is not self-assigned by the implementation: the checked proof must
  show that an agent or scripted fixer corrected at least 8 of 10 seeded
  violations before a new prompt turn, and repeated feedback for the same fixed
  violation is zero on the follow-up run.
- Docs state Codex prerequisites: `features.hooks = true` and one-time `/hooks`
  approval.
- Package helpers remain lower-level library support, not the public CLI.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --test cli_command_surface_tests --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --all-targets --quiet
cargo run --quiet -- check --format agent . --warn
cargo run --quiet -- check --format agent --agent codex . --warn
cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run
git diff --check
```

## Review Tasks

- R0: Confirm the Codex feedback spec is current before reviewing code.
- R1: Review CLI signature and schema compatibility.
- R2: Review tests for old forbidden surfaces and adapter misuse.
- R3: Reproduce checked feedback artifacts from documented commands and confirm
  the evidence contains the 10-violation seeded matrix, deterministic output
  comparison, 24 KiB Codex payload limit, and fixed-before-new-turn result
  counts.
- R4: Review website/docs for unsupported automation claims.
- R5: Confirm review-agent and Gemini findings are addressed.

## Reviewer Blocking Criteria

Block the PR if it adds a package feedback CLI, a per-agent CLI entrypoint, a
per-agent format, or docs that imply Codex hook enablement can be automated.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-01 | Started Goal 03 from updated `master` after Goal 02 merged; preserved the stable public surface as `assura check --format agent` with Codex delivery only through `--agent codex`. | `gh pr view 19 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-03-feedback-loop`; `.trellis/spec/assura/codex-agent-feedback.md`. |
| 2026-06-01 | Expanded the real-project feedback fixture to 12 seeded violations, prioritized Critical/High feedback before max-issue truncation, carried corrective context through package helpers, and generated checked Goal 03 proof artifacts. | `cargo test --test real_project_agentic_feedback_tests --quiet`; `cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run`; `docs/analysis/2026-06-01-goal-03-agent-feedback-delivery-proof.json`. |
| 2026-06-01 | Completed the full local Goal 03 validation chain and reframed the master roadmap artifact as bounded Phase 01 control-plane work, so closing the eight linked goals completes the phase rather than the broader Assura roadmap. | `cargo fmt --all -- --check`; `cargo test --all-targets --quiet`; `cargo run --quiet -- check --format agent . --warn`; `cargo run --quiet -- check --format agent --agent codex . --warn`; `node --run verify:docs`; `bash scripts/verify.sh fast`; `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`. |
