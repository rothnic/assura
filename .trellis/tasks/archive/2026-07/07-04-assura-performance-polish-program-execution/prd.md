# Assura Performance Polish Program Execution

## Goal

Advance `docs/goals/assura-performance-polish-program.md` from live repo state
by implementing the smallest reviewed slice that makes native-performance
regression policy product-grade without weakening the existing LS-Lint
no-slower gate. The first slice should keep LS-Lint comparison proof separate,
calibrate native regression metadata from checked native rows, and leave a
clear evidence-backed next optimization target.

## What I Already Know

- The roadmap still names Performance Polish as the next planned lane after PR
  `#139`; the current recommended goal remains
  `docs/goals/assura-performance-polish-program.md`.
- `benches/history/current.json` and
  `website/public/data/performance/current.json` still represent the LS-Lint
  comparison lane. The accepted-row cold claim is `not-complete`, but
  `cargo xtask performance-no-slower` remains the merge gate for accepted
  LS-Lint-equivalent fixtures.
- `benches/history/native-current.json` and
  `website/public/data/performance/native-current.json` exist and match. They
  already contain the native matrix and phase-attribution rows from PR `#139`.
- The current native gate in `cargo xtask native-performance-no-regression`
  checks schema, matrix coverage, passing status, sample presence, expected
  exit statuses, and any ad hoc `latency_threshold_met` booleans, but it does
  not enforce a baseline-calibrated per-row regression status derived from the
  checked native report.
- The checked native rows show the largest measured native attribution costs in
  `native_large` project-facts ingest/index load and incremental generation
  replacement, while the LS-Lint comparison lane still shows a distinct cold
  CLI-floor/config-load hotspot in `many_configured_scopes_regression`.

## Assumptions

- The first performance-polish slice does not need to solve the highest-impact
  optimization target itself if it can make the regression policy explicit and
  durable while recording the evidence-backed next target.
- Adding native regression metadata to the report schema is acceptable as long
  as LS-Lint no-slower proof remains unchanged and native rows do not become
  accepted LS-Lint-equivalent evidence by mistake.
- Checked native artifacts can be refreshed on this machine with comparable
  release commands.

## Open Questions

- None currently blocking. If the code suggests two plausible regression
  policies, prefer the one that is deterministic from checked rows and easy to
  review in JSON artifacts and docs.

## Requirements

- Keep `cargo xtask performance-no-slower` and accepted LS-Lint-equivalent
  rows as a separate gate from Assura-native regression checks.
- Add machine-readable native regression metadata to native report rows and/or
  summaries so reviewers can tell whether a row is within calibrated baseline,
  regressed, or exempt.
- Calibrate the first native regression policy from checked native report rows,
  not from invented round numbers.
- Refresh checked native report artifacts with comparable commands after the
  schema/gate change.
- Update docs only where the native regression-policy claim changes.
- Record which measured optimization target should be tackled next after this
  policy slice.

## Acceptance Criteria

- [ ] Native report rows include explicit regression metadata that is separate
      from LS-Lint no-slower claim fields.
- [ ] `cargo xtask native-performance-no-regression` enforces the calibrated
      native regression policy from checked baseline-derived thresholds or
      statuses.
- [ ] `benches/history/native-current.json` and
      `website/public/data/performance/native-current.json` are refreshed from
      comparable commands and still match.
- [ ] Native performance docs explain the calibrated regression gate without
      implying those rows satisfy the cold LS-Lint parity claim.
- [ ] A durable note records the current highest-impact next optimization
      target from native attribution or CLI-floor evidence.
- [ ] Scoped validation passes for the touched performance-report, docs, and
      checked-artifact surfaces.

## Definition Of Done

- Code, tracked artifacts, and docs agree on how native regression is judged.
- LS-Lint no-slower behavior is unchanged for accepted fixture rows.
- Reviewer-facing evidence shows the before/after native report and gate
  behavior.
- The task leaves the next optimization lane narrower than it started.

## Out Of Scope

- Solving all remaining cold CLI-floor misses in this task.
- Reworking the entire native performance matrix or adding new fixture classes.
- Adding any persistent cache/store dependency.
- Relaxing any accepted LS-Lint no-slower merge gate.

## Technical Notes

- Goal: `docs/goals/assura-performance-polish-program.md`
- Revalidation note:
  `docs/analysis/2026-07-04-performance-polish-program-revalidation.md`
- Roadmap: `.trellis/spec/assura/roadmap.md`
- Native report code: `src/cli/performance_report/native.rs`,
  `src/cli/performance_report/rows.rs`, `src/cli/performance_report/mod.rs`
- Native gate code: `xtask/src/main.rs`
- Native docs:
  `website/src/content/docs/reference/performance.mdx`,
  `website/src/content/docs/reference/performance-implementation.mdx`,
  `website/src/content/docs/reference/performance-test-cases.mdx`
- Checked artifacts:
  `benches/history/native-current.json`,
  `benches/history/native-history.jsonl`,
  `website/public/data/performance/native-current.json`,
  `website/public/data/performance/native-history.jsonl`
