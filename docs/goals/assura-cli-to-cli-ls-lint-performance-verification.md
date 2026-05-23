---
id: goal-assura-cli-to-cli-ls-lint-performance-verification
type: goal
title: Assura CLI-to-CLI LS-Lint performance verification
status: complete
created: 2026-05-17
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/goals/assura-ls-lint-realistic-parity-core-performance-amendment.md
  - docs/analysis/2026-05-16-performance-results-interpretation.md
  - docs/analysis/2026-05-16-website-visual-review.md
  - benches/history/current.json
  - website/public/data/performance/current.json
  - website/src/content/docs/reference/performance.mdx
  - src/cli/check.rs
  - src/cli/performance_report/mod.rs
  - src/cli/performance_report/fixtures.rs
---

# Assura CLI-to-CLI LS-Lint Performance Verification

## Objective

Replace any ambiguous Assura-versus-LS-Lint performance claim with a fair,
reviewable CLI-to-CLI verification path. The comparison must measure the
end-to-end time for a built `assura` CLI binary and a prepared LS-Lint CLI
binary, each pointed at equivalent configuration files for the same materialized
fixture tree.

The goal is not to prove a flattering number. The goal is to decide, from
evidence, which Assura execution architecture should be the default and what
winner, if any, the website can honestly claim for realistic equivalent test
cases.

## Problem Statement

The current PR data is useful but not yet enough for a durable public claim:

- Assura top-level rows are measured through the in-process checker path, while
  LS-Lint rows are measured by executing the LS-Lint CLI binary.
- Fixture descriptions on the website are manually derived from generator code
  and can drift from machine-readable report data.
- The realistic fixture manifest documents parity intent, but the performance
  report does not currently treat fixture metadata as a first-class data
  contract.
- Traversal-only rows show that parallel `jwalk` can be faster, but they do not
  prove which full-check execution strategy should be the production default.
- Synthetic stress fixtures are useful diagnostic evidence, but they must not
  be mixed into the headline claim for realistic equivalent LS-Lint cases.
- The website needs a concise, visual, auditable comparison that makes the
  benefit and the limits clear without forcing reviewers to infer the setup.

## Scope

This goal covers:

- CLI-to-CLI benchmarking for Assura and LS-Lint.
- Fixture equivalence and fixture metadata reporting.
- Full-check Assura execution architecture comparisons.
- Research into how comparable filesystem validation, linting, search, and
  ignore-aware traversal tools maximize throughput.
- Website evidence updates after the data contract is corrected.
- Visual review of the rendered website before the PR is considered complete.

This goal does not cover:

- Cold package install timing for LS-Lint.
- General claims about every possible LS-Lint repository.
- New Assura notation features outside the existing LS-Lint-compatible fixture
  surface.
- Incremental cache work unless the selected execution strategy requires it.

## Required Measurement Contract

The benchmark must produce separate, clearly named row families:

- `assura-cli`: executes a built Assura binary as a subprocess.
- `ls-lint-cli`: executes the prepared `@ls-lint/ls-lint@2.3.0` binary as a
  subprocess.
- `assura-in-process`: optional diagnostic row for current internal phase
  timing, never the headline comparison against LS-Lint.
- `assura:phase:*`: optional phase diagnostics for config discovery, config
  load, checker init, configured validation, walk and validate, and report sort.
- `traversal:*`: traversal-only rows for walkdir, serial `jwalk`, and parallel
  `jwalk`, labeled as diagnostic only.
- `strategy:*`: full-check Assura strategy rows that run equivalent validation
  work under each candidate traversal/execution strategy.

The measured loop must exclude dependency installation, package resolution,
Rust compilation, fixture generation, and binary discovery. It must include
CLI process startup, config discovery/loading, traversal, validation, output
construction, and process exit.

The benchmark should use release binaries for the public comparison. Debug or
development binaries may be recorded as diagnostics but must not drive the
website headline.

## Fixture Equivalence Contract

Every realistic comparison fixture must have machine-readable metadata in the
performance output:

- fixture id
- fixture cohort: `realistic-equivalent`, `synthetic-stress`, or
  `diagnostic-only`
- source type: generated, external pinned repo, or materialized local fixture
- checked file count
- ignored file count
- directory count
- rule count
- rule surface summary
- whether the fixture is native LS-Lint parity
- Assura config path
- LS-Lint config path
- config generation method or shared config pair id
- expected exit status for Assura and LS-Lint

The report should fail or mark a row invalid when a fixture lacks this metadata.
The website must render fixture scale and rule surface from the machine-readable
report data, not from manually transcribed prose.

Equivalent means:

- both tools run against the same fixture directory,
- both tools validate the same intended naming and structure rule surface where
  LS-Lint supports it,
- Assura-only extensions are labeled separately and excluded from the native
  LS-Lint parity headline,
