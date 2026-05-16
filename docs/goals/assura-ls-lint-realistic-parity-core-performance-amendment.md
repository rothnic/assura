---
id: goal-assura-ls-lint-realistic-parity-core-performance-amendment
type: goal
title: Assura LS-Lint performance completion amendment
status: active
created: 2026-05-16
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/analysis/2026-05-15-performance-architecture-statement.md
  - docs/analysis/2026-05-15-performance-hotspot-optimization-progress.md
  - benches/history/current.json
  - src/cli/check.rs
  - src/cli/performance_report/mod.rs
---

# Assura LS-Lint Performance Completion Amendment

## Purpose

This amendment tightens the PR #11 goal. The work is not complete when the PR
only records performance evidence or uses serial `jwalk`. The goal is to finish
the performance improvements that this PR set out to investigate and prove that
Assura is faster while preserving the project functionality defined by the test
suite and compatibility fixtures.

The technology choice is subordinate to system performance and correctness. The
preferred target is a production checker that can use parallel `jwalk` safely,
because raw traversal benchmarks show parallel walking can help. If parallel
`jwalk` does not win for a fixture class, the implementation must prove that
with data and use an adaptive or better-performing alternative for that class.

## Blocking Goal Update

PR #11 must remain draft and must not be merged until all of these are true:

1. Production `assura check` has a measured traversal strategy that includes a
   real parallel `jwalk` implementation path. `Parallelism::Serial` alone is
   not sufficient to satisfy the goal.
2. The checker architecture is safe for parallel traversal and validation, or
   explicitly separates parallel collection from deterministic serial phases.
   Shared mutable hot-path state must be avoided unless profiling proves the
   synchronization cost is acceptable.
3. The final implementation preserves the existing product contract:
   deterministic JSON/text output, stable relative paths, exclusion pruning,
   fail-fast semantics, sorted violation output, and all current parity and CLI
   behavior covered by tests.
4. Performance results show a confirmed improvement over the current PR baseline
   on the tracked bottleneck fixtures. At minimum, the report must show an
   improvement for rule-heavy validation and at least one traversal-heavy
   realistic fixture, while no stable baseline fixture regresses beyond the
   documented threshold.
5. If a fixture regresses, the PR must either fix it or document a deliberate
   tradeoff with exact numbers and a maintainer decision. Silent regressions are
   not acceptable.
6. LS-Lint comparison timing must measure LS-Lint execution, not repeated npm
   package resolution. The benchmark/report path must install or resolve
   `@ls-lint/ls-lint@2.3.0` once and execute the cached binary in the measured
   loop, or label the metric as cold npm invocation and keep it separate from
   warm tool comparison.
7. `assura performance-report` must expose enough rows to justify the decision:
   top-level Assura and LS-Lint rows, walkdir baseline rows, serial `jwalk`
   rows, parallel `jwalk` rows, and phase rows for config discovery, config
   load, checker init, configured-structure validation, walk-and-validate, and
   report sorting.
8. The PR body must include the final before/after performance table, the chosen
   traversal strategy, the reason for using parallel, serial, or adaptive
   `jwalk`, and a link to the machine-readable artifact or checked-in data.

## Required Implementation Targets

- Refactor the structure checker so most per-entry validation can run from an
  immutable context. Per-entry results should be collected independently and
  sorted once at the end.
- Keep fail-fast deterministic. If full parallel validation conflicts with
  fail-fast semantics, use a serial fail-fast path and a parallel normal path,
  or document and test the chosen split.
- Preserve or improve exclusion pruning before expensive validation work.
- Keep regex and glob compilation out of per-entry hot paths.
- Keep and extend the suffix-pattern fast path for LS-Lint-style naming rules.
- Avoid repeated direct `read_dir` passes where direct child file/directory
  counts can be derived from traversal or a per-directory child summary.
- Fix performance-report history writes so appending history does not read the
  entire existing file, and make website history copy the full intended history
  rather than replacing it with only the latest run unless that behavior is
  explicitly documented as current-only data.
- Preserve fixture integrity when materializing external fixtures, including
  symlink behavior where the source fixture contains symlinks.

## Required Verification

The final PR update must include successful results for:

- `cargo fmt --all -- --check`
- `git diff --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --quiet`
- `cargo run --quiet -- check --format json .`
- realistic LS-Lint fixture tests
- traversal regression tests covering deterministic output, pruning, and
  fail-fast behavior
- `cargo run --quiet -- performance-report --output <artifact> --iterations <n>`
- website build if website performance docs/data changed

The performance report must compare at least:

- current PR baseline before the completion pass
- final implementation after the completion pass
- walkdir traversal baseline
- serial `jwalk`
- parallel `jwalk`
- warm LS-Lint execution through a cached binary

## Stop Condition

Do not stop this PR at "performance measured". Stop only when performance is
measured, improvements are implemented, the final report confirms improvement,
and the functional test suite confirms the project behavior is still correct.

