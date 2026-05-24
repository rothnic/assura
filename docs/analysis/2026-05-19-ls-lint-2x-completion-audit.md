---
title: LS-Lint 2x Completion Audit
date: 2026-05-19
status: complete-linux-static
---

# LS-Lint 2x Completion Audit

## Objective

Deliver a well-architected Rust CLI validation path that uses an established
Rust CLI framework where appropriate and is at least 2x faster than native
LS-Lint for comparable binary CLI execution.

## Current Verdict

The objective is complete for the Linux static-CRT release artifact path.
The default local macOS dynamic build remains incomplete and is retained as a
diagnostic lower-bound record, not as the current release claim.

The implementation now provides an honest native-binary comparison, a
check-focused `assura-check` binary, LS-Lint-compatible fast validation,
compiled-config and daemon-backed diagnostic modes, and cold-start feasibility
attribution. The current tracked report also emits a top-level
`claim_summary` verdict so the public headline gate cannot be inferred from
diagnostic rows. That verdict is now `complete` for the Linux static-CRT
`assura-check-cli` release artifact.

Current tracked pass counts from `benches/history/current.json`:

| Row | 2x pass count | Completion status |
| --- | ---: | --- |
| `assura-check-cli` | 6 / 6 | Complete for Linux static-CRT release |
| `assura-check-compiled-cli` | 1 / 6 | Not complete |
| `assura-check-hot-cli` | 3 / 6 | Diagnostic only |
| `assura-check-dirty-project-cli` | 3 / 6 | One-shot warm client; diagnostic |
| `assura-check-dirty-project-session-cli` | 6 / 6 | Warm summary complete under persistent session contract |
| `assura-check-dirty-project-socket` | 6 / 6 | Diagnostic only; not CLI subprocess evidence |
| `assura-check-status-cli` | 2 / 6 | Diagnostic only |
| `assura-in-process` | 6 / 6 | Not CLI subprocess evidence |

The top-level `claim_summary` in `benches/history/current.json` now reports
`measured_iterations=5`, `sufficient_completion_iterations=true`,
`assura_faster_count=6`, `two_x_pass_count=6`, `two_x_fail_count=0`, and
`two_x_claim_verdict=complete` for the six measured headline fixtures.
The sibling warm gate reports
`warm_claim_summary.two_x_claim_verdict=complete`,
`warm_claim_summary.assura_row_family=assura-check-dirty-project-session-cli`,
`warm_claim_summary.two_x_pass_count=6`, and
`warm_claim_summary.aggregate_speedup_ratio=25.213361575198398`.

Earlier macOS/dynamic and ordinary Linux smokes remain useful as lower-bound
and rejected-experiment evidence, but they do not override the checked-in
Linux static-CRT release report. The important sequence is:

- local macOS dynamic cold rows repeatedly exposed a subprocess/startup floor;
- ordinary Linux release rows improved aggregate speed but did not clear every
  small generated fixture;
- the Linux static-CRT release artifact removed enough dynamic-loader overhead
  to clear the cold 2x gate across all six headline fixtures;
- the persistent warm/editor-session path remains a separate completed gate.

The retained lower-bound decision is still not "make the binary smaller."
Binary size is secondary evidence only. Keep a size reduction only when it
improves measured runtime or removes startup/hot-path work without regression.

Earlier screening smokes are retained below to explain rejected paths and to
prevent future agents from repeating them:

