---
title: Performance Hotspot Optimization Progress
date: 2026-05-15
status: current
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/analysis/2026-05-11-ls-lint-parity-performance-regression-audit.md
  - docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md
---

# Performance Hotspot Optimization Progress

## Scope

This pass investigated why the `jwalk` traversal benchmark can be slower than
the prior `walkdir` traversal benchmark, then targeted the highest-cost current
`assura check` hotspots before larger cache or notation work.

## Findings

The traversal-only comparison is not the dominant current-product hotspot.
Production `assura check` already uses `jwalk` in serial mode so it can prune
ignored directories through `process_read_dir` while preserving current output
semantics. In pure traversal rows, serial `jwalk` can trail `walkdir` because
the checked path pays the `jwalk` callback machinery without enough parallel
work to amortize it.

Switching production traversal to `RayonDefaultPool` improved some large and
wide synthetic paths, but it regressed small, deep, pruned, and direct-count
checks. The pruned ignored-generated fixture regressed from roughly 42 us to
roughly 654 us in local profiling, so parallel `jwalk` is not a safe default
for the current product path.

The largest observed hotspot was rule-heavy filename validation. The
`many_wildcard_extension_path_rules` profile repeatedly scanned every
`files.naming_patterns` entry for every file and cloned large cached rule
bundles when resolving effective rules.

## Implemented Optimizations

- Added a simple suffix fast path for compiled single-pattern checks such as
  `*.ts`.
- Added a direct LS-Lint suffix lookup for naming-pattern maps that contain
  only simple dot-suffix patterns, while falling back to the existing glob scan
  for mixed or complex pattern maps.
- Changed cached `EffectiveRules` bundles to share file, directory, and
  markdown rule bundles with `Arc`, avoiding repeated clones of large
  `FileBundle` maps during per-file validation.

These changes preserve the existing most-specific pattern precedence rule and
defer to the previous glob path whenever the direct suffix shortcut cannot
prove that the full map is simple.

## Local Profile Results

Command:

```bash
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo bench --bench profiling structure_check -- --noplot
```

| Profile | Before | After | Result |
| --- | ---: | ---: | --- |
| `small` | 699.57 us | 686.55 us | no regression |
| `medium` | 9.0665 ms | 8.9971 ms | no regression |
| `large` | 35.256 ms | 34.298 ms | improved |
| `deep_tree` | 1.8992 ms | 1.8589 ms | no regression |
| `wide_tree` | 9.5592 ms | 9.2963 ms | improved |
| `many_ignored_generated_dirs` | 45.780 us | 42.496 us | improved |
| `many_direct_content_checks` | 6.5926 ms | 6.4463 ms | no regression |
| `many_wildcard_extension_path_rules` | 294.22 ms | 47.702 ms | 6.2x faster |

Attribution groups remained stable: config loading stayed near 14 us, pure
large traversal stayed near 4-5 ms, pruned traversal stayed near 13 us, and
direct count reads stayed near 1.5 ms. That leaves rule-heavy pattern matching
and effective-rule cloning as the highest-value optimization area addressed by
this pass.

## Regenerated Comparison Data

Command:

```bash
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 3
```

The network-enabled report resolved `@ls-lint/ls-lint@2.3.0` successfully and
recorded live LS-Lint rows:

| Fixture | Assura median | LS-Lint median |
| --- | ---: | ---: |
| `simple_small` | 1.179747 ms | 563.096881 ms |
| `simple_medium` | 7.682870 ms | 481.220390 ms |
| `monorepo_large` | 39.886655 ms | 492.521547 ms |
| `rule_heavy` | 77.984530 ms | 502.242832 ms |
| `ignored_generated_heavy` | 0.202543 ms | 502.328341 ms |

The same data is tracked in `benches/history/current.json` and
`website/public/data/performance/current.json`.

## Deferred Work

- Direct-child indexes for `exists` checks remain a later optimization. Current
  direct-count attribution is visible but not the largest cost.
- Incremental cache-aware checking remains design-only because it affects
  invalidation, CI behavior, and configuration fingerprinting.
- Parallel `jwalk` may be worth revisiting behind a workload-aware heuristic,
  but the current evidence supports serial `jwalk` as the default.
