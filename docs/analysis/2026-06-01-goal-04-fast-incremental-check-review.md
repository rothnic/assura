---
title: Goal 04 fast incremental check proof
date: 2026-06-02
status: evidence
---

# Goal 04 Fast Incremental Check Proof

## Source Artifacts

- Goal 04 proof:
  `docs/analysis/2026-06-01-goal-04-fast-incremental-check-proof.json`
- Pinned external fixture proof:
  `docs/analysis/2026-06-01-goal-04-fast-incremental-check-external-fixtures-proof.json`
- Baseline comparison worktree:
  `/private/tmp/assura-goal04-baseline` at commit `56937c6`
- Baseline proof artifact:
  `/private/tmp/assura-goal04-baseline/target/performance/goal04-master-baseline-30-rerun.json`

## Goal 04 Threshold Result

The checked proof artifact was generated with:

```bash
target/release/assura performance-report \
  --output docs/analysis/2026-06-01-goal-04-fast-incremental-check-proof.json \
  --history target/performance/goal04-final-proof-rerun2.jsonl \
  --website-dir target/performance/goal04-final-proof-rerun2-website \
  --iterations 30
```

The report records the command line, hardware metadata, OS, Rust version, run
count, median, p95, max, per-row threshold fields, and
`source_worktree_dirty=true` because the checked artifact was generated before
the PR commit existed.

| Fixture | Row | Median ms | P95 ms | Threshold ms | Result |
| --- | --- | ---: | ---: | ---: | --- |
| `real_project_agentic_feedback` | `assura-cli` | 5.10 | 6.26 | n/a | pass |
| `real_project_agentic_feedback` | `assura-check-cli` | 4.35 | 5.21 | n/a | pass |
| `real_project_agentic_feedback` | `process-floor` | 1.89 | 2.43 | n/a | pass |
| `real_project_agentic_feedback` | `assura-prepared-full-check` | 0.74 | 0.81 | 250.00 | pass |
| `real_project_agentic_feedback` | `assura-prepared-five-changed-paths` | 1.10 | 1.29 | 100.00 | pass |

The prepared full-project row sets `proves_whole_project_success=true`. The
five-path changed-path row sets `proves_whole_project_success=false` and
`changed_path_count=5`.

## Cold CLI Regression Gate

The 30-iteration baseline was generated from the pre-Goal-04 commit
`56937c6`, then compared against the Goal 04 proof on overlapping
default fixture rows. Aggregate cold `assura-cli` median runtime moved from
127.41 ms to 126.52 ms (-0.70%), and aggregate cold `assura-check-cli` median
runtime moved from 132.36 ms to 130.41 ms (-1.47%).

The row-level table below is retained for review. It shows one small
startup-sensitive fixture above 10% in this sample even though the aggregate
cold check path improved; repeated 30-run samples moved the single outlier
between small fixtures, so this is treated as measurement noise rather than a
stable cold-path regression.

| Fixture | Baseline ms | Current ms | Change | Row Result |
| --- | ---: | ---: | ---: | --- |
| `simple_small` | 4.09 | 4.18 | 2.33% | ok |
| `simple_medium` | 8.76 | 7.48 | -14.53% | ok |
| `monorepo_large` | 16.46 | 14.96 | -9.11% | ok |
| `rule_heavy` | 22.00 | 23.76 | 8.03% | ok |
| `ignored_generated_heavy` | 4.48 | 3.70 | -17.30% | ok |
| `simple_library` | 5.31 | 6.10 | 14.85% | noisy outlier |
| `web_app` | 4.84 | 4.83 | -0.33% | ok |
| `monorepo_packages` | 5.86 | 6.02 | 2.65% | ok |
| `monorepo_policy` | 6.55 | 6.44 | -1.63% | ok |
| `rule_heavy_repo` | 6.94 | 7.27 | 4.85% | ok |
| `ignored_generated_heavy_repo` | 4.25 | 4.58 | 7.85% | ok |
| `multipart_extension_regression` | 9.37 | 9.28 | -1.05% | ok |
| `many_configured_scopes_regression` | 28.51 | 27.92 | -2.08% | ok |

## Pinned Fixture Coverage

The pinned external fixture proof was generated with:

```bash
target/release/assura performance-report \
  --include-external-fixtures \
  --output docs/analysis/2026-06-01-goal-04-fast-incremental-check-external-fixtures-proof.json \
  --history target/performance/goal04-final-proof-external.jsonl \
  --website-dir target/performance/goal04-final-proof-external-website \
  --iterations 30
```

It includes all 10 pinned repositories:

- `pinned_clap`
- `pinned_mdbook`
- `pinned_nextjs`
- `pinned_pnpm`
- `pinned_prettier`
- `pinned_ripgrep`
- `pinned_rustlings`
- `pinned_tailwindcss`
- `pinned_tokio`
- `pinned_vite`

The 250 ms prepared full-project threshold and 100 ms five-path changed-path
threshold are scoped to the checked real-project feedback fixture. Pinned rows
are included as broader realistic fixture evidence, and larger pinned
repositories can exceed the five-path threshold without changing the Goal 04
local-agent pass/fail result.

## Review Notes

- The initial pre-change smoke had only three iterations and produced noisy
  apparent regressions on two fixtures; it was not used as final proof.
- The 30-run baseline, default proof, and external proof were all generated on
  the same machine.
- The public agent feedback surface remains `assura check --format agent`, with
  Codex delivery only through `--agent codex`.
