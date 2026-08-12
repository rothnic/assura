# Support-Grade Watch And Warm Runtime

## Goal

Replace one-shot watch behavior with a bounded continuous feedback loop and
promote daemon/cache behavior only where lifecycle, latency, and fallback are
observable.

## Requirements

- `assura watch` stays alive, coalesces filesystem events, reruns affected
  checks within a bounded maximum batch window, and exits cleanly on
  cancellation even while edits continue.
- File-scoped watches survive editor-style atomic replacement by subscribing
  to the containing directory while filtering validation back to the requested
  file. An unrecoverable watcher error emits one degraded event and terminates
  rather than leaving an apparently healthy but stale process.
- The positional path is the watched and fully checked scope. An explicit
  configuration outside that scope remains watched without treating unrelated
  sibling changes as project edits.
- Ignore Assura runtime output and generated/noisy trees to avoid self-triggered
  loops.
- Reuse the existing validation engine and warm runtime rather than adding a
  watch-only validator.
- Use affected-path checks only when the configured policy can be proven from
  one changed path. Cross-path policy falls back to a prepared full-scope check.
- Report cold/warm/fallback mode, checked scope, changed paths, findings,
  recoverable errors, and cache health in bounded text and machine-readable
  diagnostics.
- Internal hot-check status publication and watcher invalidation share one
  generation lock so neither a newer edit can be published as clean nor a
  completed validation can be overwritten by a delayed dirty publication.
- Public managed-daemon changed-path IPC returns validation exit `1` for a
  failing report and the same versioned JSON schema as local fallback.
- Preserve the five checked p95 latency budgets.

## Acceptance Criteria

- [x] A deterministic integration test edits a file and observes a second report.
- [x] Burst changes coalesce and do not emit unbounded repeated feedback.
- [x] Config changes invalidate the relevant compiled policy/cache state.
- [x] External configuration and requested subdirectory scope have integration
  coverage.
- [x] File-scoped watch survives atomic replacement and a subsequent edit.
- [x] Cross-path policy cannot return a false incremental pass through watch or
  daemon changed-path commands.
- [x] Ctrl-C/process cancellation leaves no stale runtime state.
- [x] Sustained edit streams emit within the maximum batch window and remain
  immediately cancellable.
- [x] Default text output includes scope, changed paths, fallback, errors, and
  bounded actionable findings.
- [x] A deterministic concurrency test proves clean status cannot overwrite a
  later watcher generation.
- [x] A deterministic concurrency test proves delayed dirty publication cannot
  overwrite a later clean validation result.
- [x] Managed-daemon check-path reports validation failure through process exit
  and local/managed responses share one versioned schema.
- [ ] Windows CI executes managed-daemon lifecycle and watch cancellation tests
  rather than compiling or skipping those paths only.
- [x] All five warm-loop p95 budgets pass.
- [x] Watch, daemon, and cache support rows have support-grade evidence.

## Validation

```bash
cargo test --test watch_cli
cargo test --test daemon_cli_tests
cargo xtask warm-loop-no-regression benches/history/warm-loop-current.json
cargo xtask target-state
cargo xtask evidence
cargo run --quiet -- check --format json .
```

## Review Blocking Criteria

Block on polling without bounded backoff, duplicate validation engines,
self-trigger loops, hidden fallback, stale daemon state, or latency regressions.