- ignored/generated paths are present in the same fixture tree and ignored by
  equivalent config behavior,
- expected valid and invalid outcomes are asserted where the fixture is used for
  correctness, not only performance.

## Assura Execution Architecture Decision

The PR must choose the Assura execution architecture from full-check data, not
from traversal-only data. "Execution architecture" means the complete design of
how Assura walks paths, prunes ignored work, plans applicable rules, applies
rules, accumulates violations, preserves deterministic output, and avoids
repeating work across files, directories, and rule groups.

Candidate dimensions to investigate and measure:

- path walker choice: current deterministic serial path, walkdir-compatible
  baseline, serial `jwalk`, parallel `jwalk`, or another justified walker,
- traversal shape: one pass versus multiple passes, directory-first pruning,
  streaming validation versus collect-then-validate,
- parallelism boundary: parallel directory collection, parallel file
  validation, parallel rule evaluation, or deliberately serial phases where
  synchronization cost would dominate,
- rule planning: precompute which rules can apply by directory, extension,
  exact filename, glob, or direct child count before entering the hot path,
- rule specialization: fast paths for suffix/extension rules, exact filenames,
  `.dir` rules, direct-child `exists` counts, and ignored directory pruning,
- caching and indexing: compiled regex/glob reuse, path component caches,
  per-directory child summaries, extension buckets, and any other reusable
  indexes that reduce repeated path/rule matching,
- adaptive execution: choose a strategy based on rule shape, fixture size,
  ignored-directory density, and whether fail-fast is enabled,
- output architecture: preserve deterministic sorted output without making
  every hot-path operation contend on shared mutable state.

Candidate full-check strategies to benchmark:

- current deterministic serial production path,
- walkdir-compatible baseline path where still available,
- serial `jwalk` path,
- parallel `jwalk` collection plus deterministic validation/sorting,
- parallel collection plus parallel rule application,
- rule-planned or indexed execution path,
- adaptive strategy if the implementation can choose based on fixture or config
  shape without compromising determinism.

Required research before choosing the final architecture:

- inspect how comparable tools handle high-throughput traversal, ignore
  pruning, rule planning, caching, and deterministic output,
- identify which ideas are relevant to Assura's structure-first rules and which
  are not,
- record the findings in `docs/analysis/` with citations or local source
  references before implementing non-obvious architecture changes,
- use the research to challenge current assumptions about what must remain
  serial, what can be precomputed, and what can be made adaptive.

The selected default must satisfy:

- deterministic text and JSON output,
- stable relative paths,
- correct exclusion pruning,
- correct fail-fast semantics or an explicitly tested serial fail-fast path,
- no shared mutable hot-path state unless profiling proves it is acceptable,
- no silent regression beyond the agreed threshold on stable realistic fixtures,
- best or clearly defensible total CLI runtime across the realistic-equivalent
  fixture set.

If the fastest architecture differs by fixture class, the PR must either
implement an adaptive strategy or document why a single default is the right
product tradeoff.

## Website Evidence Requirement

The performance page must be reorganized around one concise comparison table for
realistic equivalent fixtures. Each row should show:

- fixture id,
- fixture scale summary,
- rule count or rule-surface summary,
- Assura CLI median runtime,
- LS-Lint CLI median runtime,
- percent lower or higher runtime,
- speedup ratio,
- equivalence/config reference,
- row status.

The page must separate:

- headline realistic-equivalent CLI comparison,
- synthetic stress fixtures,
- traversal-only diagnostics,
- in-process phase diagnostics,
- historical trend or audit log.

The headline may claim a specific winner only if the CLI-to-CLI
realistic-equivalent fixture set supports that statement. If one
realistic-equivalent fixture does not support a broad winner claim, the
headline must use the weakest-case-supported claim instead.

The website must include a short fairness note that says exactly what the
measured CLI rows include and exclude. It must not rely on large prose blocks to
explain the value proposition.

## Required Visual Review

Before the PR is considered complete, run and record a visual review of the
performance page:

- desktop viewport screenshot,
- mobile viewport screenshot,
- confirmation that tables fit without horizontal clipping that hides core
  columns,
- confirmation that code/import text is rendered as intended and not as broken
  MDX syntax,
- confirmation that the table communicates the Assura-versus-LS-Lint benefit
  without requiring the reader to inspect raw JSON,
- screenshot paths or browser-review notes checked into `docs/analysis/` or
  attached to the PR.

## Acceptance Criteria

- [x] `assura performance-report` or a companion command emits
      CLI-to-CLI Assura and LS-Lint rows.
- [x] The public Assura-versus-LS-Lint claim uses `assura-cli` and
      `ls-lint-cli` rows, not in-process Assura rows.
- [x] The measured loops prepare binaries/configs once and exclude build,
      install, package resolution, and fixture generation.