| Evidence | Headline verdict | Notes |
| --- | --- | --- |
| `target/performance/post-incremental-split-smoke.json` | `not-complete`; 2 / 6 2x passes; aggregate 1.57x | Module splits and changed-path correctness did not improve the cold headline gate. |
| `target/performance/hot-client-no-close-smoke.json` | `not-complete`; 1 / 6 2x passes; aggregate 1.82x | Hot/status diagnostic rows improved enough to remain useful, but cold `assura-check-cli` still missed 5 / 6 rows. |
| `target/performance/explicit-dirty-project-row-smoke.json` | `not-complete`; 1 / 6 cold headline 2x passes; aggregate 1.62x | Deterministic `--dirty-project-path` project-status row emitted pass measurements for all fixtures, but remains diagnostic and met 2x on only 3 / 6 realistic-equivalent fixtures. |
| `target/performance/compact-dirty-protocol-v3-smoke.json` | `not-complete`; 1 / 6 cold headline 2x passes; aggregate 1.56x | Compact raw Unix dirty-project request/response kept the editor-session row deterministic, but the diagnostic row still met 2x on only 3 / 6 realistic-equivalent fixtures. |
| `target/performance/status-ignore-retained-smoke.json` | `not-complete`; 1 / 6 cold headline 2x passes; aggregate 1.48x | Ignoring daemon status-file self-writes is useful product hardening, but the retained status diagnostic row still did not provide a universal 2x completion path. |
| `target/performance/vps-gw-smoke.json` | `not-complete`; 3 / 6 cold headline 2x passes; aggregate 2.38x | Ordinary Linux process floor was lower than macOS, but this run still missed three small fixtures before the static-CRT build. |
| `target/performance/default-git-signals-opt-in-smoke.json` | `not-complete`; 2 / 6 cold headline 2x passes; aggregate 1.78x | Default full-CLI builds no longer pull `git2`/OpenSSL, but the cold headline still misses four realistic-equivalent fixtures. |
| `target/performance/current-state-smoke.json` | `not-complete`; 2 / 6 cold headline 2x passes; aggregate 1.80x | Fresh ordinary release-artifact smoke after the dep-info freshness fix measured all six headline rows, remained faster than native LS-Lint on all six, and still missed the universal 2x gate. |
| `target/performance/external-fixtures-complete-smoke.json` | `not-complete`; 3 / 8 cold headline 2x passes; aggregate 4.46x | Opt-in pinned Next.js and mdBook fixtures now materialize and measure successfully, but the universal claim still fails because five generated realistic-equivalent rows remain below 2x. |
| `target/performance/compiled-fingerprint-current-smoke.json` | `not-complete`; 2 / 6 cold headline 2x passes; aggregate 1.46x | Compiled-config source fingerprinting improves the explicit precompiled diagnostic contract, but the ordinary cold `assura-check-cli` headline remains below the universal 2x gate. |
| `target/performance/noop-rust-floor-smoke-2.json` | `not-complete`; 1 / 6 cold headline 2x passes; aggregate 1.59x | Added a dedicated no-op Assura Rust CLI floor row so startup attribution no longer depends on the status-file product diagnostic. The new row is diagnostic evidence only and does not make the cold headline complete. |
| `target/performance/warm-claim-summary-smoke.json` | cold `not-complete`; warm `not-complete` | Added `warm_claim_summary` for `assura-check-dirty-project-cli`. The smoke showed warm aggregate speedup near 1.95x with 2 / 6 fixtures meeting 2x, so this is useful tracking but not a completed warm claim. |
| `target/performance/dirty-path-dedupe-smoke.json` | cold `not-complete`; warm one-shot incomplete | De-duplicating watcher and explicit dirty paths improved the warm dirty-project smoke to 5 / 6 2x passes, with the remaining miss blocked by that run's process floor. Later one-shot client refreshes remained incomplete, which motivated the persistent session row. |
| `target/performance/unix-client-single-write-smoke.json` | cold `not-complete`; warm `not-complete` | Rejected assembling the raw Unix dirty-path request into one stack-buffer write. The 3-iteration smoke regressed the warm dirty-project row to 3 / 6 2x passes and 2.16x aggregate speedup, so the code was reverted. |
| `target/performance/prepared-reuse-checker-smoke-2.json` plus a tracked 5-iteration refresh | cold `not-complete`; warm `not-complete` | Rejected reusing one prepared `StructureChecker` and its rule cache across daemon changed-path requests. A short smoke improved `monorepo_packages`, but repeated/tracked evidence regressed the warm summary below the prior tracked state, so the code was reverted. |
| `target/performance/dirty-project-socket-profile-smoke.json`; `benches/history/current.json` | cold `not-complete`; one-shot warm CLI incomplete; socket diagnostic clears target | Added `assura-check-dirty-project-socket` to isolate daemon/socket validation from the client process. The tracked socket row is about 0.15-0.44 ms on realistic fixtures, far below every 2x target, while the one-shot CLI-client warm row remains incomplete. |
| `target/performance/session-warm-summary-smoke.json`; `benches/history/current.json` | cold `not-complete`; warm session `complete` | Added persistent `assura-check-session` and the `assura-check-dirty-project-session-cli` row. The tracked session row clears 6 / 6 realistic fixtures at about 0.11-0.27 ms and 40.16x aggregate speedup, while the cold subprocess headline remains incomplete. |
| `target/performance/fast-count-name-borrow-smoke.json` | cold `not-complete`; warm session `complete` | Removed an extra `String` allocation per fast-count child entry and borrowed names during count validation. This is retained as a small cleanup, but the 3-iteration smoke was not strong enough to refresh tracked history or change the cold lower-bound decision. |
| `target/performance/session-config-fingerprint-smoke.json` | cold `not-complete`; warm session `complete` | Added per-request config fingerprint probing in `assura-checkd`, so cached/session checks reload changed config even without a watcher event. The smoke kept the warm session row at 6 / 6 with about 32.72x aggregate speedup. |
| `target/performance/session-persistent-daemon-connection-smoke.json`; `benches/history/current.json` | cold `not-complete`; warm session `complete` | Added a `SESSION` daemon handshake so `assura-check-session` reuses one daemon socket across stdin commands. The tracked report keeps the warm session row at 6 / 6 with about 40.16x aggregate speedup. |
| `target/performance/cold-start-profiler-blockers-2026-05-19.txt` | no cold verdict change | Tried to collect a new cold-start hypothesis with `dtruss` and `sample -wait` against a temp simple-library-style fixture. Both profiler paths were blocked by local macOS sandbox/SIP/sysctl permissions, so no new cold implementation path was proven. |
| `benches/history/current.json` phase rows | no cold verdict change | Screened a relative-current-directory fast path for no-argument `assura-check --quiet`. Current config-discovery phase medians are only about 0.03-0.04 ms on the failing realistic rows, far below the 1.0-1.7 ms needed across multiple small rows, so this is not a credible cold completion path. |
| `target/performance/vps-gw-current-scratch.json` | `not-complete`; 2 / 6 cold headline 2x passes; aggregate 2.26x | Synced the dirty worktree to a separate `vps-gw` scratch directory and ran a 3-iteration Linux release report. Lower Linux process floors improved aggregate speed, but the universal per-fixture cold gate still failed. Warm session stayed complete at 6 / 6 with 26.55x aggregate speedup. |
| `target/performance/vps-gw-static-crt-5iter-profiled-git.json` | `complete`; 6 / 6 cold headline 2x passes; aggregate 2.90x | Built the check-only package with the repo alias `cargo build-assura-check-linux-static`, which applies Linux static-CRT flags through Cargo config. The report labels the measured binaries as `release-static-crt`. This removes dynamic loader work seen in `strace` and is the first measured cold CLI completion path. Scope is Linux static-CRT release artifacts, not the default local macOS dynamic report. |

