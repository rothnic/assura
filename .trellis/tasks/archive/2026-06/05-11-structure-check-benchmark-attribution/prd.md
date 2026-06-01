# Structure Check Benchmark Attribution

## Goal

Reuse the existing Criterion benchmark infrastructure to measure the current
structure-first `assura check` path, attribute the PR #7 rule-heavy slowdown,
and correct stale LS-Lint `exists` wording without creating a parallel
performance system.

## What I Already Know

- The repo already has Criterion benchmark suites in `benches/profiling.rs`,
  `benches/ls_lint_comparison.rs`, and `benches/ls_lint_benchmarks.rs`.
- Existing `jwalk` usage is in the benchmark and older `ConstraintEngine`
  comparison path, not in the current structure-first check path.
- Current `run_structure_check` uses `walkdir::WalkDir` in `src/cli/check.rs`.
- PR #7's slow rule-heavy fixture measured `run_structure_check`, so it did
  not prove the older `ConstraintEngine + jwalk` path is slow.
- LS-Lint 2.3 docs describe `exists` for extensions and `.dir` directory
  rules. A live `README.md: exists:1` fixture reports `found 0` even when
  `README.md` exists, so exact filename `exists` should be documented as an
  Assura compatibility extension unless another official exact syntax is
  found.

## Requirements

- Start from latest `master` on branch
  `codex/structure-check-benchmark-attribution`.
- Keep exact filename `exists` support in Assura, but correct docs/specs so
  extension and `.dir` `exists` are LS-Lint parity and exact filename `exists`
  is an Assura compatibility extension.
- Add or confirm positive and negative regression coverage for exact filename
  `exists`, including a missing-file case that reports `exists_count` instead
  of `required_directory`.
- Extend the existing Criterion setup, preferably `benches/profiling.rs`, with
  structure-first benchmark groups for:
  small, medium, large, deep tree, wide tree, ignored/generated directories,
  many direct-content checks, and many wildcard/extension/path/naming-pattern
  rules.
- Attribute cost enough to distinguish full `run_structure_check`, traversal,
  config load, exclusion pruning, directory count reads, and glob/pattern
  matching.
- Preserve `jwalk` benchmarking context and explicitly document whether
  current `assura check` uses it.
- Only implement a narrow optimization if the benchmark data identifies a
  low-risk hotspot. The acceptable first optimization is private
  precompiled/reused structure-first glob matching.

## Acceptance Criteria

- [ ] Docs/specs no longer claim exact filename `exists` is native LS-Lint
      parity unless a verified official LS-Lint 2.3 syntax proves otherwise.
- [ ] Criterion includes stable `structure_check/...` benchmark names and runs
      through `cargo bench --bench profiling structure_check`.
- [ ] The analysis report records source-of-truth `exists` findings, measured
      structure-first benchmark results, hotspot attribution, and whether
      `jwalk` matters to the current check path.
- [ ] Exact filename `exists` positive and negative regression tests pass.
- [ ] Final gates pass:
      `cargo fmt --all -- --check`,
      `cargo clippy --all-targets --all-features -- -D warnings`,
      `cargo test --all-targets --quiet`, and `cargo run -- check .`.

## Out of Scope

- No new performance framework outside existing `benches/`.
- No broad runtime rewrite.
- No public CLI, config schema, or exported Rust API changes.
- No removal of `jwalk` or exact filename `exists` support.
- No Windows CI, hook policy, or Codex runtime hook work.

## Research References

- `research/ls-lint-exists-source-truth.md` - LS-Lint 2.3 docs/source and live
  fixture results for `exists`.

## Technical Notes

- `benches/profiling.rs` already compares `jwalk` and sequential `walkdir`
  traversal and profiles older constraint-engine costs.
- `benches/ls_lint_comparison.rs` uses `jwalk` around `ConstraintEngine` for
  Assura-vs-LS-Lint comparison.
- `tests/ls_lint_parity_regression_tests.rs` has an ignored manual
  performance fixture that should be replaced or superseded by Criterion
  benchmark coverage, not expanded as the main measurement path.
