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