## Prompt-To-Artifact Checklist

| Requirement | Evidence | Status |
| --- | --- | --- |
| Compare to LS-Lint binary, not Node wrapper | `src/cli/performance_report/ls_lint.rs` resolves `node_modules/@ls-lint/ls-lint/bin/ls-lint-<platform>` and rows record `ls_lint_execution_mode=native-binary-from-pinned-npm-package`. | Satisfied |
| Use established Rust CLI framework where useful | `assura-check` uses the small `pico-args` parser for its hot path; companion tools use `lexopt` where the option surface is richer. Rejected parser and entrypoint experiments are logged in `docs/goals/assura-native-ls-lint-performance-rearchitecture.md`. | Satisfied |
| Exclude unrelated full-product startup surfaces | `crates/assura-check-cli` depends on `assura` with `default-features = false`; full CLI, markdown, graph, watch, and jwalk surfaces are feature-gated away from the default check binary. The root `assura` default feature set also keeps `git-signals` opt-in so default full-CLI builds do not pull `git2`/OpenSSL. | Satisfied |
| Provide reproducible Linux static-CRT release build | `.cargo/config.toml` defines `cargo build-assura-check-linux-static`, a Cargo alias that builds `assura-check-cli` for `x86_64-unknown-linux-gnu` with static CRT flags and `-lgcc_eh`. Verified on `vps-gw`; resulting binaries are static-pie and performance rows report `assura_binary_profile=release-static-crt`. | Satisfied for Linux release artifacts |
| Preserve validation correctness | `cargo test --all-targets --quiet`, `cargo clippy --all-targets --all-features -- -D warnings`, `target/release/assura-check --quiet .`, and `git diff --check` pass after the performance branch changes. | Satisfied |
| Comparable CLI execution is measured end to end | `assura-check-cli` and `ls-lint-native-cli` rows execute release binaries from fixture working directories with stdout/stderr sent to null and exit status checked. | Satisfied |
| 2x faster than native LS-Lint on all realistic rows | `claim_summary.two_x_claim_verdict=complete`, `two_x_pass_count=6`, and `assura_binary_profile=release-static-crt` in `benches/history/current.json`. | Satisfied for Linux static-CRT release artifacts |
| Diagnose whether the miss is validation or process overhead | Phase rows show in-process validation below target; process-floor and no-op Assura Rust CLI floor fields show small-row subprocess overhead dominates. | Satisfied |
| Do not claim completion from diagnostic/proxy signals | Daemon/status and in-process rows are tracked but not accepted as completion evidence for cold `assura-check-cli`. | Satisfied |
| Publish a machine-readable headline verdict | `benches/history/current.json` and `website/public/data/performance/current.json` include `claim_summary.two_x_claim_verdict=complete` for Linux static-CRT release evidence. | Satisfied |
| Measure deterministic editor-session project status separately | `assura-check-dirty-project-cli` measures the one-shot client, `assura-check-dirty-project-socket` isolates daemon/socket work, and `assura-check-dirty-project-session-cli` measures a persistent CLI session over stdin/stdout. `warm_claim_summary` now summarizes the persistent session row independently from the cold headline gate. | Satisfied |

