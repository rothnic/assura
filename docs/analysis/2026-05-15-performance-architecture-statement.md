---
title: Performance Architecture Statement
date: 2026-05-15
status: current
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/analysis/2026-05-15-performance-hotspot-optimization-progress.md
  - benches/history/current.json
  - benches/history/ls-lint-comparison.schema.json
---

# Performance Architecture Statement

## What "Direct LS-Lint Suffix Lookup" Means

LS-Lint extension rules are commonly suffix rules: `.ts`, `.test.ts`,
`.module.css`, and similar. Assura stores the equivalent as
`files.naming_patterns` entries like `*.ts`, `*.test.ts`, and `*.module.css`.

Before the optimization, each file name was compared against every configured
pattern through the glob matcher, then the most specific matching pattern was
chosen. That is correct, but expensive for rule-heavy projects.

The direct lookup optimization is only used when every
`files.naming_patterns` key is a simple dot-suffix pattern. For a file such as
`button.test.ts`, Assura builds the possible LS-Lint suffix patterns
`*.test.ts` and `*.ts`, checks the map directly, and picks the longest match.
If any pattern is more complex, Assura falls back to the previous full glob
scan. The optimization is therefore a fast path for the common LS-Lint shape,
not a new notation rule.

## Instrumentation Contract

`assura performance-report` now emits three levels of performance evidence:

1. End-to-end tool comparison rows: `assura` and `ls-lint`.
2. Traversal-only rows: `walkdir`, `jwalk-serial`, and `jwalk-parallel`.
3. Assura phase rows:
   - `assura:config-discovery`
   - `assura:config-load`
   - `assura:checker-init`
   - `assura:configured-structure`
   - `assura:walk-and-validate`
   - `assura:report-sort`

The phase rows are emitted per fixture with the same schema metadata as the
top-level rows: commit, branch, OS, architecture, Rust, Node/npm, fixture id,
source revision, cohort, rule cohort, median runtime, samples, status, and
baseline id. CI writes these phase rows into the uploaded performance artifact
and prints an Assura phase-breakdown table in the GitHub Actions summary.

## Representative Corpus

The performance report includes both size/stress scenarios and realistic
project-shape scenarios:

| Fixture | Purpose | Primary question |
| --- | --- | --- |
| `simple_small` | Small extension and directory naming tree | Baseline CLI overhead |
| `simple_medium` | Common source/test sized tree | Scaling with ordinary file count |
| `monorepo_large` | Larger package-like tree | Large traversal and validation cost |
| `rule_heavy` | Many extension patterns | Pattern matching cost |
| `ignored_generated_heavy` | Large ignored/generated tree | Exclusion pruning cost |
| `simple_library` | Source, docs, tests, generated output | Realistic library config shape |
| `web_app` | Components, tests, assets, build output | Frontend-style naming patterns |
| `monorepo_packages` | Explicit package-local source/test/docs | Nested config and inheritance |
| `rule_heavy_repo` | Realistic multi-extension tree | LS-Lint suffix rule pressure |
| `ignored_generated_heavy_repo` | Generated and coverage trees | Pruning before validation |

The realistic fixtures use LS-Lint-style configs converted into Assura configs
so parity tests, benchmarks, and CI performance reporting exercise the same
configuration surface.

## Hotspot Matrix

| Area | Evidence | Implemented optimization | Current stance | Future research |
| --- | --- | --- | --- | --- |
| Rule-heavy naming patterns | `many_wildcard_extension_path_rules` dropped from 294.22 ms to 47.702 ms after suffix and cache work. | Direct simple-suffix lookup; simple suffix match fast path. | Highest-value product hotspot addressed for current LS-Lint shape. | Pre-index complex globs by literal suffix/prefix where semantics allow it. |
| Effective rule resolution | Rule-heavy profile improved materially after avoiding repeated `FileBundle` map clones. | Cache entries now share bundles with `Arc`. | Current cache clone cost is no longer the dominant measured cost. | Flatten inherited rules into an immutable per-directory plan during checker initialization. |
| Traversal | CI/report rows show serial `jwalk` can be slower than raw `walkdir` traversal; parallel `jwalk` regressed small/pruned local fixtures. | Production keeps serial `jwalk` with pruning through `process_read_dir`. | Traversal is visible but not the main product hotspot. | Revisit a workload-aware traversal strategy only after phase rows show traversal dominates real fixtures. |
| Exclusion pruning | Ignored/generated fixtures stay very small end-to-end when pruning happens before validation. | `jwalk` `process_read_dir` prunes ignored directories before child reads. | Keep pruning in traversal; do not validate ignored generated trees. | Add fixture variants with nested generated directories and mixed ignored/allowed siblings. |
| Config load | Attribution and phase timing show config load is tiny relative to validation. | Patterns/regexes compiled once per check during checker initialization. | No current optimization justified beyond keeping parsing isolated. | Cache parsed config only with explicit fingerprinting and invalidation. |
| Configured direct requirements and `exists` counts | Direct-count attribution is measurable but below rule-heavy matching. | Direct content checks run only where config asks for them. | Not the next optimization target. | Build direct-child indexes if phase rows show direct checks dominate realistic projects. |
| Deterministic output sorting | New phase row tracks sort cost. | Sorting remains after validation to keep stable output. | Keep sorting unless phase data shows it matters. | Skip sort only for fail-fast or already ordered modes if exposed later. |
| Notation shape | LS-Lint-style suffix notation can be optimized because it maps to deterministic suffix lookup. | Existing notation fast path exploits simple suffix keys without changing semantics. | Current notation is good for suffix-heavy LS-Lint parity. | Consider a first-class extension-rule table or compiled rule index for future Assura-native notation instead of asking users to encode all rules as globs. |
| Incremental checking | Current report measures cold full-check performance only. | Documented cache-aware strategy separately. | Not implemented because invalidation is correctness-sensitive. | Add cache fingerprints for config, ignore rules, file metadata, and content-policy inputs. |

## Why This Is Enough For This PR

The PR now has an architecture-level performance contract instead of only
notes:

- The top-level comparison shows Assura versus LS-Lint on every tracked
  fixture.
- Traversal rows explain the `walkdir` versus `jwalk` tradeoff.
- Phase rows show where Assura spends time inside each fixture.
- The realistic fixture set covers library, web app, monorepo, rule-heavy, and
  ignored/generated-heavy shapes with real Assura configs.
- The hotspot matrix records what was optimized, why lower-priority areas were
  deferred, and what future research would change the architecture.

Future PRs should use the phase rows to justify optimization work. A claim that
an area is the next hotspot should point to its phase row across at least one
representative fixture, not only to a synthetic microbenchmark.
