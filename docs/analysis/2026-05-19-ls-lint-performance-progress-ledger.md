---
title: LS-Lint Performance Progress Ledger
date: 2026-05-19
status: active
---

# LS-Lint Performance Progress Ledger

## Purpose

This ledger is the routing document for Assura's LS-Lint performance work. It
keeps the progress, rejected experiments, remaining research areas, and current
"point in sand" in one place so future work does not rediscover the same
dead ends.

Speed is the priority. Binary size matters only when it materially affects
startup latency or removes unnecessary work from a hot path. A smaller binary
that benchmarks slower is not progress.

## Tracking Artifacts

Use these files as the durable source of truth before starting or describing
new performance work:

| Artifact | Purpose |
| --- | --- |
| `benches/history/current.json` | Current machine-readable cold and warm verdicts. |
| `benches/history/ls-lint-comparison-history.jsonl` | Append-only checked-in performance history. |
| `website/public/data/performance/current.json` | Website copy of the current report. |
| `website/public/data/performance/ls-lint-comparison-history.jsonl` | Website copy of the checked-in history. |
| `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md` | Completion audit and rejected experiment evidence. |
| `docs/analysis/2026-05-19-ls-lint-performance-scope-decision.md` | Scope decision for cold, warm, and diagnostic claims. |
| `docs/goals/assura-native-ls-lint-performance-rearchitecture.md` | Active goal log and detailed implementation trail. |

The current goal document is intentionally not the only record. It is capped by
project structure limits and should stay focused on the active goal trail. Use
this ledger and the completion audit for compact progress tracking, research
triage, and future "do not repeat this" notes.

## Current Point In Sand

The Linux static-CRT cold-subprocess 2x goal is complete in the checked-in
current report. The default local macOS dynamic build remains a separate
incomplete diagnostic and should not be used to contradict the scoped Linux
release claim.

Source of truth:

```text
benches/history/current.json
claim_summary.two_x_claim_verdict = complete
claim_summary.measured_iterations = 5
claim_summary.assura_faster_count = 6
claim_summary.two_x_pass_count = 6
claim_summary.two_x_fail_count = 0
claim_summary.aggregate_speedup_ratio = 2.8980855186211874
assura-check-cli.assura_binary_profile = release-static-crt
warm_claim_summary.assura_row_family = assura-check-dirty-project-session-cli
warm_claim_summary.two_x_claim_verdict = complete
warm_claim_summary.two_x_pass_count = 6
warm_claim_summary.two_x_fail_count = 0
warm_claim_summary.aggregate_speedup_ratio = 25.213361575198398
```

Current interpretation:

- `assura-check-cli` is faster than native LS-Lint on all six current
  realistic-equivalent generated fixtures under the Linux static-CRT release
  build.
- The checked-in current report clears the 2x gate on all six fixtures.
- The validation engine is not the bottleneck. In-process Assura validation is
  below every 2x target.
- The earlier default dynamic cold misses were dominated by subprocess/startup
  floor plus dynamic loader overhead; Linux static CRT removes enough of that
  overhead to complete the gate.
- The one-shot warm dirty-project daemon client remains useful attribution but
  is still constrained by client process startup on small rows.
- The persistent warm dirty-project session CLI row is faster than native
  LS-Lint on all six realistic-equivalent fixtures and clears the 2x gate on
  every tracked fixture.
- The direct daemon socket diagnostic row is far below every 2x target, which
  isolates the remaining warm CLI miss to the client process boundary rather
  than daemon-side validation.
- Further cold-path work should focus on preserving the Linux static-CRT release
  path and deciding whether macOS needs a separate scoped claim.

## Decision Rules

Use these rules before starting another performance slice:

1. Do not reopen a cold-start experiment unless it names a new source of
   measurable startup work and can plausibly remove about 1.0-1.7 ms from
   multiple small-fixture rows.
2. Do not treat binary-size reduction as success. Keep a size reduction only
   when release-mode runtime improves or it removes an unnecessary hot-path
   dependency without regression.
3. Do not count diagnostic rows as the cold headline claim. `claim_summary`
   remains tied to `assura-check-cli`; warm/editor-session rows are summarized
   by `warm_claim_summary`.
4. If a fixture's 2x target is below the measured Rust CLI floor, classify that
   miss as an execution-model problem, not as a validation-engine bug.
