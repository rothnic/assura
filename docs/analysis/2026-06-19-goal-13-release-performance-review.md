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
