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