5. Prefer warm/editor-session work once a cold hypothesis only changes parser
   shape, report formatting, binary size, or duplicate compiled-artifact
   probing. Those families have already been measured and rejected.
6. Keep every retained or rejected experiment tied to a concrete artifact path
   before using it in PR or website language.
7. Treat binary size as an input to speed analysis, not a goal. If a smaller
   binary does not improve measured runtime, reject the change or keep it only
   for unrelated maintainability with explicit non-performance wording.
8. Once a bounded cold-path spike cannot identify a credible path to removing
   about 1.0-1.7 ms from multiple small rows, stop searching that family and
   move back to warm/editor-session product work.

## Experiment Tracking Contract

Every future performance slice should leave one durable record in this ledger
or the completion audit before it is used in PR language:

- hypothesis and changed files,
- command and artifact path,
- headline `claim_summary` / `warm_claim_summary` result,
- retained, rejected, or inconclusive decision,
- reason the result changes or does not change the current point in sand.

Short smokes are allowed for screening, but they should not refresh checked-in
history unless they beat the retained state with enough headroom to survive a
5-iteration report. If an experiment is only noise-level, keep the current
tracked data and document the experiment as non-completion evidence.

## Progress Over Time

| Stage | What changed | Evidence | Result |
| --- | --- | --- | --- |
| Original comparison | Benchmarked LS-Lint through `node_modules/.bin/ls-lint`. | Historical reports before the native-binary correction. | Invalid for product claims; the apparent ~100 ms LS-Lint floor was the Node wrapper. |
| Native LS-Lint correction | Resolved and executed the packaged native Go binary from pinned `@ls-lint/ls-lint@2.3.0`. | `src/cli/performance_report/ls_lint.rs`; `docs/analysis/2026-05-18-native-ls-lint-performance-gap-review.md`. | Revealed the real gap: full `assura check` was slower than native LS-Lint. |
| Check-only CLI split | Added `crates/assura-check-cli` and measured `assura-check-cli` instead of the full product CLI for the focused check path. | `crates/assura-check-cli/src/main.rs`; tracked performance rows. | Moved from slower-than-LS-Lint full CLI to faster-than-LS-Lint on all tracked realistic rows. |
| Feature gating | Kept full CLI, git, markdown, graph, watch, and diagnostic traversal surfaces out of the check-only binary where possible. | `Cargo.toml`; `crates/assura-check-cli/Cargo.toml`; artifact inspection in the completion audit. | Removed unrelated startup surfaces. |
| LS-Lint-compatible fast path | Compiled LS-Lint-compatible scopes/rules/naming structures and used a narrow fast validator for compatible configs. | `src/cli/check/ls_fast*.rs`; parity and benchmark tests. | In-process validation dropped below every 2x target. |
| Fair subprocess loops | Measured status execution with stdout/stderr sent to null instead of timing pipe capture. | Performance harness rows and feasibility fields. | Produced fairer CLI-to-CLI evidence and explicit process-floor attribution. |
| Hot/editor-session architecture | Added daemon/client/status-file/dirty-path/prepared-check paths. | `crates/assura-check-cli/src/server.rs`; `server_dirty.rs`; `status.rs`; `src/cli/check/prepared.rs`. | Demonstrated the direction for repeated agent/editor checks, but remains diagnostic under the current cold headline gate. |
| Compiled config artifacts | Added `assura-check-compile-config` and `assura-check-compiled` plus portable artifact/fingerprint contracts. | `crates/assura-check-cli/src/compile_config.rs`; `compiled.rs`; `src/cli/check/compiled_artifact.rs`. | Matches the config-dirty model and avoids YAML in explicit compiled runs, but the diagnostic row is not universal 2x. |
| Cache/fingerprint hardening | Added source config fingerprints to compiled, prepared, and cached paths. | `src/cli/check/compiled_fingerprint.rs`; `prepared.rs`; `cache.rs`. | Avoids unnecessary config reads/reloads when freshness is proven; useful for warm paths, not sufficient for cold universal 2x. |
| Completion reporting | Added machine-readable `claim_summary` and low-sample guard. | `src/cli/performance_report/claim_summary.rs`; website contract tests. | Prevents accidental completion claims from diagnostic or low-sample rows. |
| Floor attribution hardening | Added a minimal Assura-built no-op CLI floor row separate from the status-file product diagnostic. | `crates/assura-check-cli/src/noop.rs`; `target/performance/noop-rust-floor-smoke-2.json`. | Keeps `/usr/bin/true`, no-op Assura CLI startup, and status-file warm diagnostics distinct. This explained the dynamic-build floor before Linux static-CRT completed the cold gate. |
| Warm gate reporting | Added a separate `warm_claim_summary` for editor-session rows. | `src/cli/performance_report/claim_summary.rs`; `target/performance/warm-claim-summary-smoke.json`; `target/performance/session-warm-summary-smoke.json`. | Makes warm/editor-session progress machine-readable without weakening the cold `claim_summary` gate. |
| Dirty-path de-duplication | Avoided validating the same dirty file twice when watcher paths and explicit editor paths overlap. | `crates/assura-check-cli/src/server_dirty.rs`; `target/performance/dirty-path-dedupe-smoke.json`. | Improved short-smoke warm dirty-project evidence, but the one-shot client remained process-bound. |
| Direct daemon socket profile | Added `assura-check-dirty-project-socket` as a diagnostic row that sends dirty-project requests directly to the hot daemon without launching the CLI client process. | `src/cli/performance_report/changed_path_cli.rs`; `benches/history/current.json`. | Shows daemon/socket validation is roughly 0.15-0.44 ms on realistic fixtures; the one-shot warm client miss is dominated by the client process boundary. |
| Persistent session CLI | Added `assura-check-session` and `assura-check-dirty-project-session-cli`, a long-lived binary CLI process that accepts repeated stdin commands and forwards dirty-path checks to the daemon. | `crates/assura-check-cli/src/session.rs`; `src/cli/performance_report/session_cli.rs`; `benches/history/current.json`. | Warm/editor-session gate clears 6 / 6 realistic fixtures; later Linux static-CRT work completed the cold subprocess gate too. |
| Cold syscall profiling attempt | Tried to profile the remaining one-shot startup gap with `dtruss` against a minimal temp project. | `target/performance/cold-start-dtruss-blocked.txt`. | Blocked by macOS DTrace privileges/SIP in this environment. A deeper cold-start syscall profile needs a host/session where DTrace/Instruments access is available. |
| Fast count child-name borrowing | Removed an extra `String` allocation per fast-count child entry by borrowing from `OsString` at validation time. | `src/cli/check/ls_fast_counts.rs`; `src/cli/check/ls_fast.rs`; `target/performance/fast-count-name-borrow-smoke.json`. | Retained as a small hot-path cleanup, but not counted as completion evidence. The 3-iteration smoke kept cold `claim_summary` at `not-complete` and warm session at `complete`; tracked history was not refreshed. |
| Per-request warm config freshness | Made `assura-checkd` probe the prepared config fingerprint on every request, so cached/session checks reload when config changes even if a file watcher misses or cannot see the config path. | `crates/assura-check-cli/src/server.rs`; `crates/assura-check-cli/tests/hot_cli.rs`; `target/performance/session-config-fingerprint-smoke.json`. | Retained as warm-path correctness hardening. The 3-iteration smoke kept warm session complete at 6 / 6 and about 32.72x aggregate speedup; cold remained `not-complete`. |
| Persistent daemon session connection | Added an explicit `SESSION` daemon handshake so `assura-check-session` keeps one daemon socket open across repeated stdin commands while one-shot clients keep their existing one-request connection model. | `crates/assura-check-cli/src/server.rs`; `crates/assura-check-cli/src/server_io.rs`; `crates/assura-check-cli/src/session.rs`; `benches/history/current.json`. | Retained as warm-path latency improvement. The warm session report remains complete at 6 / 6; later Linux static-CRT work completed the cold gate too. |
| Cold profiler access check | Tried `dtruss` and `sample -wait` against a temp simple-library-style fixture to identify a new startup hypothesis. | `target/performance/cold-start-profiler-blockers-2026-05-19.txt`. | Blocked by local macOS sandbox/SIP permissions. No new cold implementation hypothesis was identified; a real syscall/profile pass needs a less restricted host/session. |
| Relative cwd fast-path screen | Considered avoiding `std::env::current_dir()` for the common no-path `assura-check --quiet` benchmark invocation. | `benches/history/current.json` phase rows. | Rejected as too small for the cold gap: config discovery is only about 0.03-0.04 ms on the failing realistic rows, while the remaining cold gap needs about 1.0-1.7 ms across multiple rows. |
| Linux scratch host check | Synced the dirty worktree to `vps-gw` scratch space and ran a 3-iteration release performance report. | `target/performance/vps-gw-current-scratch.json`. | Linux lowered process floors and cold aggregate exceeded 2x, but the universal cold gate still failed: `assura-check-cli` passed 2 / 6 fixtures and `claim_summary.two_x_claim_verdict` remained `not-complete`. Warm session stayed complete at 6 / 6. |
| Linux static-CRT release screen | Built `assura-check-cli` on `vps-gw` with static CRT flags to remove dynamic loader work seen in `strace`, then ran 3- and 5-iteration diagnostic reports. | `.cargo/config.toml`; `target/performance/vps-gw-static-crt-5iter-profiled-git.json`; `target/performance/vps-gw-strace-static-assura-check.txt`. | Retained as the first cold CLI completion path on Linux. The 5-iteration static-CRT report completed the cold gate: 6 / 6 fixtures, 2.90x aggregate, `claim_summary.two_x_claim_verdict=complete`, and `assura_binary_profile=release-static-crt`. This is Linux static-CRT evidence, not the default macOS dynamic report. |

