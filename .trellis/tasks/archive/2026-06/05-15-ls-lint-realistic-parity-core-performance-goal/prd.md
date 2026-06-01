# Define LS-Lint realistic parity and core performance goal

## Goal

Create the next durable Codex goal for Assura focused on robust LS-Lint
compatibility testing against realistic example repositories, production use of
`jwalk` for the core `assura check` traversal path, and investigation of the
main current-product performance hotspots before moving into richer notation
extensions.

## What I Already Know

- The user wants to deprioritize agent nudge proof work for the next goal.
- The next goal should focus on LS-Lint parity coverage using realistic simple
  and comprehensive repository fixtures.
- The user explicitly wants production `assura check` switched from `walkdir`
  to faster `jwalk`; this was the intended traversal strategy.
- The goal should investigate additional performance improvements around known
  hotspots rather than assuming traversal is the only bottleneck.
- Existing docs include active and historical notation material:
  `docs/unified-tree-design.md`, `docs/ls-lint-capability-comparison.md`,
  `docs/archive/final-config-design.md`, and
  `docs/archive/ls-lint-notation-guide.md`.
- Existing performance audit notes that rule-heavy pattern matching and direct
  count reads are important to measure alongside traversal.

## Assumptions

- This planning task creates the goal file only; implementation happens in a
  later execution session using that goal.
- The goal should require current-product `assura check` benchmarks, not legacy
  `ConstraintEngine` benchmark paths.
- The goal should keep richer notation implementation mostly deferred until the
  LS-Lint parity and performance baseline is stronger, but it should require a
  current notation design consolidation so the next notation goal starts from
  source-truth docs.

## Requirements

- Add a goal file under `docs/goals/`.
- State that execution starts by creating a new branch from a clean,
  up-to-date `master`.
- State that execution ends with the branch pushed and a draft PR created.
- Require the final PR body and final handoff to include benchmark and
  performance results, artifact links, chart-ready data links, and website or
  preview visualization links.
- Define up front what is "good enough" for LS-Lint parity and performance
  comparisons.
- Include objective, repo truth, acceptance criteria, test matrix, benchmark
  matrix, notation requirements, known gaps, non-goals, validation commands,
  progress log, and final checklist.
- Require a pinned external-real-repo fixture manifest that materializes
  stable realistic test cases in CI/CD instead of checking full third-party
  repositories into source control.
- Require PR-visible Assura versus LS-Lint performance comparison artifacts.
- Require durable machine-readable performance history suitable for website
  charting.
- Make production `jwalk` migration a primary acceptance criterion.
- Require investigation of extension/suffix indexing, direct-child indexing,
  glob matching, and avoiding required-directory semantics for lint scopes.
- Require research into incremental/cache-aware checking using git-assisted
  change detection, file hashing, config invalidation, and cache placement that
  avoids git and Assura self-check noise.
- Require realistic LS-Lint fixture repositories with simple and comprehensive
  configurations.

## Acceptance Criteria

- [ ] `docs/goals/assura-ls-lint-realistic-parity-core-performance.md` exists.
- [ ] The goal starts with creating a branch from clean, up-to-date `master`.
- [ ] The goal ends with a pushed branch and draft PR URL.
- [ ] The goal requires PR benchmark/performance details and artifact links.
- [ ] The goal explicitly requires `jwalk` in the production `assura check`
      traversal path.
- [ ] The goal defines good-enough LS-Lint compatibility and performance
      criteria before optimization work begins.
- [ ] The goal requires a pinned external fixture harness for CI/CD.
- [ ] The goal requires PR-visible performance comparison evidence.
- [ ] The goal requires stored chart-ready performance history and a website
      link/page.
- [ ] The goal requires incremental/cache-aware checking research, including
      config invalidation and safe cache placement.
- [ ] The goal includes realistic LS-Lint parity fixture requirements.
- [ ] The goal includes current-product performance benchmark and profiling
      requirements.
- [ ] The goal includes notation design consolidation requirements.
- [ ] The goal gives future agents clear stop conditions and validation
      commands.

## Out of Scope

- Implementing the `jwalk` migration in this planning task.
- Adding LS-Lint fixtures or benchmark code in this planning task.
- Implementing richer notation extensions in this planning task.

## Technical Notes

- Existing goal format examples:
  `docs/goals/assura-v0.1-polished.md` and
  `docs/goals/assura-agent-nudge-mvp.md`.
- Current production traversal uses `walkdir::WalkDir` in `src/cli/check.rs`.
- `jwalk` is already a dependency in `Cargo.toml` and appears in
  `benches/profiling.rs`.
- Existing current-product LS-Lint benchmark is
  `benches/ls_lint_comparison.rs`.
- Existing LS-Lint parity regression coverage is
  `tests/ls_lint_parity_regression_tests.rs`.