If the final data shows parallel `jwalk` is not the best production default, the
PR may choose an adaptive or alternate implementation, but only after the PR
shows the measured evidence and preserves the goal: best system-level
performance with correct Assura behavior.

## Critical Interpretation Addendum

The PR must not use traversal-only rows as a proxy for full production checker
performance. Reviewers need a written interpretation that separates:

- raw traversal timings (`walkdir`, `jwalk-serial`, `jwalk-parallel`),
- top-level production `assura` timings,
- Assura phase rows,
- LS-Lint warm binary execution timings.

The interpretation must explain fixture intent, especially the difference
between synthetic bottleneck fixtures such as `rule_heavy` and repo-shaped
fixtures such as `rule_heavy_repo`.

If raw traversal rows favor one strategy but top-level Assura rows do not prove
that strategy should be the default, the PR must say so directly. Before the
production default changes again, the performance report should compare
full-check strategy rows for walkdir, serial `jwalk`, and parallel `jwalk`
under equivalent validation work.

## Progress Log

| Date | Iteration | Notes | Evidence |
|------|-----------|-------|----------|
| 2026-05-16 | Continuation setup | Created Trellis continuation task context for the amendment, confirmed current branch `codex/ls-lint-realistic-parity-core-performance`, and preserved unrelated `.codex/config.toml` sandbox config as out-of-scope for product commits. | `.trellis/tasks/05-16-pr-11-performance-amendment/prd.md`; `.trellis/tasks/05-16-pr-11-performance-amendment/implement.jsonl`; `.trellis/tasks/05-16-pr-11-performance-amendment/check.jsonl`; `git status --short --branch` |
| 2026-05-16 | Iteration 1 gap classification | Classified current amendment gaps before editing: production `assura check` still forces serial `jwalk`; traversal report has only walkdir plus one serial `jwalk` row; LS-Lint measured loop repeats `npm exec --package`; history append reads and rewrites the full file; website history writes only latest rows; external fixture copy follows symlinks instead of preserving them. Planned a focused slice covering production traversal, report validity, history persistence, symlink materialization, and regression tests. | `src/cli/check.rs`; `src/cli/performance_report/mod.rs`; `src/cli/performance_report/traversal.rs`; `src/cli/performance_report/io.rs`; `tests/realistic_lslint_fixtures.rs`; `tests/cli_check_tests.rs` |
| 2026-05-16 | Iteration 2 implementation and evidence | Added a separated traversal module with deterministic serial fail-fast/default validation plus an opt-in parallel `jwalk` collection path, expanded report rows to `walkdir`, `jwalk-serial`, and `jwalk-parallel`, prepared LS-Lint once before measured loops, switched history append to streaming append, copied full website history when a history source is provided, preserved external fixture symlinks, and regenerated checked-in performance data. Local same-machine baseline comparison showed final rule-heavy improvement (`rule_heavy` 194.615 ms -> 167.611 ms; `rule_heavy_repo` 29.797 ms -> 22.148 ms) and traversal-heavy improvement (`ignored_generated_heavy` 0.752 ms -> 0.482 ms); raw parallel `jwalk` rows also improved traversal-heavy evidence while full validation remains serial by default. | `benches/history/current.json`; `benches/history/ls-lint-comparison-history.jsonl`; `website/public/data/performance/current.json`; `website/public/data/performance/ls-lint-comparison-history.jsonl`; `target/performance/pr11-amendment-default-v2.json`; temporary baseline worktree report `target/performance/pr11-baseline-local.json` |
| 2026-05-16 | Iteration 3 context health and skill capture | Context level: active goal reports unbounded remaining tokens; relevant prior messages are persisted in the amendment progress log, Trellis task PRD/context files, checked-in performance artifacts, and validation command outputs. Captured reusable performance-report workflow in a project skill and kept `AGENTS.md` as a one-line router. | `.agents/skills/assura-performance-reporting/SKILL.md`; `AGENTS.md`; `cargo run --quiet -- check --format json .` |
| 2026-05-16 | Final review and validation | Required review agent found no blocking code issues and flagged one residual test gap for the opt-in parallel traversal path; added `check_parallel_jwalk_traversal_env_path_preserves_sorted_json_output` to cover it. Final local gates passed after the fix. | `cargo fmt --all -- --check`; `git diff --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --quiet`; `cargo run --quiet -- check --format json .`; `cargo run --quiet -- performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 5`; `cd website && pnpm build` |
| 2026-05-16 | Critical interpretation follow-up | Clarified that traversal-only rows cannot justify the production default by themselves, explained `rule_heavy` versus `rule_heavy_repo`, documented what warm LS-Lint rows measure, and added an explicit follow-up requirement for full-check strategy comparisons before changing the default again. | `docs/analysis/2026-05-16-performance-results-interpretation.md`; `website/src/content/docs/reference/performance.mdx`; PR #11 body |