## Live Audit Commands

The latest audit against the checked-in report used these direct report queries:

```bash
jq -r '.claim_summary, .warm_claim_summary' benches/history/current.json
jq -r '.results[] | select(.tool_name=="assura-check-cli") | ...' benches/history/current.json
jq -r '.results[] | select(.tool_name=="ls-lint-native-cli") | ...' benches/history/current.json
jq -r '.results[] | select(.tool_name=="assura-in-process") | ...' benches/history/current.json
```

The current cold headline summary remains:

```text
assura_row_family = assura-check-cli
ls_lint_row_family = ls-lint-cli
assura_faster_count = 6
two_x_pass_count = 6
two_x_fail_count = 0
aggregate_speedup_ratio = 2.8980855186211874
assura_binary_profile = release-static-crt
two_x_claim_verdict = complete
```

The current warm sibling summary remains:

```text
assura_row_family = assura-check-dirty-project-session-cli
two_x_pass_count = 6
two_x_fail_count = 0
aggregate_speedup_ratio = 25.213361575198398
two_x_claim_verdict = complete
```

Native LS-Lint evidence comes from the measured `ls-lint-native-cli` tool rows,
which record `ls_lint_execution_mode=native-binary-from-pinned-npm-package`.
The claim summary uses the normalized row family `ls-lint-cli` for this native
binary baseline. This distinction matters because the historical Node wrapper
comparison is not accepted as evidence.

The default local macOS audit outcome is unchanged: the cold comparable binary
CLI objective is not complete there. The checked-in current report now uses the
Linux static-CRT release artifact, where the cold objective is complete. The
persistent warm/editor-session contract is also complete under its separate
`warm_claim_summary`.

## Current Tracked Rows