- [x] Realistic-equivalent fixture rows include file counts, ignored file
      counts, directory counts, rule counts, native parity status, and config
      references in machine-readable output.
- [x] Full-check strategy rows exist for the candidate Assura execution
      architectures needed to choose the default logically.
- [x] The selected Assura default architecture is justified by full-check CLI
      runtime, correctness behavior, deterministic output requirements, and any
      research findings from comparable tools.
- [x] Synthetic stress and traversal-only rows are labeled as diagnostics and
      do not drive the headline website claim.
- [x] The website performance page renders a concise comparison table with
      percent difference, speedup, fixture scale, rule surface, and config
      references.
- [x] A desktop and mobile visual review is recorded after the final website
      changes.
- [x] The PR body links to the machine-readable report data, website page or
      preview, visual review evidence, and the strategy decision rationale.

## Required Verification Commands

Run these before marking the goal complete:

```bash
cargo fmt --all -- --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
cargo run --quiet -- performance-report --output <artifact> --iterations <n>
cd website && pnpm build
```

Also run the browser/visual review against the local website page or preview
URL used for PR evidence.

## Decision Record Required

The PR must include a short decision record answering:

1. Which row family is used for the public LS-Lint comparison?
2. Which fixtures count toward the headline claim?
3. Which fixtures are diagnostic-only and why?
4. Are Assura and LS-Lint measured through equivalent CLI subprocess paths?
5. Which Assura execution architecture is the default after this work?
6. Why is that architecture logically supported by research and full-check data?
7. What is the weakest realistic-equivalent fixture result?
8. What claim does the weakest result permit the website to make?

## Stop Condition

Do not stop at better wording. Stop only when the measured data, selected
Assura execution strategy, website presentation, and visual review all support
the same defensible story.

## Progress Log

| Date | Phase | Notes | Evidence |
|------|-------|-------|----------|
| 2026-05-17 | Planning | Created a narrow Trellis task for the first implementation slice: report row-family separation, Assura CLI subprocess measurement, and fixture metadata plumbing. Current checked-in report data still uses ambiguous `assura` and `ls-lint` `tool_name` rows with no first-class row family or fixture metadata object. | `.trellis/tasks/05-17-cli-to-cli-ls-lint-performance-verification/prd.md`; `benches/history/current.json`; `src/cli/performance_report/mod.rs` |
| 2026-05-17 | CLI-to-CLI report slice | Added `assura-cli`, `ls-lint-cli`, `assura-in-process`, `assura:phase:*`, `traversal:*`, and initial `strategy:*` row families; added fixture metadata to report rows; refreshed release report data; moved the website comparison to render from `current.json`; recorded desktop/mobile visual review. Broader walkdir-compatible and parallel rule-application strategy rows remain open. | `src/cli/performance_report/`; `benches/history/current.json`; `website/src/components/performance-evidence.astro`; `docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md`; `docs/analysis/2026-05-17-performance-cli-to-cli-visual-review.md` |
| 2026-05-17 | Walkdir full-check strategy slice | Added `ASSURA_CHECK_TRAVERSAL=walkdir` and `strategy:walkdir-cli` full-check rows, plus focused CLI tests for sorted JSON output, exclusion pruning, default equivalence, and fail-fast determinism. The 15-iteration release report shows walkdir and serial `jwalk` effectively tied across the realistic bundle; walkdir is now the default non-fail-fast path, while deterministic serial `jwalk` remains the fail-fast path and an opt-in diagnostic strategy. | `src/cli/check/traversal.rs`; `tests/cli_check_tests.rs`; `benches/history/current.json`; `docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md`; `docs/analysis/2026-05-17-filesystem-validation-throughput-research.md` |
| 2026-05-17 | Research and architecture deferral | Recorded primary-source research for walkdir pruning/sorting, jwalk parallel traversal, ripgrep ignore-aware search, and ESLint cache semantics. The decision record explicitly defers parallel rule-application and indexed/rule-planned execution as later planner/result-architecture work; the current default decision is scoped to measured full-check CLI traversal strategies. | `docs/analysis/2026-05-17-filesystem-validation-throughput-research.md`; `docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md` |
| 2026-05-17 | Final verification and PR evidence | Ran the required local gates, refreshed the release CLI-to-CLI report with 15 iterations, rebuilt the website, captured desktop/mobile screenshots, and updated PR #11 with report, website, visual review, research, and strategy-decision links. The mobile visual pass found a horizontal clipping issue in the generated evidence section; the final screenshots verify the stacked mobile table. | `benches/history/current.json`; `website/public/data/performance/current.json`; `docs/analysis/2026-05-17-performance-cli-to-cli-visual-review.md`; `https://github.com/rothnic/assura/pull/11` |
