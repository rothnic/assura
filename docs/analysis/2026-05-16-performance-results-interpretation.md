---
title: Performance Results Interpretation
date: 2026-05-16
status: current
---

# Performance Results Interpretation

## 2026-05-17 Update

This note is superseded for public Assura-versus-LS-Lint claims by the
CLI-to-CLI report contract in
`docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md`.

Rows named `assura` in this document are now treated as `assura-in-process`
diagnostics. Public website claims must use `assura-cli` and `ls-lint-cli`
rows from the machine-readable report.

This note explains what the PR #11 performance rows prove, what they do not
prove, and which decision should follow from them.

## Core Distinction

The traversal-only rows are not production `assura check` timings.

- `walkdir`, `jwalk-serial`, and `jwalk-parallel` count filesystem entries.
- They do not load Assura config, resolve effective directory rules, validate
  naming patterns, read file contents, apply markdown checks, or sort
  violations.
- They are useful for understanding raw traversal overhead, but they cannot by
  themselves justify the production checker default.

The top-level `assura` row is the production comparison row. It includes config
discovery/load, checker initialization, configured structure checks,
walk-and-validate, and report sorting.

## Current Decision

The final PR keeps deterministic serial `jwalk` as the production default and
keeps a real parallel `jwalk` path available through
`ASSURA_CHECK_TRAVERSAL=parallel-jwalk`.

This is a conservative decision, not proof that serial `jwalk` is globally
faster than every alternative. The evidence supports these narrower claims:

1. The final implementation improves the tracked bottleneck rows against the
   PR baseline measured on the same machine.
2. Raw parallel `jwalk` is often faster than raw walkdir and raw serial
   `jwalk` on traversal-heavy fixtures.
3. Full Assura time is dominated by validation work on rule-heavy fixtures, so
   saving a few milliseconds of raw traversal does not automatically improve
   end-to-end `assura` rows.
4. The current evidence does not prove that a full walkdir-based checker would
   be slower than the serial `jwalk` checker.

The next performance decision should not use traversal-only rows as a proxy for
the production default. It should add full-check strategy rows that compare
walkdir, serial `jwalk`, and parallel `jwalk` under the same validation logic.

## Fixture Meanings

`rule_heavy` and `rule_heavy_repo` are intentionally different.

| Fixture | Shape | Purpose |
| --- | --- | --- |
| `rule_heavy` | Synthetic large fixture: 40 directories, 50 files per directory, 30 extension-specific naming patterns. | Stress worst-case multi-extension rule matching at scale. This is a bottleneck fixture, not a realistic repo shape. |
| `rule_heavy_repo` | Smaller realistic fixture: 8 feature directories, 24 files per directory, 36 extension-specific rules plus a wildcard extension rule. | Test the same rule class in a repo-shaped workload where startup/config overhead and directory shape are more realistic. |

When `rule_heavy` is slower than LS-Lint but `rule_heavy_repo` is faster, that
means Assura still has a synthetic scale bottleneck even though it is
competitive on a more realistic rule-heavy repo shape.

## LS-Lint Timing Interpretation

The final report no longer measures repeated npm package resolution in the
timed loop. It installs/resolves `@ls-lint/ls-lint@2.3.0` once, then executes
the cached LS-Lint binary for each measured sample.

The LS-Lint rows still include process startup, LS-Lint config load, traversal,
and validation for each iteration. They are warm tool execution timings, not
pure in-process validation timings.

The final local rows are plausible for this measurement design:

| Fixture | Assura ms | LS-Lint ms | Speedup | Interpretation |
| --- | ---: | ---: | ---: | --- |
| `simple_library` | 0.843 | 104.185 | 123.5x | Assura is much faster on a realistic small library shape. |
| `web_app` | 0.801 | 101.060 | 126.2x | Assura is much faster on a realistic frontend shape. |
| `monorepo_packages` | 1.669 | 104.973 | 62.9x | Assura is much faster on a realistic package-monorepo shape. |
| `rule_heavy_repo` | 26.145 | 99.177 | 3.8x | Assura is faster on the smaller realistic multi-extension repo shape. |
| `ignored_generated_heavy_repo` | 0.542 | 108.457 | 200.3x | Assura pruning is very effective on generated/ignored-heavy repo-shaped input. |
| `rule_heavy` | 173.930 | 117.829 | 0.7x | LS-Lint is faster on the synthetic multi-extension stress case; Assura still has rule-heavy scale work to do. |
| `ignored_generated_heavy` | 0.509 | 102.912 | 202.2x | Assura pruning is very effective on generated/ignored-heavy synthetic input. |

The LS-Lint rows should not be read as proof that LS-Lint is always about
100 ms. They show warm binary invocation cost plus fixture-specific work on
this macOS x86_64 machine.

## Website Evidence

The website evidence is generated from tracked public data:

- Page source: `website/src/content/docs/reference/performance.mdx`
- Built page: `website/dist/reference/performance/index.html`
- Current data: `website/dist/data/performance/current.json`
- History data: `website/dist/data/performance/ls-lint-comparison-history.jsonl`

The local build passed with `cd website && pnpm build`. There is not currently
a deployed website URL recorded in the PR; reviewers can inspect the built
page locally with `pnpm preview` from `website/`.

## Follow-Up Requirement

Before changing the production traversal default again, add a performance
report row family that measures full `assura check` using each candidate
strategy:

- full check with walkdir traversal and pruning,
- full check with serial `jwalk`,
- full check with parallel `jwalk` collection plus deterministic validation.

Only those full-check rows should drive the default traversal strategy.
