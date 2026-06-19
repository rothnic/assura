---
title: Goal 13 Release And Performance Evidence Review
status: active
---

# Goal 13 Release And Performance Evidence Review

## Scope Review

Goal 13 extends the existing `cargo xtask target-state` release/performance
checks instead of adding a parallel verifier. The first implementation slice
turns release artifacts into an explicit matrix and requires current
performance artifacts to carry clean-source provenance.

## Initial Findings

| Finding | Evidence | Action |
| --- | --- | --- |
| Current performance artifacts were generated from a dirty worktree. | `benches/history/current.json` and `website/public/data/performance/current.json` record commit `56937c6f688dd4182b195363569b1a4dbeb8f815` with `source_worktree_dirty: true`. | `cargo xtask target-state` now rejects dirty-source performance artifacts; regenerate from a clean tree before final PR validation. |
| Release artifact checks were hard-coded and partial. | `xtask/src/main.rs` checked selected archive names but did not join release workflow, CI smoke labels, installer scripts, release checklist, and website release docs through one matrix. | Added a release artifact matrix for the five public archives and their installer/CI smoke expectations. |

## Refreshed Performance Evidence

Generated from a clean source tree after committing the stricter target-state
checks.

| Field | Value |
| --- | --- |
| Timestamp | `2026-06-19T03:21:52Z` |
| Source commit | `c515e189566c8b2b87cd772851c7d13e6940a5f4` |
| Branch | `codex/goal-13-release-performance-evidence` |
| Dirty source | `false` |
| Iterations | `5` |
| LS-Lint available | `true` |
| Cold claim verdict | `not-complete` |
| Warm claim verdict | `complete` |

The refreshed evidence preserves the existing product claim boundary: cold
`assura-check-cli` evidence is not complete, while warm editor-session evidence
remains complete. Public copy should continue to separate those claims.

## Cold Gate Follow-Up Acceptance

Goal 13 accepts the current cold `assura-cli` LS-Lint-equivalent 2x result as
`not-complete` for the checked macOS dynamic report. This PR must not claim a
complete cold 2x headline. The accepted follow-up is bounded to a future
release-artifact performance task that reruns the clean Linux static-CRT
release profile, compares it with the current dynamic report, and reopens only
optimizations that either remove a startup-floor miss or reduce one of the
plain implementation misses by at least 10% on the same machine/profile.

Ranked cold misses from the refreshed `realistic-equivalent` cohort:

| Rank | Fixture | Status | Ratio To 2x Target | Follow-Up Boundary |
| --- | --- | --- | --- | --- |
| 1 | `rule_heavy_repo` | `blocked-by-rust-cli-floor` | `2.59` | Recheck on Linux static-CRT release artifact before optimizing validation. |
| 2 | `monorepo_packages` | `blocked-by-rust-cli-floor` | `2.15` | Recheck on Linux static-CRT release artifact before optimizing validation. |
| 3 | `web_app` | `blocked-by-rust-cli-floor` | `2.15` | Recheck on Linux static-CRT release artifact before optimizing validation. |
| 4 | `simple_library` | `blocked-by-rust-cli-floor` | `2.04` | Recheck on Linux static-CRT release artifact before optimizing validation. |
| 5 | `multipart_extension_regression` | `misses-target` | `1.42` | Profile shorthand/multipart extension normalization only with before/after evidence. |
| 6 | `monorepo_policy` | `misses-target` | `1.40` | Profile scope/rule matching only with before/after evidence. |
| 7 | `many_configured_scopes_regression` | `misses-target` | `1.32` | Profile configured-scope traversal only with before/after evidence. |

The warm editor-session gate remains complete across all eight
`realistic-equivalent` rows, so public docs may describe warm/editor-session
speed separately from the cold one-process claim.

## Validation Plan

```bash
cargo fmt --all -- --check
cargo xtask target-state
cargo build --release -p assura --bins
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
cargo xtask docs
cargo run --quiet -- check --format json .
git diff --check
```

## Validation Results

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo xtask target-state` | Passed after refreshing performance data |
| `cargo xtask docs` | Passed with known Astro sitemap `site` warning |
| `cargo run --quiet -- check --format json .` | Passed, 741 files and 171 dirs checked, 0 violations |
| `git diff --check` | Passed |

## Independent Review Response

The review agent blocked the PR-ready handoff until public performance copy
matched the checked artifact and the cold 2x miss had an accepted bounded
follow-up. The response is to align website copy with the current
`realistic-equivalent` report command and to make `cargo xtask target-state`
fail if public copy drifts from the checked cohort/command or if a non-complete
cold verdict lacks this follow-up record.
