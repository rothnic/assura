---
id: goal-assura-roadmap-04-fast-incremental-check-engine
type: goal
title: Assura roadmap 04 fast incremental check engine
status: completed
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/goals/assura-cli-to-cli-ls-lint-performance-verification.md
---

# Goal 04: Fast Incremental Check Engine

## Objective

Make repeated checks fast enough for same-turn agent feedback while preserving
deterministic full-project correctness and truthful performance evidence.

This is a two-week team chunk for engine, benchmark, and docs owners.

## Scope

- Define the product contract for full-project checks, changed-path checks, and
  prepared checker reuse.
- Measure cold CLI, warm session, changed-path, and process-floor rows on the
  real-project feedback scenario and pinned realistic fixtures.
- Implement or harden rule planning, scope indexing, direct-count summaries, and
  compiled pattern reuse.
- Preserve deterministic JSON/text output ordering.
- Document which performance claims are public and which rows are diagnostic.
- Update checked performance evidence only through the established workflow.

## Non-Goals

- No hosted daemon as default user experience.
- No synthetic-only headline claim.
- No broad LS-Lint comparison rewrite unless the evidence contract requires it.

## Definition Of Done

- Repeated same-turn checks have measured evidence and an explicit acceptable
  latency threshold.
- The primary local-agent threshold is concrete: on the checked real-project
  fixture, 30 consecutive warm prepared checks must report p95 latency at or
  below 250 ms, changed-path checks touching five files must report p95 latency
  at or below 100 ms, and cold CLI checks must not regress by more than 10% from
  the checked baseline captured before implementation.
- The checked performance artifact records hardware, OS, Rust version, command
  line, run count, median, p95, max, and baseline comparison for cold CLI, warm
  prepared checker, changed-path checker, and process-floor rows.
- Changed-path checks do not silently claim whole-project success unless they
  prove it.
- Full-project checks remain deterministic and correct.
- Wildcard scope checks do not multiply full-tree traversals as policy size
  grows.
- Website and docs distinguish cold CLI, warm, changed-path, and diagnostic
  evidence.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --release --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
cd website && npx pnpm@10.25.0 build
git diff --check
```

## Review Tasks

- R0: Confirm performance work follows `.agents/skills/assura-performance-reporting/SKILL.md`.
- R1: Review the evidence contract before reviewing implementation.
- R2: Review hot-path changes for duplication, nondeterminism, and hidden
  allocation growth.
- R3: Reproduce checked performance artifacts, compare website data, and confirm
  the 30-run p95 thresholds, baseline comparison, and hardware metadata are
  present in committed evidence.
- R4: Review website language for overstated claims.
- R5: Confirm any accepted regressions have explicit owner and follow-up.

## Reviewer Blocking Criteria

Block the PR if a headline claim is synthetic-only, if changed-path checks are
misrepresented as full validation, or if deterministic output changes without a
test and documented reason. Also block if the warm same-turn p95 threshold is
missed and the PR attempts to redefine the target after implementation.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-01 | Started Goal 04 from updated `master` after Goal 03 merged; loaded the performance-reporting workflow and moved Phase 01 execution to `codex/phase-01-goal-04-incremental-check-engine`. | `gh pr view 20 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-04-incremental-check-engine`; `.agents/skills/assura-performance-reporting/SKILL.md`; `python3 ./.trellis/scripts/task.py set-branch 06-01-roadmap-phase-01-execution codex/phase-01-goal-04-incremental-check-engine`. |
| 2026-06-02 | Added the Goal 04 evidence contract for prepared full-project and five-path changed-path checks, including hardware metadata, command-line recording, dirty-worktree provenance, p95 rows, threshold fields, and whole-project-success labeling. Generated a 30-run default proof, a 30-run opt-in pinned external fixture proof with all 10 pinned repositories, and a 30-run pre-Goal-04 baseline comparison from commit `56937c6`. Aggregate cold `assura-cli` and `assura-check-cli` medians improved versus the adjacent baseline; the proof note preserves row-level variance and one noisy small-fixture outlier for review. | `docs/analysis/2026-06-01-goal-04-fast-incremental-check-proof.json`; `docs/analysis/2026-06-01-goal-04-fast-incremental-check-external-fixtures-proof.json`; `docs/analysis/2026-06-01-goal-04-fast-incremental-check-review.md`; `cargo test prepared --quiet`; `cargo test performance_report --quiet`; `cargo test --test performance_report_contract_tests --quiet`; `cargo run --quiet -- check --format agent . --warn`. |
| 2026-06-02 | Completed Goal 04 via merged PR #21 and moved Iteration 01 execution to Goal 05 from updated `master`. | `gh pr view 21 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-05-installable-adoption-path`; `python3 ./.trellis/scripts/task.py set-branch 06-01-roadmap-phase-01-execution codex/phase-01-goal-05-installable-adoption-path`. |
