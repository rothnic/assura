---
title: CLI-to-CLI Performance Decision Record
date: 2026-05-17
status: current
---

# CLI-to-CLI Performance Decision Record

## Status

This is the current decision record for the CLI-to-CLI verification slice. It
replaces the old public comparison interpretation that used the in-process
Assura top-level row as the comparison row.

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
   subprocess, and `ls-lint-cli` runs the prepared cached LS-Lint binary as a
   subprocess. Dependency install, package resolution, Rust compilation,
   fixture generation, and binary discovery happen outside the measured loops.

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

7. Weakest realistic-equivalent result:
   in `benches/history/current.json`, `rule_heavy_repo` is the weakest current
   realistic-equivalent CLI row: Assura CLI median 19.492293 ms versus LS-Lint
   CLI median 112.401829 ms, or 82.7% lower runtime and 5.77x faster.

8. Supported website claim:
   the current release CLI-to-CLI data supports saying that Assura CLI is
   multiple times faster than LS-Lint CLI on the current realistic-equivalent
   native-parity fixture bundle. The website uses the weaker per-row result as
   the bound rather than the stronger bundle total alone.