| Fixture | `assura-check-cli` | Native LS-Lint | 2x target | Result |
| --- | ---: | ---: | ---: | --- |
| `simple_library` | 0.79 ms | 1.88 ms | 0.94 ms | Pass |
| `web_app` | 0.72 ms | 1.79 ms | 0.90 ms | Pass |
| `monorepo_packages` | 0.89 ms | 2.13 ms | 1.07 ms | Pass |
| `monorepo_policy` | 1.62 ms | 3.64 ms | 1.82 ms | Pass |
| `rule_heavy_repo` | 1.08 ms | 2.98 ms | 1.49 ms | Pass |
| `ignored_generated_heavy_repo` | 0.79 ms | 4.63 ms | 2.32 ms | Pass |

## Floor Attribution

| Fixture | Process floor | Assura Rust CLI floor | `assura-check-cli` | 2x target |
| --- | ---: | ---: | ---: | ---: |
| `simple_library` | 0.56 ms | 0.41 ms | 0.79 ms | 0.94 ms |
| `web_app` | 0.55 ms | 0.38 ms | 0.72 ms | 0.90 ms |
| `monorepo_packages` | 0.57 ms | 0.40 ms | 0.89 ms | 1.07 ms |
| `monorepo_policy` | 0.56 ms | 0.35 ms | 1.62 ms | 1.82 ms |
| `rule_heavy_repo` | 0.69 ms | 0.44 ms | 1.08 ms | 1.49 ms |
| `ignored_generated_heavy_repo` | 0.60 ms | 0.43 ms | 0.79 ms | 2.32 ms |

The Linux static-CRT release artifact keeps the measured Rust CLI floor below
every 2x target in the current report. The earlier dynamic macOS rows remain
useful lower-bound evidence, but they are no longer the checked-in headline
claim.

Earlier local macOS and ordinary Linux smokes remain in the table above as
diagnostic history. They explain why the work moved away from another
unscoped cold-start micro-optimization and toward a scoped Linux static-CRT
release claim plus a separate warm/editor-session contract.

## Engine Attribution

| Fixture | Assura in-process | 2x target | Result |
| --- | ---: | ---: | --- |
| `simple_library` | 0.21 ms | 2.74 ms | Pass |
| `web_app` | 0.21 ms | 2.55 ms | Pass |
| `monorepo_packages` | 0.43 ms | 2.64 ms | Pass |
| `monorepo_policy` | 1.44 ms | 4.43 ms | Pass |
| `rule_heavy_repo` | 0.81 ms | 3.21 ms | Pass |
| `ignored_generated_heavy_repo` | 0.13 ms | 5.10 ms | Pass |

This confirms the remaining miss is dominated by CLI process/startup overhead,
not validation throughput.

## Latest Rejected Experiments

Additional changes were tested on 2026-05-19 and rejected as completion paths
because none improved the cold headline row enough to satisfy the universal 2x
gate:

