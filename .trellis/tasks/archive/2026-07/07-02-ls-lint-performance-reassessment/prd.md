# Reassess LS-Lint Performance

## Objective

Close `docs/goals/assura-ls-lint-performance-reassessment.md` by proving the
current beta increment's accepted LS-Lint-equivalent performance rows are
row-by-row no slower than native LS-Lint, and by explaining the remaining CLI
floor/2x misses with phase-level evidence.

## User Outcome

An Assura maintainer should be able to read one current report and answer:

- which accepted fixture rows are compared against native LS-Lint;
- whether any accepted cold `assura-cli` row is slower than native LS-Lint;
- how much of each row is process startup, Rust CLI floor, config loading,
  checker initialization, traversal/validation, and report sorting;
- which rows are only diagnostic or warm-session evidence;
- what CI command fails a merge if an accepted LS-Lint-equivalent row regresses.

## Scope

- Revalidate the goal against current `origin/master`, PR #134, roadmap state,
  checked performance data, CI gates, and self-check output.
- Generate fresh release-artifact performance evidence from the current branch
  using the performance-reporting build sequence.
- Add or update analysis/docs/target-state checks if the current evidence does
  not explain phase-level attribution clearly enough.
- Keep cold CLI no-slower proof separate from warm daemon/session proof.
- Close the child goal only after local validation and independent review.

## Non-Goals

- Do not reclassify accepted fixture rows to make the gate pass.
- Do not use warm daemon/session rows as proof for cold CLI rows.
- Do not make normal `assura check` usage more cumbersome for benchmark wins.
- Do not claim the parent post-beta program is complete.

## Validation

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report --output target/performance/ls-lint-reassessment.json --history target/performance/ls-lint-reassessment.jsonl --website-dir target/performance/website-data --iterations 3
cargo xtask performance-no-slower target/performance/ls-lint-reassessment.json
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

Tracked data updates should use 5 iterations if the report shape and local
runtime are stable enough for merge evidence.

## Reviewer Blocking Criteria

Block if any accepted LS-Lint-equivalent row is slower than native LS-Lint, if
phase-level attribution is missing or misleading, if aggregate speedups hide a
row failure, if warm measurements are used as cold proof, or if fixture
acceptance is changed without explicit reviewer-approved evidence.