## Gap Compression Snapshot

The important progress is not binary size. It is the movement from an invalid
comparison, to honest cold CLI attribution, to a completed warm/editor-session
contract.

| Checkpoint | Headline row | What changed | Outcome |
| --- | --- | --- | --- |
| Invalid initial claim | LS-Lint through package wrapper | Timed the Node/package wrapper instead of the native LS-Lint binary. | Rejected; the apparent ~100 ms LS-Lint floor was not a fair product comparison. |
| Fair native comparison | Full `assura check` versus native LS-Lint | Resolved the packaged Go binary from pinned `@ls-lint/ls-lint@2.3.0`. | Exposed the real target: native LS-Lint is fast enough that full Assura startup could not compete. |
| Dedicated cold check CLI | `assura-check-cli` | Split the focused validation path from the full product CLI and removed unrelated feature surfaces. | Moved the product path from full-CLI overhead toward a focused check artifact; the current tracked Linux static-CRT row clears 6 / 6 at 2.90x aggregate. |
| Engine attribution | `assura-in-process` | Measured validation without subprocess startup. | Engine clears 6 / 6 2x targets, proving the remaining gap is startup/execution model, not validation throughput. |
| One-shot warm client | `assura-check-dirty-project-cli` | Kept config/rule planning in a daemon but launched a client process for each check. | Useful attribution, but still misses small fixtures because client process startup dominates. |
| Persistent warm session | `assura-check-dirty-project-session-cli` | Kept one CLI process and one daemon socket alive across repeated commands. | Current tracked warm row clears 6 / 6 2x targets with 40.16x aggregate speedup. |

