---
id: goal-assura-ls-lint-no-slower-performance-gate
type: goal
title: Assura LS-Lint no-slower performance gate
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-cli-to-cli-ls-lint-performance-verification.md
  - ./assura-real-repo-ls-lint-performance-evidence.md
  - ./assura-ls-lint-realistic-parity-core-performance.md
  - ../analysis/2026-05-18-native-ls-lint-performance-gap-review.md
  - ../../benches/history/current.json
  - ../../website/public/data/performance/current.json
---

# Assura LS-Lint No-Slower Performance Gate

## Objective

Make native LS-Lint comparison a hard beta merge gate: for every
LS-Lint-equivalent headline fixture, Assura must not be slower than native
LS-Lint. If any fixture is slower, the merge is blocked until the regression is
attributed, fixed, or the fixture is explicitly removed from the headline set
with reviewer approval.

## Current Gap

The checked `benches/history/current.json` currently reports cold
`assura-cli` faster on 7 of 8 realistic-equivalent fixtures, with
`two_x_claim_verdict="not-complete"`. Warm session evidence is complete, but
warm rows cannot substitute for the cold one-shot CLI gate. The old "CLI floor"
explanation is not acceptable as a reason to merge slower fixture behavior
without a concrete attribution and product decision.

Rust should be capable of matching or beating native LS-Lint for the same
LS-Lint-equivalent workload. If it does not, the goal is to find the actual
cause: binary payload, startup, config discovery, YAML parsing, ignore pruning,
traversal, rule planning, path allocation, sorting, process model, fixture
shape, or an invalid comparison setup.

## Scope

- Re-run current LS-Lint comparison evidence from release binaries.
- Identify every headline fixture where Assura is slower than native LS-Lint.
- Attribute slower rows by phase and by binary payload.
- Separate cold CLI, warm daemon/session, in-process, traversal-only, and
  status-file evidence.
- Add a CI or review gate that fails when headline LS-Lint-equivalent fixtures
  regress to slower-than-LS-Lint.
- Keep generated, adversarial, and diagnostic fixtures out of the headline
  gate unless they are explicitly promoted.
- Investigate without making the CLI cumbersome: no required daemon for the
  one-shot CLI gate, no hidden cache dependency, and no confusing user flags for
  normal checks.

## Non-Goals

- No broad rewrite before attribution.
- No using warm daemon evidence to pass the cold CLI gate.
- No hiding slower rows behind aggregate speedups.
- No claiming 2x faster unless every relevant fixture meets the stricter 2x
  gate.

## Definition Of Done

- The checked performance report is covered by a machine-readable no-slower
  gate command for headline LS-Lint-equivalent fixtures.
- CI or release checks fail when any headline fixture is slower than native
  LS-Lint on the accepted cold CLI row family.
- Current slower rows are fixed or removed from the headline set with written
  reviewer approval and a replacement fixture rationale.
- Phase diagnostics explain the cause of any remaining misses before the fix.
- Website language cannot claim faster-than-LS-Lint unless the current checked
  data proves it.
- Warm daemon/session performance remains separately reported and cannot mask a
  cold CLI regression.

## Validation Commands

```bash
cargo fmt --check
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
jq -r '.claim_summary' benches/history/current.json
jq -r '.results[] | select(.row_family=="assura-cli" or .row_family=="ls-lint-cli") | [.fixture_id,.row_family,.median_runtime_ms,.status] | @tsv' benches/history/current.json
cargo xtask performance-no-slower
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

`cargo xtask performance-no-slower` is intentionally expected to fail until
every headline realistic-equivalent fixture has an `assura-cli` median less
than or equal to its paired native `ls-lint-cli` median. Use
`cargo xtask performance-no-slower <report.json> --cohort <name> --assura-row
<row> --ls-lint-row <row>` for target artifacts during diagnosis.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Added the concrete `cargo xtask performance-no-slower` gate command contract. The command reads an existing performance report and exits nonzero when any headline Assura row is slower than its paired native LS-Lint row or when a pair is missing, so the gate is cheap and does not regenerate benchmarks during ordinary validation. | `xtask/src/main.rs`; `cargo test -p xtask performance_no_slower`; expected current-data failure: `cargo xtask performance-no-slower`. |
| 2026-06-30 | Attributed the checked `simple_library` miss to stale performance build instructions that measured a default-feature primary launcher instead of the release bundle's lightweight `assura` launcher. A release-style target report passed the no-slower gate with `simple_library` at 3.705 ms for `assura-cli` versus 4.703 ms for native LS-Lint. | `cargo build --release --bin assura --no-default-features --features json-output,yaml-config`; `cargo build --release --bin assura-full`; `cargo build --release -p assura-check-cli`; `target/release/assura performance-report --output target/performance/no-slower-light-launcher.json --history target/performance/no-slower-light-launcher.jsonl --website-dir target/performance/no-slower-light-launcher-website --iterations 3`; `cargo xtask performance-no-slower target/performance/no-slower-light-launcher.json`. |
| 2026-06-30 | Fixed the remaining `rule_heavy_repo` miss by adding an exact LS-Lint extension-segment lookup for non-wildcard patterns such as `*.kind-17.ts`, while retaining the existing wildcard/glob fallback. The checked 5-iteration report now has Assura faster than native LS-Lint on all 8 realistic-equivalent fixtures; the stricter 2x claim remains incomplete and separate from the no-slower beta gate. | `src/cli/check/ls_fast_naming.rs`; `src/cli/check/ls_fast_plan_tests.rs`; `target/release/assura performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 5`; `cargo xtask performance-no-slower`; `jq '.claim_summary,.warm_claim_summary' benches/history/current.json`. |

## Review Tasks

- R1: Confirm the gate compares native LS-Lint to the accepted cold Assura CLI
  row family on equivalent fixtures.
- R2: Confirm warm/session rows are reported separately.
- R3: Confirm every slower fixture has phase attribution before a fix is
  accepted.
- R4: Confirm the CLI remains simple for normal users.

## Reviewer Blocking Criteria

Block if any headline fixture is slower than native LS-Lint, if aggregate
speedups hide a slower row, if process-floor language substitutes for
attribution, if warm daemon rows are used to pass the cold CLI gate, or if the
solution requires cumbersome flags for normal structure checks.
