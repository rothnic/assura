---
id: goal-assura-cli-to-cli-ls-lint-performance-verification
type: goal
title: Assura CLI-to-CLI LS-Lint performance verification
status: planned
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
evidence, which Assura execution strategy should be the default and whether the
website can honestly claim that Assura is multiple times faster than LS-Lint on
realistic equivalent test cases.

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
- Full-check Assura strategy comparisons.
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

## Assura Strategy Decision

The PR must choose the Assura execution strategy from full-check data, not from
traversal-only data.

Candidate strategies to measure:

- current deterministic serial production path,
- walkdir-compatible baseline path where still available,
- serial `jwalk` path,
- parallel `jwalk` collection with deterministic final sorting,
- adaptive strategy if the implementation can choose based on fixture or config
  shape without compromising determinism.

The selected default must satisfy:

- deterministic text and JSON output,
- stable relative paths,
- correct exclusion pruning,
- correct fail-fast semantics or an explicitly tested serial fail-fast path,
- no shared mutable hot-path state unless profiling proves it is acceptable,
- no silent regression beyond the agreed threshold on stable realistic fixtures,
- best or clearly defensible total CLI runtime across the realistic-equivalent
  fixture set.

If the fastest strategy differs by fixture class, the PR must either implement
an adaptive strategy or document why a single default is the right product
tradeoff.

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

The headline may say "multiple times faster than LS-Lint" only if the
CLI-to-CLI realistic-equivalent fixture set supports that statement. If one
realistic-equivalent fixture is not multiple times faster, the headline must use
the weakest-case-supported claim instead.

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

- [ ] `assura performance-report` or a companion command emits
      CLI-to-CLI Assura and LS-Lint rows.
- [ ] The public Assura-versus-LS-Lint claim uses `assura-cli` and
      `ls-lint-cli` rows, not in-process Assura rows.
- [ ] The measured loops prepare binaries/configs once and exclude build,
      install, package resolution, and fixture generation.
- [ ] Realistic-equivalent fixture rows include file counts, ignored file
      counts, directory counts, rule counts, native parity status, and config
      references in machine-readable output.
- [ ] Full-check strategy rows exist for the candidate Assura execution
      strategies needed to choose the default logically.
- [ ] The selected Assura default strategy is justified by full-check CLI
      runtime, correctness behavior, and deterministic output requirements.
- [ ] Synthetic stress and traversal-only rows are labeled as diagnostics and
      do not drive the headline website claim.
- [ ] The website performance page renders a concise comparison table with
      percent difference, speedup, fixture scale, rule surface, and config
      references.
- [ ] A desktop and mobile visual review is recorded after the final website
      changes.
- [ ] The PR body links to the machine-readable report data, website page or
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
5. Which Assura execution strategy is the default after this work?
6. Why is that strategy logically supported by full-check data?
7. What is the weakest realistic-equivalent fixture result?
8. What claim does the weakest result permit the website to make?

## Stop Condition

Do not stop at better wording. Stop only when the measured data, selected
Assura execution strategy, website presentation, and visual review all support
the same defensible story.