| Experiment | Evidence | Result |
| --- | --- | --- |
| `CARGO_PROFILE_RELEASE_OPT_LEVEL=z` for `assura-check-cli` | `target/performance/check-cli-opt-z-smoke.json` | Binary size improved, but headline aggregate fell to 1.65x and stayed 1 / 6 on the 2x gate. |
| Minimal `assura-check-quiet` binary | `target/performance/check-quiet-smoke.json` | Smaller binary, but not faster than the retained `assura-check` row across the headline set. |
| Unix raw entrypoint for `assura-check` | `target/performance/raw-main-15-smoke.json` | Kept the same parser and validation engine, but still reported 1 / 6 on the 2x gate with a 1.66x aggregate. |
| Exact `assura-check --quiet` fast parser | `target/performance/quiet-fast-parse-smoke.json` | Bypassed `pico-args` for the headline invocation, but regressed aggregate speedup to 1.53x and still passed only 1 / 6. |
| Default build without JSON/cache support | `target/performance/no-json-check-smoke.json` | Shrunk `assura-check` to about 940 KB, but regressed aggregate speedup to 1.64x and broke the cached diagnostic row in default release builds. |
| Lazy file-stem computation in the fast validator | `target/performance/lazy-stem-15-smoke.json` | Improved some walk-phase samples, but regressed cold CLI aggregate to 1.59x and still passed only 1 / 6. |
| Module-boundary cleanup and incremental changed-path validation | `target/performance/post-incremental-split-smoke.json` | Correctness and maintainability improved; cold headline stayed `not-complete` with 2 / 6 2x passes. |
| Removing explicit `close(2)` from the raw Unix hot client | `target/performance/hot-client-no-close-smoke.json` | Kept the tiny hot client small and diagnostic rows useful; cold headline stayed `not-complete` with 1 / 6 2x passes. |
| Watcher-driven dirty-project benchmark row | `target/performance/dirty-project-row-smoke.json` | Rejected because watcher delivery timed out for every generated fixture on the local macOS smoke. Replaced by deterministic `--dirty-project-path` measurement. |
| Compact dirty-project Unix protocol | `target/performance/compact-dirty-protocol-v3-smoke.json` | Reduced hot-client request/response overhead for explicit dirty project checks; cold headline stayed `not-complete` with 1 / 6 2x passes. |
| Default in-project status-file measurement | `target/performance/default-status-file-smoke.json` | Rejected because reading the default relative status file from the fixture root regressed the status diagnostic and still met 2x on only 3 / 6 realistic-equivalent fixtures. |
| Versioned status marker files | `target/performance/status-marker-smoke.json` | Rejected because marker probing added path-building/syscall overhead, regressed the status diagnostic to 2 / 6 2x passes, and left the cold headline `not-complete`. |
| Symlink-backed status file | `target/performance/status-symlink-smoke.json`; `cargo test -p assura-check-cli --test batch_cli status_cli --quiet`; `cargo clippy -p assura-check-cli --bin assura-check-status -- -D warnings` | Rejected because replacing the tiny binary status payload with an atomic symlink target and a `readlink(2)` fast path regressed the status diagnostic locally: the smoke still left `two_x_claim_verdict=not-complete`, the cold headline passed only 1 / 6 rows, and `assura-check-status-cli` met 2x on only 3 / 6 realistic-equivalent fixtures. The experiment was reverted. |
| Linux cross-host cold validation | `target/performance/vps-gw-smoke.json` | Useful attribution: Linux aggregate speedup was 2.38x, but cold headline still met 2x on only 3 / 6 fixtures, so host choice alone does not complete the universal claim. |
| `serde_yaml` alias to `serde_norway` | `target/performance/serde-norway-smoke.json` | Rejected because it still linked a libyaml-backed implementation, did not shrink `assura-check`, and regressed the 3-iteration smoke to 1.51x aggregate with 1 / 6 fixtures meeting 2x. |
| `serde_yaml` alias to `serde-saphyr` | `CARGO_TARGET_DIR=/private/tmp/assura-serde-saphyr-target cargo check -p assura-check-cli --bin assura-check --no-default-features` | Rejected because it is not a drop-in replacement for this codebase: it does not expose the `serde_yaml::Value` and `serde_yaml::Mapping` API used by LS-Lint migration and markdown/frontmatter paths. |
| Opt-in pinned external fixtures | `target/performance/external-fixtures-complete-smoke.json` | Useful scoped evidence: pinned Next.js and mdBook both clear 2x, and the eight-row aggregate reaches 4.46x. Still not a completion path for the universal gate because the small generated rows remain misses. |
| Compiled-config source metadata fingerprint | `target/performance/compiled-fingerprint-current-smoke.json`; `cargo test -p assura compiled_artifact --quiet` | Retained as artifact-contract hardening. It avoids rereading unchanged source YAML for explicit compiled artifacts on Unix, falls back to exact source-byte hashing without strong identity data, and keeps stale-artifact tests passing. It does not complete the cold headline 2x gate. |
| Prepared-check source metadata fingerprint | `cargo test -p assura prepared_check --quiet`; `cargo test -p assura-check-cli --test hot_cli --quiet` | Retained as daemon/editor-session hardening. Long-lived prepared checks can now prove an unchanged config without rereading and hashing YAML on every reload check, while same-content rewrites fall back to the hash path and changed content still reparses/recompiles. It does not complete the cold headline 2x gate. |
| Cached-check source metadata fingerprint | `cargo test -p assura-check-cli --test batch_cli cache --quiet`; `cargo test -p assura cache --quiet` | Retained as project-state-cache hardening. The opt-in cached path can now accept a fresh cache entry without reading and hashing YAML when the config fingerprint proves freshness, with content-hash fallback before YAML validation; regression tests cover fingerprint-only acceptance and stale fingerprint rejection. It does not complete the cold headline 2x gate. |
| Compiled-artifact source freshness fast paths | `target/performance/compiled-default-source-relaxed-smoke.json`; `cargo test -p assura compiled_artifact --quiet`; `cargo test -p assura-check-cli --test compiled_config_cli --quiet`; `cargo clippy -p assura --lib -- -D warnings`; `cargo clippy -p assura-check-cli --bin assura-check-compiled -- -D warnings` | Retained as low-risk compiled-config hardening. Artifact freshness checks compare already-absolute runtime paths before falling back to `canonicalize()`, and default in-project artifacts can prove source freshness from the strong source fingerprint or exact source-byte hash without also requiring an exact source config path match. Explicit `--config` invocations keep the stricter source-path contract. The smoke still left the cold headline `not-complete` with 1 / 6 rows meeting 2x and did not make the compiled diagnostic row a universal 2x path. |
| Automatic default compiled-artifact probe | `target/performance/default-compiled-artifact-isolated-smoke-2.json`; `target/performance/default-compiled-artifact-reverted-smoke.json`; `cargo test -p assura performance_report --quiet` | Rejected as a default `assura-check` behavior because probing `.assura/check-config.bin` regressed the no-artifact cold headline smoke. The code path was reverted, but the benchmark harness now removes stale default artifacts before each scenario so compiled diagnostic rows cannot contaminate later headline measurements; regression coverage preserves that cleanup invariant. |
| Low-sample claim-summary guard | `cargo test -p assura headline_summary --quiet`; `cargo test --test performance_report_contract_tests --quiet`; `pnpm --dir website build`; `target/performance/low-sample-guard-command-smoke.json` | Retained as reporting hardening. Generated reports now require at least three measured iterations before `two_x_claim_verdict=complete` can be emitted; one-iteration smokes serialize `not-complete-low-sample` even if measured rows happen to clear 2x, and the public website renders that verdict explicitly. |

