---
id: goal-assura-roadmap-06-review-evidence-and-quality-gates
type: goal
title: Assura roadmap 06 review evidence and quality gates
status: completed
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/analysis/
---

# Goal 06: Review Evidence And Quality Gates

## Objective

Make goal completion auditable by standardizing review records, generated
evidence, validation commands, and CI gates across Assura PRs.

This is a two-week team chunk for workflow, CI, docs, and reviewer owners.

## Scope

- Define a review-record template under `docs/analysis/`.
- Define what evidence must be checked in versus generated in `target/`.
- Add scripts or concise commands for reproducing reports and feedback
  artifacts.
- Align PR templates with goal docs, review tasks, and validation gates.
- Add CI coverage for docs links, goal metadata, and forbidden stale command
  surfaces where practical.
- Document how to handle known baseline issues without hiding new regressions.

## Non-Goals

- No new external hosted quality service unless explicitly justified.
- No replacement for Trellis as task/spec workflow.
- No broad issue tracker migration.

## Definition Of Done

- New goal PRs have a standard review record format.
- PR descriptions consistently link goal docs and evidence.
- CI or repo scripts catch stale forbidden command surfaces where possible.
- Review-agent findings are recorded and resolved or rejected with rationale.
- The workflow documents how to close a goal and name the next goal path.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
node --run verify:fast
node --run verify:docs
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm templates reinforce Trellis rather than replacing it.
- R1: Review CI/script changes for false positives and local reproducibility.
- R2: Review documentation examples against current PR flow.
- R3: Run evidence reproduction commands from the PR body.
- R4: Review completed PRs for whether the new template would have caught past
  misses.
- R5: Confirm the next-goal handoff is copy/paste-ready.

## Reviewer Blocking Criteria

Block the PR if evidence can only be understood from chat history, if templates
create a second workflow system, or if CI checks are too noisy to be useful.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-02 | Started Goal 06 from updated `master` after Goal 05 merged; moved the active Trellis task to `codex/phase-01-goal-06-review-evidence-gates`. | `gh pr view 22 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-06-review-evidence-gates`; `python3 ./.trellis/scripts/task.py set-branch 06-01-roadmap-phase-01-execution codex/phase-01-goal-06-review-evidence-gates`. |
| 2026-06-02 | Added the initial Goal 06 evidence gate slice: review-record template, evidence policy, PR template, `node --run verify:evidence`, and CI evidence-gates job for review metadata, markdown links, goal frontmatter, and stale command-surface checks. | `docs/analysis/review-record-template.md`; `docs/analysis/evidence-and-review-policy.md`; `.github/PULL_REQUEST_TEMPLATE.md`; `scripts/verify.sh`; `.github/workflows/ci.yml`. |
| 2026-06-02 | Added the Goal 06 review record with backfilled PR #21 and PR #22 evidence examples, then proved the new artifact with the evidence gate. | `docs/analysis/2026-06-02-goal-06-review-evidence-gates-review.md`; `node --run verify:evidence`. |
| 2026-06-02 | Addressed review-agent findings by broadening stale-surface detection, narrowing markdown link checks to current evidence, and recording review feedback closure. | Review agent `019e8636-de87-7941-b958-48433238b284`; `node --run verify:evidence`; `node --run verify:fast`. |
| 2026-06-02 | Completed Goal 06 via merged PR #23 and moved Iteration 01 execution to Goal 07 on `codex/phase-01-goal-07-extension-plugin-foundation`. | `gh pr view 23 --json state,mergedAt,mergeCommit,url`; CI `Evidence Gates` plus existing checks passed on amended commit `b96a01b`. |
