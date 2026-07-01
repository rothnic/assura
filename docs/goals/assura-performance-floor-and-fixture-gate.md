---
id: goal-assura-performance-floor-and-fixture-gate
type: goal
title: Assura performance floor and fixture gate
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-ls-lint-no-slower-performance-gate.md
  - ./assura-cli-to-cli-ls-lint-performance-verification.md
---

# Assura Performance Floor And Fixture Gate

## Objective

Make Assura's performance policy match the product bar: every accepted
LS-Lint-equivalent fixture row must be no slower than native LS-Lint, and any
Rust-vs-Go CLI floor miss must be measured, explained, and fixed or removed
from the accepted set with explicit review.

## Current Gap

`v0.2.0` enforces the no-slower gate for headline realistic-equivalent rows.
The post-beta bar is stricter: no accepted fixture test case should silently be
slower, and "CLI floor" should drive remediation rather than excuse a miss.

## Scope

- Audit all performance fixture cohorts and classify accepted, diagnostic,
  experimental, and retired rows.
- Expand `cargo xtask performance-no-slower` so it can fail on every accepted
  LS-Lint-equivalent fixture, not only headline aggregate/cohort summaries.
- Preserve a fast CI path for existing report JSON and a separate regeneration
  path for checked benchmark updates.
- Add CLI floor attribution that separates process startup, config load,
  traversal, rule planning, validation, sorting, and output.
- Investigate release profile, binary size, feature flags, static/dynamic
  linking, cache probes, and minimal hot-path binaries when Rust rows are
  slower than Go rows.
- Update docs and website performance data only from reproducible checked
  evidence.

## Non-Goals

- No aggregate-only pass criteria.
- No fixture removal without written rationale and independent review.
- No benchmark system parallel to the existing `benches/` and `xtask` evidence
  flow.

## Definition Of Done

- CI fails when any accepted LS-Lint-equivalent fixture row is slower than
  native LS-Lint.
- Checked performance data records accepted/diagnostic fixture classification.
- CLI floor attribution identifies the dominant cost for any current miss.
- Any slower accepted row is fixed before merge or reclassified with reviewer
  approval and replacement coverage.
- Independent review confirms the gate cannot pass on aggregate speed while
  hiding a slower accepted fixture.

## Validation Commands

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release -p assura-check-cli
target/release/assura performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 5
cargo xtask performance-no-slower
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if any accepted LS-Lint-equivalent fixture is slower than LS-Lint, if the
gate can pass by aggregate-only math, if fixture removal lacks rationale, or if
new performance claims are not backed by checked report JSON.
