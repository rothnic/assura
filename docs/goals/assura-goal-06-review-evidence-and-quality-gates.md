---
id: goal-assura-roadmap-06-review-evidence-and-quality-gates
type: goal
title: Assura roadmap 06 review evidence and quality gates
status: planned
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