This is the practical gap history: cold went from invalid comparison, to
floor-limited dynamic builds, to a completed Linux static-CRT release artifact;
warm went from diagnostic evidence to a complete persistent-session result.
Future performance work should preserve these scoped claims or explicitly
define a new claim scope.

## Current Warm-Path Gap

The warm/editor-session path is the most promising remaining direction for the
agentic-development workload because it keeps config and rule planning hot and
can validate known dirty files without a full project traversal.

Current tracked dirty-project rows:

| Fixture | Dirty-project CLI | Native LS-Lint | 2x target | Result |
| --- | ---: | ---: | ---: | --- |
| `simple_library` | 3.08 ms | 5.48 ms | 2.74 ms | Miss |
| `web_app` | 3.15 ms | 5.09 ms | 2.55 ms | Miss |
| `monorepo_packages` | 3.76 ms | 5.28 ms | 2.64 ms | Miss |
| `monorepo_policy` | 3.70 ms | 8.85 ms | 4.43 ms | Pass |
| `rule_heavy_repo` | 2.97 ms | 6.42 ms | 3.21 ms | Pass |
| `ignored_generated_heavy_repo` | 3.06 ms | 10.19 ms | 5.10 ms | Pass |

Current persistent dirty-project session CLI rows:

| Fixture | Dirty-project session CLI | 2x target |
| --- | ---: | ---: |
| `simple_library` | 0.19 ms | 2.74 ms |
| `web_app` | 0.11 ms | 2.55 ms |
| `monorepo_packages` | 0.23 ms | 2.64 ms |
| `monorepo_policy` | 0.27 ms | 4.43 ms |
| `rule_heavy_repo` | 0.11 ms | 3.21 ms |
| `ignored_generated_heavy_repo` | 0.12 ms | 5.10 ms |