These results support the existing attribution: the retained path is already
small enough that code-size trimming and report-format removal do not solve the
universal cold-subprocess target.

## Implementation State

The implementation is now in a broadly validated state:

- Native LS-Lint comparison uses the packaged binary from pinned
  `@ls-lint/ls-lint@2.3.0`.
- `assura-check` uses a lightweight check-only crate with `pico-args` for the
  common path, while companion tools use `lexopt` for richer option surfaces.
- Full CLI, git, markdown, graph, watch, and diagnostic traversal dependencies
  are gated away from the ordinary check-only binary.
- LS-Lint-compatible configs use a specialized fast path with compiled
  scope/rule/naming structures.
- Compiled config artifacts carry schema/version/source-fingerprint guards and
  stale default artifacts are rejected.
- `assura-rust-cli-floor` now measures a minimal no-op Assura-built process
  separately from both `/usr/bin/true` and the status-file product diagnostic.
- Hot/editor-session validation keeps a prepared validated config and can
  revalidate changed paths plus directly affected parent aggregate rules without
  traversing the whole project.
- Prepared editor-session checks reuse the compiled-config source fingerprint
  to skip reload work when the config metadata proves the source is unchanged,
  while retaining exact content hashing as the stale-fingerprint fallback.
- Opt-in cached checks also store the source fingerprint, allowing cache hits
  to bypass config-byte reads when the fingerprint is strong and unchanged.
- The ordinary `assura-check` path intentionally does not auto-probe
  `.assura/check-config.bin`; that prototype was measured and rejected because
  it regressed no-artifact cold CLI evidence. The performance harness now
  removes stale default compiled artifacts before each scenario so diagnostic
  compiled rows cannot contaminate the headline row.
- `assura-check-client --dirty-project-path <PATH>` gives editors a
  deterministic CLI protocol for refreshing project status from a known changed
  path without depending on OS watcher delivery.
- `warm_claim_summary` reports the persistent dirty-project session row
  separately from the cold headline claim so warm/editor-session evidence has
  its own machine-readable gate.
