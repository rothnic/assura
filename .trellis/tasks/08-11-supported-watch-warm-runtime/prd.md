# Support-Grade Watch And Warm Runtime

## Goal

Replace one-shot watch behavior with a bounded continuous feedback loop and
promote daemon/cache behavior only where lifecycle, latency, and fallback are
observable.

## Requirements

- `assura watch` stays alive, coalesces filesystem events, reruns affected
  checks, and exits cleanly on cancellation.
- Ignore Assura runtime output and generated/noisy trees to avoid self-triggered
  loops.
- Reuse the existing validation engine and warm runtime rather than adding a
  watch-only validator.
- Report cold/warm/fallback mode and cache health in machine-readable diagnostics.
- Preserve the five checked p95 latency budgets.

## Acceptance Criteria

- [ ] A deterministic integration test edits a file and observes a second report.
- [ ] Burst changes coalesce and do not emit unbounded repeated feedback.
- [ ] Config changes invalidate the relevant compiled policy/cache state.
- [ ] Ctrl-C/process cancellation leaves no stale runtime state.
- [ ] All five warm-loop p95 budgets pass.
- [ ] Watch, daemon, and cache support rows have support-grade evidence.

## Validation

```bash
cargo test --test watch_cli
cargo test --test daemon_cli
cargo xtask warm-loop-no-regression benches/history/warm-loop-current.json
cargo xtask target-state
cargo xtask evidence
cargo run --quiet -- check --format json .
```

## Review Blocking Criteria

Block on polling without bounded backoff, duplicate validation engines,
self-trigger loops, hidden fallback, stale daemon state, or latency regressions.
