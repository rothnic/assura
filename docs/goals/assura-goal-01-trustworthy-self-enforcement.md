---
id: goal-assura-roadmap-01-trustworthy-self-enforcement
type: goal
title: Assura roadmap 01 trustworthy self-enforcement
status: active
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .assura/config.yml
---

# Goal 01: Trustworthy Self-Enforcement

## Objective

Make Assura's own repository a reliable proof that `assura check` can enforce
real project structure without noisy exceptions, stale workflow state, or
manual tribal knowledge.

This is a two-week team chunk for core CLI, docs, and workflow owners. The
result should be a clean self-check baseline, current Trellis state, and a
documented operating model for keeping the repo clean after future PRs.

## Scope

- Close or archive completed Trellis tasks that still appear active.
- Align `.trellis/spec/assura/roadmap.md`, active goal docs, and current PR
  state.
- Audit `.assura/config.yml` against current repository shape.
- Remove or document stale active docs that point to OpenSpec, `specs-bak`, old
  feedback command surfaces, or retired performance claims.
- Add targeted self-check tests or fixtures where the current config has blind
  spots.
- Document the local and CI gates that must preserve self-enforcement.

## Non-Goals

- No new Assura policy language beyond what is required to express this repo.
- No broad website redesign.
- No dependency graph validation.
- No automatic mutation of user-level Codex settings.

## Definition Of Done

- `cargo run --quiet -- check --format json .` reports zero violations.
- The active Trellis task list no longer includes completed merged work.
- The roadmap names the current active epic, task, branch, and next goal.
- Every active goal status is one of planned, active, completed, or archived and
  matches current repo truth.
- Stale command surfaces are either removed or explicitly marked historical.
- A reviewer can reproduce the self-check baseline from a clean checkout.

## Required Validation

```bash
git status --short --branch
python3 ./.trellis/scripts/get_context.py
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Verify the goal doc and Trellis task are the current source of truth.
- R1: Review `.assura/config.yml` for accidental overfitting or missing current
  directories.
- R2: Review task/archive changes for accidental removal of active work.
- R3: Reproduce self-check from a clean checkout.
- R4: Read docs that explain self-enforcement and confirm they do not reference
  stale workflow systems as active.
- R5: Confirm the PR links this goal and includes current task/roadmap evidence.

## Reviewer Blocking Criteria

Block the PR if self-check is green only because exclusions hide active source
or docs, if completed tasks remain active without explanation, or if roadmap
state cannot tell the next agent what to do.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-01 | Started Goal 01 on fresh branch `codex/roadmap-phase-01-execution`; established `.trellis/tasks/06-01-roadmap-phase-01-execution` as current task; archived completed/merged Trellis tasks after checking live PR state. | `python3 ./.trellis/scripts/task.py current --source`; GitHub PRs #1-#5, #8, #11, #12, #17 are merged; `python3 ./.trellis/scripts/task.py list` now shows only the current Phase 01 task active. |
| 2026-06-01 | Opened Goal 01 review PR after local validation and review-agent pass. | PR #18: `https://github.com/rothnic/assura/pull/18`; review agent `019e853b-84df-76a1-b03f-2c3a4d711fc7`; `node --run verify:fast`; `cargo test --all-targets --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `node --run verify:docs`; `git diff --check`. |