Current direct daemon/socket diagnostic rows:

| Fixture | Dirty-project socket | 2x target |
| --- | ---: | ---: |
| `simple_library` | 0.21 ms | 2.74 ms |
| `web_app` | 0.13 ms | 2.55 ms |
| `monorepo_packages` | 0.23 ms | 2.64 ms |
| `monorepo_policy` | 0.26 ms | 4.43 ms |
| `rule_heavy_repo` | 0.13 ms | 3.21 ms |
| `ignored_generated_heavy_repo` | 0.08 ms | 5.10 ms |

Immediate warm-path conclusion:

- The one-shot client still misses the small rows, so it should not be the
  warm product claim.
- The persistent session CLI and direct socket rows both clear every target
  with large headroom. The current warm claim should be framed as an
  editor-session/prepared-config execution contract, not as a one-shot cold
  subprocess claim.

## Retained Improvements

Retained implementation improvements:

- Native LS-Lint binary comparison from the pinned npm package.
- Dedicated check-only CLI using a lightweight Rust CLI parser.
- Feature-gated check-only dependency surface.
- LS-Lint-compatible fast validation path.
- Exit-status subprocess timing with null stdout/stderr.
- Process-floor and Rust CLI floor attribution.
- Dedicated no-op Assura Rust CLI floor row for cleaner startup attribution.
- Separate warm dirty-project claim summary for editor-session evidence.
- Dirty-path de-duplication for watcher plus explicit editor path overlap.
- Compiled config artifacts using `postcard`.
- Source config fingerprints with hash fallback.
- Prepared checks that reload config only when content changes.
- Opt-in cached checks that can avoid config reads when fingerprints match.
- Hot daemon, dirty-path, and status-file diagnostic modes.
- Low-sample and diagnostic-row completion guards.
- Borrowed fast-count child names instead of allocating a cached string per
  child entry. This is a narrow cleanup, not a new performance claim.
- Per-request config fingerprint freshness in the daemon/session path, so
  unchanged config remains warm but changed config cannot be hidden by watcher
  delay or an external config path outside the watched tree.
- Persistent daemon socket reuse for `assura-check-session` through an explicit
  `SESSION` handshake, avoiding per-command reconnects without changing
  one-shot client behavior.

Retained documentation:

- Completion audit:
  `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`
- Scope decision:
  `docs/analysis/2026-05-19-ls-lint-performance-scope-decision.md`
- Active goal log:
  `docs/goals/assura-native-ls-lint-performance-rearchitecture.md`

## Rejected Or Exhausted Experiment Areas

The completion audit contains the detailed table with evidence paths. This
section is the quick "do not start here again" list.

Rejected as cold universal 2x completion paths:

- Node-wrapper LS-Lint comparison.
- Release `opt-level=z`.
- Minimal `assura-check-quiet` binaries.
- Raw Unix entrypoints for the ordinary check binary.
- Exact `--quiet` pre-parsers.
- Removing JSON/YAML report output and cache support from the default check
  binary.
- Default automatic compiled-artifact probing from ordinary `assura-check`.
- Smaller/no-YAML compiled-artifact package splits.
- Direct simple-fast artifact runners that duplicate fast validation logic.
- `serde_norway`, `serde-yml`, and `serde-saphyr` parser swaps.
- Lazy filename/file-stem changes that regressed cold rows.
- Dot-suffix lookup-map rule-heavy experiments.
- Status-file marker files and symlink-backed status files.
- Compact hot-client protocol changes as cold completion evidence.
- Single-write raw Unix client request assembly.
- Reusing one prepared `StructureChecker` across daemon changed-path requests.
- Linux host selection as a universal cold completion strategy.
- External larger fixtures as a universal generated-fixture claim.

Important nuance:

- Some rejected experiments reduced binary size. They were still rejected
  because they did not reduce measured end-to-end runtime enough.
- Some retained warm-path features are valuable product architecture. They are
  not completion evidence for the current cold `assura-check-cli` row.

## Current Lower-Bound View

