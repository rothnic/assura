---
title: CLI-to-CLI Performance Decision Record
date: 2026-05-17
status: superseded
---

# CLI-to-CLI Performance Decision Record

## Status

This decision record has been superseded by the 2026-05-18 native LS-Lint
correction. It remains as historical context for the CLI-to-CLI verification
slice, but its old winner and speedup numbers came from the npm package wrapper
path and must not be used for current product claims.

This slice chooses the default traversal/execution path from measured
full-check CLI strategy rows. Parallel rule-application and rule-planned or
indexed execution remain explicitly deferred to a later architecture slice
because they require new rule-planning and deterministic result-accumulation
work, not just a walker substitution. The current public claim does not depend
on those unimplemented strategies.

## Decisions

1. Public LS-Lint comparison row family:
   use `assura-cli` versus `ls-lint-cli`.

2. Headline fixtures:
   only fixtures with `fixture_cohort: realistic-equivalent`,
   `native_ls_lint_parity: true`, and `evidence_role: headline-candidate`.

3. Diagnostic-only fixtures and rows:
   `synthetic-stress` fixtures are diagnostic because they stress bottlenecks
   rather than realistic equivalent repo shapes. `assura-in-process`,
   `assura:phase:*`, and `traversal:*` rows are diagnostic because they do not
   measure the same end-to-end CLI contract as LS-Lint.

4. Equivalent subprocess paths:
   yes for the current report. `assura-cli` runs the built Assura binary as a
   subprocess, and `ls-lint-cli` must run the prepared native binary from the
   pinned `@ls-lint/ls-lint@2.3.0` package as a subprocess. Dependency install,
   package resolution, Rust compilation, fixture generation, and binary
   discovery happen outside the measured loops.

5. Current Assura default execution architecture:
   walkdir is the default production full-check path. The fail-fast path remains
   deterministic serial `jwalk`, and the opt-in
   `ASSURA_CHECK_TRAVERSAL=jwalk-serial` and
   `ASSURA_CHECK_TRAVERSAL=parallel-jwalk` paths are retained for diagnostics.

6. Architecture support:
   the current public claim is supported by release CLI-to-CLI data. The report
   now includes diagnostic `strategy:walkdir-cli`,
   `strategy:jwalk-serial-cli`, and `strategy:jwalk-parallel-cli` rows. In the
   current 15-iteration realistic-equivalent report, serial `jwalk` has a
   87.563752 ms strategy bundle total, walkdir has an 87.59217 ms total, and
   parallel `jwalk` has a 92.004143 ms total. The serial `jwalk` and walkdir
   totals are effectively tied at a 0.028418 ms bundle gap, so the default was
   changed to walkdir for non-fail-fast full checks because it gives Assura a
   deterministic sorted baseline with explicit `filter_entry` exclusion
   pruning and no meaningful runtime regression in the measured bundle.
   The fastest individual row varies by fixture, but the differences are small
   enough that an adaptive walker is not justified by this slice alone.
   Parallel rule-application and rule-planned/indexed strategies are deferred
   because they are planner/result-architecture changes rather than additional
   traversal strategies.
   Research notes are recorded in
   `docs/analysis/2026-05-17-filesystem-validation-throughput-research.md`.

7. Corrected current realistic-equivalent result:
   in the 2026-05-18 `benches/history/current.json`, native LS-Lint wins every
   generated realistic-equivalent CLI row. Bundle total is Assura CLI
   105.666354 ms versus native LS-Lint CLI 62.239124 ms. The closest row is
   `ignored_generated_heavy_repo`: Assura CLI 16.712235 ms versus native
   LS-Lint CLI 15.761383 ms, or native LS-Lint 1.06x faster.

8. Supported website claim:
   the current generated-fixture CLI-to-CLI data supports saying that native
   LS-Lint is faster than Assura on the measured realistic-equivalent fixture
   bundle. It does not support the older claim that Assura is multiple times
   faster than LS-Lint.