- The branch passes `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `target/release/assura-check --quiet .`, and `git diff --check`.

Current retained artifact inspection does not expose another obvious hidden
dependency payload to cut from the headline binary. `cargo tree -p
assura-check-cli --edges normal --no-default-features` still lists package-wide
siblings such as `notify`, `postcard`, `lexopt`, `serde_json`, and
`serde_yaml`, but per-binary release inspection is the relevant evidence for
startup work. `strings target/release/assura-check` shows YAML/JSON payloads
because the headline binary parses `.assura/config.yml` and supports structured
report output; it does not show `notify`, `postcard`, `lexopt`, `git2`, or
`clap`. `otool -L target/release/assura-check` shows only the system library.
The retained release artifact sizes are about 1.1 MB for `assura-check`, 519 KB
for `assura-check-compiled`, and 8.3 KB for `assura-check-status`. Prior
no-output/cache, minimal quiet-binary, exact quiet-parser, raw-entrypoint, and
default compiled-artifact experiments already measured this family of cuts and
did not satisfy the universal cold CLI gate.

The bounded cleanups after the fast path still matter, but they are not the
reason the cold gate completed. They either hardened correctness, improved
attribution, or confirmed rejected paths. The cold completion came from the
combination of the check-only binary, LS-Lint-compatible fast validation, and
the Linux static-CRT release artifact.

The retained warm-session hardening at
`target/performance/session-config-fingerprint-smoke.json` does not change the
cold verdict either. It does close an important correctness gap for the warm
contract: unchanged config uses the cheap fingerprint path, while changed config
forces reload even when notify does not deliver an event or the config lives
outside the watched project tree.

Those are implementation-readiness signals for the warm/editor-session product
mode. They are intentionally summarized by `warm_claim_summary`, not folded
into the cold `claim_summary`.

## Completion Paths That Remain Honest

1. Preserve the completed cold claim as Linux static-CRT release evidence.
   Do not generalize it to default local macOS dynamic builds.
2. Keep the persistent CLI service/editor-session mode as a separate compared
   interface. The persistent `assura-check-session` path satisfies this warm
   contract in the tracked report.
3. If a future PR wants a broader claim, define and measure that new scope
   explicitly instead of reusing this checked-in evidence.

The warm path is now separately measurable and complete under the persistent
session contract: `warm_claim_summary.assura_row_family` is
`assura-check-dirty-project-session-cli`,
`warm_claim_summary.two_x_pass_count=6`, and
`warm_claim_summary.two_x_claim_verdict=complete` in
`benches/history/current.json`. The one-shot CLI-client model still misses
small rows, so the honest warm path is a daemon/editor session contract rather
than more per-check client micro-optimization.

The existing multi-root `assura-check` path is not an obvious missed
implementation shortcut: `src/cli/check/batch.rs` already groups paths by
project root/config path and reuses the loaded config plus `StructureChecker`
for subsequent paths in the same project. A future amortized benchmark should
therefore be framed as a benchmark/product-scope decision, not as a simple
batch implementation fix.

The current check-only binary also still carries JSON/YAML report formatting,
but that is not an untested completion route. A prior default build without
JSON/cache support shrank the binary and still regressed the headline
benchmark. The explicit quiet-only parser and minimal quiet-binary experiments
were also measured and rejected. Removing structured output from the public
check binary would therefore be a product-surface tradeoff, not a proven path
to the universal cold-subprocess 2x target.

The explicit compiled-config path is similarly not a hidden cold headline
route. It remains useful for prepared and cached execution contracts, but the
public cold claim is now the measured Linux static-CRT `assura-check-cli` row.

Fresh generated reports guard against accidental low-sample completion claims:
`claim_summary.two_x_claim_verdict` cannot be `complete` unless the report was
generated with at least three measured iterations.

## Audit Conclusion

The active performance objective is complete for the Linux static-CRT release
artifact path and complete for the separate persistent warm/editor-session
contract. Remaining work is PR hygiene: keep the docs, website, and PR body
scoped to those claims; do not present local macOS dynamic evidence as the
headline release result.
