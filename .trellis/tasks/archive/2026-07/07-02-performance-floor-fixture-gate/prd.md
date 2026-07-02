---
title: Performance floor fixture gate
status: active
priority: P0
---

# Performance Floor Fixture Gate

## Goal

Execute `docs/goals/assura-performance-floor-and-fixture-gate.md` as the next
child of the post-beta capabilities program.

This slice makes the no-slower performance gate operate on explicit accepted
LS-Lint-equivalent fixture rows instead of only headline aggregate or cohort
summary claims. A merge must fail when any accepted fixture has a passing
Assura row that is slower than the matching native LS-Lint row, or when an
accepted fixture is missing a required row.

## User Story Fit

In the parent verification story, a maintainer must be able to trust CI to
reject a branch when Assura is slower than LS-Lint on any accepted structure
fixture. This task enables that decision by making fixture acceptance
machine-readable in the performance report and by making the gate enforce every
accepted LS-Lint-equivalent fixture.

## Scope

- Add explicit performance fixture acceptance metadata to report rows.
- Treat native LS-Lint parity, non-diagnostic headline rows in the configured
  cohort as accepted unless a fixture is intentionally classified otherwise.
- Update `cargo xtask performance-no-slower` so the default gate checks
  accepted fixtures, not every incidental row with the same cohort.
- Keep diagnostic, experimental, synthetic, and Assura-native-only rows out of
  the accepted no-slower gate unless explicitly promoted later.
- Update target-state/docs/tests so public performance evidence cannot drift
  back to aggregate-only claims.
- Keep tracked benchmark data updates separate from implementation unless
  regenerated with release binaries and recorded command evidence.

## Acceptance Criteria

- [ ] Performance report rows include stable fixture acceptance metadata.
- [ ] `cargo xtask performance-no-slower` fails for a slower accepted fixture.
- [ ] The gate ignores diagnostic/non-accepted fixture rows unless explicitly
      configured to check them.
- [ ] Current checked performance JSON passes the accepted fixture gate.
- [ ] Target-state checks require accepted fixture metadata and the no-slower
      gate over checked performance data.
- [ ] Parent goal and roadmap progress log route from this child to the next
      post-beta goal after merge.
- [ ] Independent review confirms aggregate speed cannot hide a slower accepted
      fixture.

## Validation

```bash
cargo fmt --check
cargo test -p xtask --quiet
cargo test --test performance_report_contract_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask performance-no-slower
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
python3 ./.trellis/scripts/task.py validate 07-02-performance-floor-fixture-gate
git diff --check
```

Release evidence, when benchmark data changes:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 5
cargo xtask performance-no-slower
```

## Reviewer Blocking Criteria

Block if the gate can pass while an accepted LS-Lint-equivalent fixture is
slower, if accepted fixture classification is implicit or undocumented, if
diagnostic rows become public proof, or if checked benchmark data changes
without release-binary regeneration evidence.
