---
title: Performance Decision Matrix
status: active
---

# Performance Decision Matrix

## Decision

Pause cold one-shot CLI micro-optimization for the stable beta lane unless a
new profiler-backed dominant cause appears. Keep the cold LS-Lint no-slower
gate as the release guardrail, keep the warm/session 2x evidence as the
repeat-use story, and move primary engineering effort back to core structure
validation quality.

Current checked evidence:

- Accepted cold LS-Lint-equivalent rows are `8/8` faster than native LS-Lint.
- Strict cold 2x remains incomplete at `2/8` accepted rows.
- Warm/session rows are `8/8` complete at 2x.
- Native Assura rows now have checked regression metadata and checked history.

## Stop Policy

Do not retain a cold structure optimization unless all of these are true on the
same remote host and comparable release bundle:

- The target public `assura-cli` row improves by at least `5%` and `0.75 ms`.
- The exact public `assura check --quiet` tie-breaker improves by at least `3%`.
- No accepted fixture regresses by more than `5%` or `0.25 ms`.
- `cargo xtask performance-no-slower` passes on the candidate report.
- Two same-host runs, preferably with order reversal, agree on direction.

Stop a sublane when any of these are true:

- Three plausible attempts in the same category fail the keep bar.
- The remaining strict-2x miss is below `1 ms` and dominated by process floor,
  launch overhead, or benchmark noise.
- A change only improves phase rows, check-only rows, or auxiliary entrypoints
  while the ordinary public command is flat or slower.
- The candidate needs user-visible command complexity only to satisfy a
  benchmark.

For beta, strict cold 2x is a diagnostic stretch goal. It is not a release
blocker while the accepted cold no-slower gate stays green.

## LS-Lint Structure Rows

Checked report: `benches/history/current.json`.

| Fixture | Assura ms | LS-Lint ms | Speedup | 2x status | Gap to 2x | Dominant checked phases | Decision |
| --- | ---: | ---: | ---: | --- | ---: | --- | --- |
| `ignored_generated_heavy_repo` | 1.499 | 5.650 | 3.77x | meets-target | -1.326 ms | config-load 0.03, walk-and-validate 0.02 | Protect; no optimization planned. |
| `many_configured_scopes_regression` | 14.950 | 19.075 | 1.28x | misses-target | +5.413 ms | walk-and-validate 5.12, config-load 4.15 | Pause micro-tuning; reopen only with profiler-backed evidence. |
| `monorepo_packages` | 1.873 | 2.520 | 1.35x | misses-target | +0.613 ms | walk-and-validate 0.34, config-load 0.17 | Keep no-slower; stop strict-2x chase. |
| `monorepo_policy` | 2.730 | 4.047 | 1.48x | misses-target | +0.706 ms | walk-and-validate 0.37, config-load 0.25 | Keep no-slower; stop strict-2x chase. |
| `multipart_extension_regression` | 3.695 | 10.203 | 2.76x | meets-target | -1.406 ms | walk-and-validate 1.00, config-load 0.02 | Protect; no optimization planned. |
| `rule_heavy_repo` | 2.384 | 3.700 | 1.55x | misses-target | +0.534 ms | walk-and-validate 0.39, config-load 0.17 | Keep no-slower; stop strict-2x chase. |
| `simple_library` | 1.748 | 2.301 | 1.32x | misses-target | +0.598 ms | walk-and-validate 0.25, config-load 0.06 | Keep no-slower; stop strict-2x chase. |
| `web_app` | 2.009 | 2.548 | 1.27x | misses-target | +0.735 ms | config-load 0.06, walk-and-validate 0.03 | Keep no-slower; stop strict-2x chase. |

## Retained Native Gains

Checked history: `benches/history/native-history.jsonl`.

| Native row | First checked ms | Current checked ms | Change | Decision |
| --- | ---: | ---: | ---: | --- |
| `native_large native:phase:fact-ingest-load` | 2993.6 | 137.0 | -95.4% | Retained product win; protect. |
| `native_large native:phase:incremental-replace-generation` | 2733.0 | 118.3 | -95.7% | Retained product win; protect. |
| `native_large native:phase:object-load-validate` | 236.3 | 188.7 | -20.1% | Retained product win; reopen only if gate or budget fails. |
| `native_large native:phase:repository-validate-total` | 257.7 | 209.8 | -18.6% | Retained product win; reopen only if gate or budget fails. |

## Rejected Pattern

Recent structure experiments repeatedly showed the same failure mode:

- Internal phase rows improved while `assura-cli` or exact
  `assura check --quiet` did not.
- Check-only or auxiliary rows improved while the normal public launcher
  regressed.
- Adjacent fixture spillover was mixed enough to erase confidence in the target
  win.
- Small-fixture strict-2x misses were sub-millisecond and not beta-relevant.

That makes more cold micro-tuning low expected value until a profile identifies
a new dominant public-command cost.

## Next Direction

Use performance effort to protect product value rather than chase smaller
numbers:

- Keep `cargo xtask performance-no-slower benches/history/current.json` as the
  cold LS-Lint release gate.
- Keep `cargo xtask native-performance-no-regression
  benches/history/native-current.json` as the Assura-native regression gate.
- Use `cargo perf-vps <label> <paths...>` only when a candidate has a
  profiler-backed hypothesis and can pass the stop policy above.
- Prefer warm/session, compiled-config, changed-path, daemon/editor, and
  core-structure validation improvements over new public entrypoint branching.
- Route the next major engineering lane to core structure validation stability:
  rule semantics, diagnostics, fixtures, config behavior, and beta support.