The cold path appears near a practical lower bound on this machine for small
fixtures under the current benchmark shape:

| Fixture | `assura-check-cli` | Process floor | Rust CLI floor | 2x target |
| --- | ---: | ---: | ---: | ---: |
| `simple_library` | 3.58 ms | 1.99 ms | 2.54 ms | 2.74 ms |
| `web_app` | 4.05 ms | 2.23 ms | 2.78 ms | 2.55 ms |
| `monorepo_packages` | 4.47 ms | 2.39 ms | 2.90 ms | 2.64 ms |
| `monorepo_policy` | 5.60 ms | 2.40 ms | 2.91 ms | 4.43 ms |
| `rule_heavy_repo` | 4.74 ms | 2.22 ms | 2.93 ms | 3.21 ms |
| `ignored_generated_heavy_repo` | 3.71 ms | 2.22 ms | 2.58 ms | 5.10 ms |

The in-process engine has enough headroom:

| Fixture | In-process Assura | 2x target |
| --- | ---: | ---: |
| `simple_library` | 0.21 ms | 2.74 ms |
| `web_app` | 0.21 ms | 2.55 ms |
| `monorepo_packages` | 0.43 ms | 2.64 ms |
| `monorepo_policy` | 1.44 ms | 4.43 ms |
| `rule_heavy_repo` | 0.81 ms | 3.21 ms |
| `ignored_generated_heavy_repo` | 0.13 ms | 5.10 ms |

Working conclusion:

```text
Further cold CLI work is only justified with a new startup hypothesis that can
realistically remove about 1.0-1.7 ms from multiple small-fixture rows.
Otherwise, focus on the warm/editor-session path.
```

This is the current stop line for cold work. A new cold experiment should be
accepted only if it is backed by syscall/profile evidence or a clearly different
execution contract. Parser reshaping, report stripping, binary-size trimming,
and additional default artifact probes are no longer reasonable starting points.

## Cold-Path Closure Criteria

Before spending another implementation slice on cold subprocess speed, require
all of the following:

1. The hypothesis names a measurable source of startup or per-run work not
   already covered by the rejected experiment list.
2. The expected win is large enough to matter: about 1.0-1.7 ms across multiple
   small realistic-equivalent rows, not a noise-level improvement on one
   fixture.
3. The measurement can distinguish process creation, dynamic loader work,
   filesystem syscalls, parser/config work, and validation work.
4. The change preserves the public check behavior and does not trade away
   structured output, cache support, or config correctness for an unproven
   microbenchmark.
5. A 3-iteration screening report improves the cold headline row enough to
   justify a 5-iteration tracked refresh.

If those gates are not met, treat the cold path as lower-bound constrained for
the current execution model and move effort to the warm/editor-session path.
That is the workload that matters most for agentic development because agents
will repeatedly hit the validator during an editing session.

Attribution note:

`target/performance/noop-rust-floor-smoke-2.json` first added an
`assura-rust-cli-floor` diagnostic row using a minimal Assura-built no-op
binary. The tracked 5-iteration report now includes that row and uses it as
the cleaner Rust CLI floor input instead of the status-file product diagnostic.
The checked-in current report now uses the Linux static-CRT artifact and reports
`claim_summary.two_x_claim_verdict = complete`. The older macOS dynamic report
remains useful as lower-bound evidence only.

## Research Areas Still Worth Considering

Cold path research that may still be worth a bounded spike:

1. Platform-level startup profiling
   - Use `dtruss`, Instruments, or `sample` on macOS to identify whether the
     remaining 1.0-1.7 ms is dynamic loader, page faults, filesystem syscalls,
     panic/runtime setup, or code execution.
   - A local `dtruss` attempt was blocked by DTrace privileges/SIP; see
     `target/performance/cold-start-dtruss-blocked.txt`.
   - A later local `dtruss` plus `sample -wait` attempt was also blocked by
     sandbox/SIP/sysctl permissions; see
     `target/performance/cold-start-profiler-blockers-2026-05-19.txt`.
   - Stop if the result is mostly process creation/dynamic-loader floor.

2. Linux-first production target validation
   - Local macOS rows are useful for development, but agentic workloads may run
     mostly on Linux servers.
   - Existing Linux smoke improved aggregate performance but did not complete
     the universal cold gate. A future Linux run should be used for deployment
     planning, not to hide macOS cold limitations.
   - A `vps-gw` scratch refresh at
     `target/performance/vps-gw-current-scratch.json` again left the universal
     cold gate incomplete: aggregate speedup was 2.26x, but only 2 / 6
     realistic-equivalent rows met the per-fixture 2x target.
   - A follow-up Linux static-CRT build using
     `cargo build-assura-check-linux-static` completed the same gate in a
     5-iteration scratch report: 6 / 6 rows, 2.90x aggregate,
     `assura_binary_profile=release-static-crt`, and
     `claim_summary.two_x_claim_verdict=complete`. This is the current
     productization candidate for Linux release artifacts.

3. AOT/precompiled project state beyond config
   - Current compiled artifacts compile config, not directory state.
   - A project-state index could help warm paths, but any cold path that has to
     validate index freshness may pay enough filesystem checks to erase wins.

4. Alternative process contract
   - Persistent service, Unix socket, status file, or editor integration can
     avoid repeated cold startup.
   - This is the most promising direction for agentic development loops.

5. Workload-scope benchmark
   - If agents run many validations per task, a repeated-check or batch-check
     contract may be more representative than single cold subprocess checks.
   - Must be documented as amortized or warm, not as cold CLI evidence.

Warm path research that is still worth considering:

1. Dirty-path client floor profiling
   - Profile `assura-check-client --dirty-project-path` separately from daemon
     validation to determine whether the remaining warm miss is process launch,
     socket IO, request parsing, response handling, or changed-path validation.

2. Editor/native integration without a subprocess per check
   - For agentic loops that can call a daemon API directly, measure the daemon
     request itself rather than a CLI client process. This would be a different
     product contract and must be labeled accordingly.

3. Project-state index for aggregate rules
   - Maintain enough file-set state in the daemon to know when a dirty file can
     be checked directly and when parent directory count/exists rules require a
     broader refresh.

4. Config artifact lifecycle
   - Keep compiled/prepared config fingerprints as the boundary for skipping
     validation. Revalidate config only after content, path, version, schema, or
     feature changes.

5. Amortized agent workflow benchmark
   - Measure a representative sequence of repeated checks during an edit loop:
     daemon start, initial full check, repeated dirty-file checks, config edit,
     and required full fallback. Use this to decide product messaging for
     agentic development.

6. Config-dirty and file-dirty invalidation proof
   - Expand tests around config fingerprints, config edits, dirty files, parent
     aggregate rules, and fallback-to-full traversal. This is the correctness
     side of making warm validation the default agentic contract.

Research areas that should not be reopened without a materially different
design:

- Parser swaps or hand-rolled argument parsing.
- Binary-size-only reductions.
- More report-format stripping.
- More default compiled-artifact probing in ordinary `assura-check`.
- Another status-file encoding change without a syscall-level profile proving
  it helps.
- Another suffix-match lookup structure for the current rule-heavy fixture.
- Another raw Unix client request encoding change unless the profile proves
  write syscalls, rather than process launch or daemon validation, are the
  remaining bottleneck.
- Another prepared-check cache/reuse change unless the measured warm summary
  improves, not just one fixture in a short smoke.
- A relative-current-directory fast path for no-argument `assura-check --quiet`
  unless fresh phase data contradicts the current report; config discovery is
  currently far below the size of the cold miss.

## Recommended Next Goal

Productize and harden the warm-path session contract with explicit completion
criteria.

Suggested objective:

```text
Document, test, and package the editor-session/prepared-config validation path
where unchanged config does not reparse/revalidate, dirty file checks avoid
whole-project traversal when safe, and repeated CLI-facing checks remain at
least 2x faster than native LS-Lint under the persistent session contract.
```

Suggested completion evidence:

- `warm_claim_summary` for the `assura-check-dirty-project-session-cli` row
  family, not reuse of the cold `assura-check-cli` verdict.
- At least three measured iterations.
- Explicit row families for one-shot CLI, persistent session CLI, and direct
  daemon socket attribution.
- Correctness tests for config-change invalidation, dirty-path invalidation,
  and fallback to full project checks when the dirty set is unsafe.
- Website language that says "warm/editor-session" or "prepared config"
  wherever a 2x claim is made.

Keep the cold `claim_summary.two_x_claim_verdict` tied to the Linux static-CRT
release profile unless another platform-specific release artifact has its own
measured completion evidence.
