---
title: Assura Native LS-Lint Performance Rearchitecture
status: completed
created: 2026-05-18
---

# Assura Native LS-Lint Performance Rearchitecture

## Objective

Make Assura's public check path faster than native LS-Lint on equivalent
LS-Lint-compatible fixture rules, without hiding extra work in the comparison
or weakening correctness coverage.

## Context

The native LS-Lint correction in PR #11 changed the headline result. Native
LS-Lint is faster than the full `assura check` CLI on all six realistic
headline fixtures, while Assura's in-process structure check is faster than
native LS-Lint on the same fixtures.

The dedicated `assura-check` entrypoint now beats native LS-Lint by more than
2x on all six realistic headline rows in the checked-in Linux static-CRT
release report. The check binary also builds without the full CLI, markdown,
intelligence, git, watch, and graph dependency surfaces. Local macOS dynamic
builds remain diagnostic and previously exposed a startup floor, but they are
not the current release claim.

Reference analysis:

- `docs/analysis/2026-05-18-native-ls-lint-performance-gap-review.md`
- `benches/history/current.json`
- `website/public/data/performance/current.json`

## Completion Gates

This goal required all of these to be true before it could be marked complete:

1. Headline performance rows compare Assura against the packaged native
   LS-Lint binary, not `npm exec` or `node_modules/.bin/ls-lint`.
2. The public website reports the current winner honestly from data.
3. A lightweight check path exists that excludes unrelated runtime surfaces
   from ordinary structure validation.
4. LS-Lint-compatible fixture rows use an Assura path that does only
   LS-Lint-equivalent work unless explicitly labeled otherwise.
5. The final report shows Assura faster than native LS-Lint on the realistic
   equivalent fixture set, or the PR documents the remaining gap with exact
   attribution and does not claim victory.
6. Any claim of "2x faster than LS-Lint" is backed by checked-in native-binary
   CLI evidence for the exact row set being claimed.
7. Regression coverage proves the fast path preserves LS-Lint parity fixtures
   and current structure-first validation behavior.
8. If the requested 2x target is below the observed subprocess floor for small
   fixtures, the PR must document that floor with commands and must not claim
   the target is technically achieved.

## Current Status

As of the latest 2026-05-19 checked-in report, the cold-subprocess 2x gate is
complete for Linux static-CRT release artifacts:

```text
claim_summary.two_x_claim_verdict = complete
claim_summary.two_x_pass_count = 6
claim_summary.two_x_fail_count = 0
claim_summary.aggregate_speedup_ratio = 2.8980855186211874
assura-check-cli.assura_binary_profile = release-static-crt
```

The local macOS dynamic lower-bound evidence remains valuable for attribution,
but it must not be presented as the current release claim. Further work should
not repeat cold-start micro-optimizations already rejected in the progress log
unless new evidence changes the measurement floor or the product claim scope.

Binary size is not a primary completion criterion. It is useful only when it
removes unnecessary startup or hot-path work and improves measured runtime.
Speed is the product priority because agentic development will repeatedly hit
the validation tool during an editing session.

The warm/editor-session execution model has separate positive evidence:
`warm_claim_summary` tracks `assura-check-dirty-project-session-cli` and is
complete in the checked-in report. It should stay separate from the cold
`assura-check-cli` release-artifact claim.

The durable progress ledger for this goal is
`docs/analysis/2026-05-19-ls-lint-performance-progress-ledger.md`. Before
starting another cold-path optimization, check its rejected experiment list and
decision rules. If the next hypothesis is only another parser tweak,
report-format strip, binary-size reduction, or default compiled-artifact probe,
do not start there without new profiling evidence.

## Work Slices

### Slice 1: Honest Baseline

Ship the benchmark correction already identified:

- resolve `node_modules/@ls-lint/ls-lint/bin/ls-lint-<platform>`,
- store `ls_lint_execution_mode`,
- update website copy to avoid false speedup claims,
- keep checked-in performance history generated from release-mode runs.

Validation:

```bash
cargo fmt --all -- --check
git diff --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --quiet
pnpm --dir website build
cargo run --quiet -- check --format json .
```

### Slice 2: Lightweight Check Entrypoint

Create a check-focused binary or feature split that does not initialize or link
workflow features irrelevant to lint-style validation.

Remove from the check startup path where possible:

- Tokio runtime creation,
- unconditional tracing subscriber setup,
- `git2` / OpenSSL / hook management,
- `notify` watch support,
- graph/agent/plugin roadmap dependencies,
- heavyweight validation crates unused by structure check.

Target evidence:

```bash
cargo build --release -p assura --bin assura
cargo build --release -p assura-check-cli
target/release/assura-check --version
target/release/assura-check --help
target/release/assura-check <fixture> --format json
```

Build `assura` and `assura-check-cli` in separate Cargo invocations. A single
workspace `cargo build --release --bins` can unify default features and
reintroduce `git2`/OpenSSL into the check-only executable.

Acceptance:

- startup-only median is within 1.25x native LS-Lint startup,
- CLI fixture rows materially close the current 13-16 ms wrapper gap,
- existing `assura check` behavior remains available or is explicitly migrated.

Current status:

- `crates/assura-check-cli` exists and depends on `assura` with
  `default-features = false`.
- `git2` is behind the opt-in `git-signals` feature for the full CLI; the
  default `assura` crate feature set keeps `full-cli` without enabling git
  signals.
- `target/release/assura-check` links only `libSystem` on macOS when built in a
  separate `cargo build --release -p assura-check-cli` invocation.
- `assura-check-cli` no longer pulls the full CLI dependency surface when built
  with `assura = { default-features = false }`; the large optional surfaces are
  gated behind the `full-cli` feature, and config validation derives stay on
  the full CLI path rather than the check-only binary path.
- The diagnostic jwalk traversal strategies are gated behind `full-cli`, so
  check-only builds no longer include `jwalk`, `rayon`, or `crossbeam`.
- Simple suffix patterns are matched directly rather than compiled as globs in
  checker initialization.
- A conservative LS-Lint-compatible fast path now handles naming/count/ignore
  configs and falls back to the full structure engine for richer Assura rules
  or `--fail-fast`.
- Release builds now use LTO, one codegen unit, stripping, and `panic = "abort"`.
- The checked-in 5-iteration release report beats native LS-Lint on all six
  realistic headline rows, but only `ignored_generated_heavy_repo` exceeds 2x:
  `simple_library` 3.58 ms vs 5.48 ms, `web_app` 4.05 ms vs 5.09 ms,
  `monorepo_packages` 4.47 ms vs 5.28 ms, `monorepo_policy` 5.60 ms vs
  8.85 ms, `rule_heavy_repo` 4.74 ms vs 6.42 ms, and
  `ignored_generated_heavy_repo` 3.71 ms vs 10.19 ms.
- `assura-check` accepts multiple path arguments for batch validation. A fair
  same-config six-root batch smoke is currently not a headline win
  (21.97 ms vs 21.44 ms median in the latest local smoke), and still not 2x.

### Slice 3: Compiled Structure Plan

Move `run_structure_check` from per-directory rule resolution toward a compiled
plan:

- compile inherited directory rules once,
- store compact path-scope and extension-scope matchers,
- avoid cloning `EffectiveRules` during directory traversal,
- keep direct-content policies separate from inherited naming policies,
- measure plan-build and walk/validate phases separately.

Acceptance:

- in-process `monorepo_policy` and `rule_heavy_repo` improve from current
  3.63 ms and 1.63 ms medians,
- all realistic LS-Lint fixture tests pass,
- phase rows expose plan-build versus walk/validate costs.

### Slice 4: LS-Lint-Compatible Fast Path

Add a specialized fast path for configs that are native LS-Lint parity:

- directory scope + extension index,
- compiled name rules,
- ignored-path pruning before validation,
- no Assura-only required structure, markdown, size, line, or docs checks.

Acceptance:

- fast path matches LS-Lint parity fixtures,
- fast path is used for LS-Lint-equivalent headline rows,
- full structure-first rows stay available as richer diagnostic evidence.

### Slice 5: Public Claim Refresh

Only after the corrected native comparison supports the intended public claim:

- update `website/src/content/docs/reference/performance.mdx`,
- update `website/src/content/docs/reference/performance-implementation.mdx`,
- regenerate `benches/history/current.json`,
- regenerate `website/public/data/performance/current.json`,
- update PR body with exact commands and winner table.

### Slice 6: Compiled Config and Incremental State

Use compiled config artifacts as the bridge from cold CLI optimization to
editor-session incremental validation.

Config artifact requirements:

- fingerprint the source config file contents, canonical project root, Assura
  version, artifact schema version, and relevant feature flags,
- store a validated portable config plus compiled rule plan instead of
  rehydrating YAML-facing runtime structs,
- skip config validation only when the fingerprint and schema match exactly,
- invalidate on config edits, config path changes, Assura upgrades, or artifact
  version changes,
- stale or unreadable artifacts must fall back to a normal validated config load.

Incremental validation requirements:

- keep a project file-state index in a daemon/editor process or explicit cache,
- fingerprint file metadata and content only at the granularity needed by active
  rules,
- map changed files to affected rule scopes before walking the whole project,
- for a single edited file, validate naming/content rules directly and only
  re-check parent directory aggregate rules when the file set changed,
- still run full traversal for cold CLI checks unless a valid file-state index
  proves the tree is unchanged.

Non-goals:

- do not claim a fresh one-shot CLI can avoid traversal without a trusted
  external index,
- do not deserialize unvalidated artifacts from arbitrary locations without a
  schema/version/fingerprint guard,
- do not duplicate the validation engine only to produce a quiet success exit
  code unless benchmarks prove the separate path is worth the complexity.

## Review Criteria

Reviewers should block on any of these:

- LS-Lint is measured through the npm wrapper again.
- Website copy says Assura is faster when data says otherwise.
- A fast path skips checks while still being labeled as full Assura structure
  validation.
- The final performance claim mixes debug and release builds.
- In-process-only speed is presented as CLI speed.

## Progress Log

- 2026-05-21: Reconciled the goal status after the Linux static-CRT report.
  The current checked-in cold `claim_summary` is complete for the
  `assura-check-cli` Linux static-CRT release artifact (`6 / 6`, 2.898x
  aggregate), and the warm `assura-check-dirty-project-session-cli` gate is
  separately complete (`6 / 6`, 25.213x aggregate). Earlier entries below
  remain as dated experiment history from before the static-CRT completion and
  should not be read as the current verdict.
- 2026-05-19: Added `assura-check-session`, a persistent CLI process that
  reads repeated commands from stdin and forwards checks to `assura-checkd`.
  The performance report now includes
  `assura-check-dirty-project-session-cli` and uses that row for
  `warm_claim_summary`. The tracked 5-iteration report keeps the cold
  `assura-check-cli` gate honest and incomplete (`1 / 6`, 1.58x aggregate),
  while the warm persistent-session gate is complete (`6 / 6`, 40.16x
  aggregate). At that point in the sequence, the original cold subprocess claim
  was not achieved, but the agent/editor warm CLI session contract was.
- 2026-05-19: Attempted syscall-level cold-start profiling with `dtruss` on a
  minimal temp project. The command was blocked by macOS DTrace privileges/SIP
  both in the normal sandbox and after requesting elevated execution. Evidence:
  `target/performance/cold-start-dtruss-blocked.txt`. A deeper cold-path
  profile now requires Instruments/DTrace access on a host where those tools
  are permitted; otherwise the retained lower-bound decision should stand.
- 2026-05-19: Added a durable progress ledger and stop rules for future
  performance work. Further cold work should require a new startup-floor
  hypothesis and artifact-backed experiment record; otherwise focus on the
  persistent daemon/editor-session contract. Binary size remains secondary.
- 2026-05-19: Hardened the warm daemon/session config freshness contract.
  `assura-checkd` now probes the prepared config fingerprint on every request,
  so unchanged config avoids validation while changed config reloads even when
  notify misses the event or the config is outside the watched project tree; it
  also lets `assura-check-session` reuse one daemon socket. Latest smoke:
  `benches/history/current.json`: warm stayed complete (`6 / 6`, 40.16x),
  cold stayed incomplete.
- 2026-05-19: Added `assura-check-dirty-project-socket` as a diagnostic
  profiling row. It starts the same hot daemon, mutates the same deterministic
  fixture path, and times connect/write/read from the report process without
  launching `assura-check-unix-client`. The tracked report shows the direct
  socket row at roughly 0.15-0.44 ms on realistic fixtures, far below every 2x
  target, while the one-shot CLI-client warm row remains incomplete. This
  isolates the remaining warm gap to the subprocess/client boundary rather
  than daemon-side validation.
- 2026-05-19: Rejected a single-write raw Unix client request experiment for
  the warm dirty-project path. The prototype assembled `D\t<path>\n` and
  `CHECK-PATH\t<path>\n` into a stack buffer so the common request used one
  write syscall instead of prefix/path/newline writes, with fallback for long
  paths. A 3-iteration release smoke at
  `target/performance/unix-client-single-write-smoke.json` regressed the warm
  dirty-project evidence to 3 of 6 realistic-equivalent fixtures meeting 2x
  and 2.16x aggregate speedup, below the then-current tracked 4 of 6 / 2.29x
  baseline. The code was reverted.
- 2026-05-19: Rejected reusing a single prepared `StructureChecker` across hot
  daemon changed-path requests. The hypothesis was that avoiding per-request
  cloning of compiled config/rule structures and preserving the rules cache
  would reduce warm dirty-project latency. The first 3-iteration smoke at
  `target/performance/prepared-reuse-checker-smoke.json` and the second at
  `target/performance/prepared-reuse-checker-smoke-2.json` did not improve the
  warm summary; a tracked 5-iteration refresh also remained incomplete and
  regressed below the prior tracked warm state. The code was reverted.
- 2026-05-19: Rejected two additional startup-oriented experiments. Building
  `assura-check-cli` with `CARGO_PROFILE_RELEASE_OPT_LEVEL=z` shrank
  `assura-check` from about 1.0 MB to about 764 KB, but
  `target/performance/check-cli-opt-z-smoke.json` regressed the headline
  aggregate ratio to 1.65x with only 1 of 6 realistic-equivalent fixtures
  meeting the 2x target. A separate minimal `assura-check-quiet` binary avoided
  report formatting code and measured at about 855 KB, but
  `target/performance/check-quiet-smoke.json` was not faster than the existing
  `assura-check` row on the headline set. Both experiments were removed rather
  than adding unsupported public surface area.
- 2026-05-19: Rejected a Unix raw-entrypoint experiment for the retained
  `assura-check` binary. The experiment bypassed Rust's default `std::rt`
  entry wrapper while keeping the same `pico-args` parser and validation
  engine. A 15-iteration release smoke at
  `target/performance/raw-main-15-smoke.json` still reported
  `two_x_claim_verdict=not-complete`, 1 of 6 headline 2x passes, and a 1.66x
  aggregate speedup, which was worse than the checked-in 1.68x aggregate. The
  raw-entrypoint code was removed.
- 2026-05-19: Rejected an exact `assura-check --quiet` fast-parse branch for
  the retained `pico-args` CLI. The branch bypassed the general parser only for
  the public quiet/default invocation used by the headline row, but
  `target/performance/quiet-fast-parse-smoke.json` regressed the aggregate to
  1.53x, reduced `assura_faster_count` to 5 of 6, and still passed only 1 of 6
  headline 2x targets. The parser fast path was removed.
- 2026-05-19: Rejected removing JSON/cache support from the default
  `assura-check` build. Gating `serde_json`, `assura/json-output`, and
  `--cache-dir` behind an opt-in feature shrank `assura-check` from about
  1.0 MB to 940 KB, but `target/performance/no-json-check-smoke.json`
  regressed the aggregate speedup to 1.64x, still passed only 1 of 6 headline
  2x targets, and broke the cached diagnostic row for default release builds.
  The feature split was removed.
- 2026-05-19: Rejected lazy file-stem computation inside the LS-Lint-compatible
  fast validator. The change avoided scanning file names before confirming that
  a naming rule applied and improved some walk-phase samples, but the measured
  cold CLI target regressed: `target/performance/lazy-stem-15-smoke.json`
  showed a 1.59x aggregate speedup and still only 1 of 6 headline 2x passes.
  The change was removed.
- 2026-05-19: Refreshed the retained implementation after the rejected
  experiments. `target/performance/current-retained-smoke.json` showed
  `assura-check-cli` faster than native LS-Lint on all six realistic-equivalent
  headline fixtures, but still not complete for the requested universal 2x
  gate: 1 of 6 headline 2x passes, 1.66x aggregate speedup, and
  `two_x_claim_verdict=not-complete`.
- 2026-05-19: Added diagnostic changed-path hot-client evidence for the
  editor-session/incremental-validation direction. `assura-check-unix-client`
  now accepts an optional changed path and sends `CHECK-PATH` to the daemon, and
  `assura performance-report` includes `assura-check-changed-path-cli` as a
  diagnostic-only row. A 3-iteration release smoke at
  `target/performance/changed-path-cli-smoke.json` showed the incremental row
  validating one changed file in roughly 2.7-4.9 ms across the displayed
  fixtures, but the headline cold `assura-check-cli` gate remains
  `two_x_claim_verdict=not-complete` with 2 of 6 realistic-equivalent fixtures
  meeting the 2x target. This is useful evidence for daemon/editor-session
  architecture, not proof that the one-shot CLI objective is complete.
- 2026-05-19: Added a lightweight semantic config-validation path for
  compiled artifacts and hot validation sessions without pulling the full
  `validator` derive stack into the check-only runtime. `ConfigLoader` now has
  `parse_validated` / `load_validated` for artifact compilation and daemon
  config reloads, while the normal cold `assura-check` path keeps the fast
  parse behavior to avoid paying extra validation work on every comparison
  run. `assura-check-compile-config` now rejects invalid naming conventions in
  the no-full-cli package before writing a binary artifact, and
  `PreparedStructureCheck` validates config semantics on initial load/reload.
  Focused gates passed: `cargo fmt --all -- --check`, `cargo check -p assura
  --no-default-features`, `cargo test -p assura-check-cli
  compile_config_rejects_invalid_semantics_without_full_cli_validator --quiet`,
  `cargo test -p assura prepared_check_reloads_when_config_changes --quiet`,
  `cargo test -p assura-check-cli --test batch_cli --quiet`, and `cargo
  clippy -p assura-check-cli --bins --tests -- -D warnings`. A 3-iteration
  release smoke at `target/performance/semantic-validated-artifacts-smoke.json`
  still showed `claim_summary.two_x_claim_verdict=not-complete`, with
  `assura-check-cli` meeting 2x on only one of six realistic-equivalent
  headline fixtures, so checked-in benchmark history was not refreshed.
- 2026-05-19: Hardened the default `assura-check-compiled` project artifact
  path so it verifies `.assura/check-config.bin` against
  `.assura/config.yml` automatically. Explicit `--compiled-config` remains
  portable unless callers pass `--config`, but the default project-local
  artifact now rejects stale config without requiring users to remember the
  extra flag. Added
  `compiled_config_cli_rejects_stale_default_project_artifact` and split the
  compiled-config integration coverage into
  `crates/assura-check-cli/tests/compiled_config_cli.rs` so
  `batch_cli.rs` stays under the Assura file-length policy. Focused checks
  passed: `cargo fmt --all -- --check`, `cargo test -p assura-check-cli
  --test compiled_config_cli --quiet`, `cargo clippy -p assura-check-cli
  --bin assura-check-compiled -- -D warnings`, and Assura self-checks for the
  touched files. A 3-iteration release smoke at
  `target/performance/compiled-default-stale-check-smoke.json` still showed
  `claim_summary.two_x_claim_verdict=not-complete` with the headline
  `assura-check-cli` row meeting 2x on only one of six realistic-equivalent
  fixtures, so this is a correctness/architecture improvement rather than
  completion evidence.
- 2026-05-19: Added a top-level `claim_summary` verdict to
  `assura performance-report` so the headline 2x gate is machine-readable and
  cannot be satisfied by diagnostic hot/status/in-process rows. The website now
  renders the verdict directly: the refreshed checked-in data shows `2x gate:
  Not complete`, `2x cases: 1 / 6`, and `Assura faster: 5 / 6` for the
  realistic-equivalent headline row set. Focused checks passed:
  `cargo fmt --all -- --check`, `cargo test -p assura claim_summary --quiet`,
  `cargo test -p assura
  synthetic_and_diagnostic_families_are_not_headline_rows --quiet`,
  `cargo check -p assura --no-default-features`, Assura self-checks for the new
  report and website files, a 1-iteration release smoke at
  `target/performance/claim-summary-smoke.json`, the tracked 5-iteration
  release refresh in `benches/history/current.json`, and
  `pnpm --dir website build`. Added
  `tests/performance_report_contract_tests.rs` so the checked-in
  `claim_summary` is recomputed from headline rows during tests rather than
  trusted as an unverified report field. Updated
  `.agents/skills/assura-performance-reporting/SKILL.md` so future report
  refreshes inspect `claim_summary` and use the current `assura-check-cli`
  headline row instead of the legacy full-CLI row. The objective remains
  incomplete.
- 2026-05-19: Corrected subprocess timing boundaries in
  `assura performance-report` so Assura, native LS-Lint, process-floor,
  hot-client, and status-client rows start timing immediately before
  `Command::status()` rather than before benchmark-parent `Command` builder
  setup. This keeps process launch, child execution, and wait time in scope
  while removing parent-side setup noise that is not part of either CLI binary.
  The refreshed checked-in report improved the aggregate headline ratio to
  1.68x and restored `assura_faster_count=6`, but it still shows only
  `two_x_pass_count=1` and `two_x_claim_verdict=not-complete`. Evidence:
  `target/performance/command-status-timing-smoke.json`,
  `benches/history/current.json`, `website/public/data/performance/current.json`,
  and `pnpm --dir website build`.
- 2026-05-19: Kept the direct-policy fast-rule flags after focused
  validation and release measurement. The flags let the LS-Lint-compatible fast
  validator skip allowed/forbidden/allow-extra checks when a resolved scope has
  no direct child policy, and the fast path now preserves the full validator's
  extension allowlist behavior when direct file policy is present. Focused
  gates passed: `cargo fmt --all -- --check`,
  `cargo test -p assura --test cli_check_tests
  check_supports_multi_part_extension_rules_without_leading_dot --quiet`,
  `cargo test -p assura ls_fast --quiet`,
  `cargo check -p assura --no-default-features`, and `git diff --check`. A
  7-iteration release smoke at
  `target/performance/fast-direct-policy-flags-smoke.json` improved the
  compiled CLI row enough for `monorepo_policy` to meet the 2x target, but the
  objective remains incomplete: cold `assura-check-compiled-cli` meets the
  target on only two of six realistic-equivalent headline fixtures
  (`monorepo_policy` and `ignored_generated_heavy_repo`), while
  `simple_library`, `web_app`, `monorepo_packages`, and `rule_heavy_repo`
  remain above target.
- 2026-05-19: Rejected making the normal `assura-check` single-path flow
  automatically read/write compiled-plan artifacts. The prototype used the
  existing `postcard` artifact, invalidated by source config content hash and
  artifact compatibility, and placed automatic cache files under `.git/assura`
  or a temp cache namespace. It was architecturally aligned with config-dirty
  checking, but measured worse for the active CLI target: the optimized
  `assura-check` binary grew from about 1.0 MB to about 1.1 MB, and
  `target/performance/compiled-plan-cache-smoke.json` showed
  `assura-check-cli` still meeting only one of six realistic-equivalent
  headline fixtures. The prototype was removed; the existing explicit
  `assura-check-compile-config` / `assura-check-compiled` path remains the
  lower-overhead compiled-artifact route.
- 2026-05-19: Rejected an exact `assura-check-compiled --quiet` pre-parser
  for the default project-root artifact invocation. The special case bypassed
  the general `pico-args` parser only for the benchmarked invocation shape, but
  the optimized binary grew from about 502 KB to about 506 KB and
  `target/performance/compiled-quiet-fast-parser-smoke.json` showed the
  compiled CLI meeting only one of six realistic-equivalent headline fixtures.
  The code was removed; `pico-args` remains the measured lower-risk parser.
- 2026-05-19: Rejected folding `files.exists` / `directories.exists` count
  validation into the main LS-Lint-compatible traversal pass. The experiment
  reduced some tiny walk phases by avoiding a second `read_dir` for directories
  with count constraints, but the release CLI evidence regressed: 
  `target/performance/fast-count-integrated-smoke.json` showed
  `assura-check-compiled-cli` meeting only one of six realistic-equivalent
  headline fixtures and losing the previous `monorepo_policy` 2x pass. The
  code was removed in favor of the prior measured state.
- 2026-05-19: Rejected lazy relative-path stringification inside direct-policy
  glob matching. The change avoided allocating the relative path when a simple
  filename suffix was enough, but release evidence regressed and the compiled
  binary grew to about 506 KB. `target/performance/lazy-rel-pattern-smoke.json`
  showed `assura-check-compiled-cli` meeting only one of six
  realistic-equivalent headline fixtures, so the code was removed.
- 2026-05-18: Continuing after the first fast-check split. Next slice is
  removing remaining diagnostic traversal dependencies from the check-only
  binary before rerunning native LS-Lint evidence.
- 2026-05-18: Gated diagnostic jwalk traversal behind `full-cli` and skipped
  glob compilation for simple suffix patterns. The checked-in 15-iteration
  native report still only exceeds 2x on `ignored_generated_heavy_repo`.
- 2026-05-18: Added conservative LS-Lint-compatible fast path. The checked-in
  15-iteration native report still only exceeds 2x on
  `ignored_generated_heavy_repo`.
- 2026-05-18: Reviewed remaining hot path after native-binary correction and
  check-only split. Added a no-allocation fast path for common single naming
  conventions and changed `assura-check-cli` measurement to validate from the
  fixture working directory, matching native LS-Lint's invocation shape. The
  refreshed 15-iteration native report still only exceeds 2x on
  `ignored_generated_heavy_repo`; the other rows remain process-floor-bound.
- 2026-05-18: Compiled LS-Lint-compatible fast-path naming rules into suffix /
  glob matchers plus parsed naming validators. This reduced the tracked
  `rule_heavy_repo` walk phase to 0.79 ms and `monorepo_policy` walk phase to
  1.63 ms, but the refreshed native CLI report still only exceeds 2x on
  `ignored_generated_heavy_repo`.
- 2026-05-18: Moved LS-Lint-compatible fast-plan compilation out of the
  per-check path and added a reusable compiled rule-scope plan for the full
  structure engine. The full engine now resolves inherited directory rules from
  the compiled plan instead of recursively recomputing inheritance from the raw
  config tree for every directory. Focused validation passed, and a 5-iteration
  target smoke still showed the realistic rows below universal 2x, confirming
  that the remaining headline gap is dominated by process/startup and workload
  shape rather than per-directory rule resolution alone. Evidence:
  `cargo fmt --all -- --check`, `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo run --quiet -- check --format json .`, and
  `target/release/assura performance-report --output
  target/performance/compiled-rule-plan-smoke.json --history
  target/performance/compiled-rule-plan-smoke.jsonl --website-dir
  target/performance/compiled-rule-plan-smoke-website --iterations 5`.
- 2026-05-19: Extended the LS-Lint-compatible fast-plan eligibility gate to
  accept direct `allowed_names`, `allowed_patterns`, `forbidden_patterns`, and
  `allow_extra` policies that the fast validator already enforces. Added
  compiled-config CLI regression coverage for direct-policy fast artifacts and
  removed the obsolete `bincode` dependency from `assura-check-cli` after the
  artifact format moved to `postcard`.
- 2026-05-19: Added ASCII fast branches for static naming conventions while
  preserving Unicode fallback behavior, then tightened common delimited cases
  into one-pass ASCII validation. A 5-iteration release smoke at
  `target/performance/ascii-delimited-fast-smoke.json` still did not meet
  universal 2x: `assura-check-cli` and `assura-check-compiled-cli` each meet
  only `ignored_generated_heavy_repo`; `monorepo_policy` is close but remains
  over target, and smaller fixtures are marked process/Rust-floor blocked. The
  goal remains incomplete.
- 2026-05-19: Removed avoidable measured hot-daemon protocol work by reading
  client requests into a fixed buffer, parsing the text protocol from bytes,
  and writing single-digit success/error responses without `format!`. A
  15-iteration release smoke at `target/performance/hot-protocol-fast-15.json`
  shows the daemon-backed CLI rows now meet 2x on four of six realistic
  fixtures, but `simple_library` and `web_app` remain blocked by measured
  process/Rust CLI floors. Cold `assura-check-cli` and
  `assura-check-compiled-cli` still do not meet the universal 2x gate.
- 2026-05-19: Added a prechecked compiled-artifact entrypoint so
  `assura-check-compiled` can avoid duplicate existence/canonicalization work
  after resolving the checked path for project discovery. A 5-iteration release
  smoke at `target/performance/prechecked-compiled-smoke.json` still shows
  cold `assura-check-compiled-cli` meeting 2x only on
  `ignored_generated_heavy_repo`; the goal remains incomplete.
- 2026-05-19: Added a measured-shape fast path for `assura-check-compiled`
  when invoked from the project root without an explicit checked path. This
  avoids generic parent project discovery for the common compiled CLI
  invocation while preserving fallback discovery for explicit paths and
  subdirectories. A 5-iteration release smoke at
  `target/performance/compiled-root-fast-smoke.json` still shows cold
  `assura-check-compiled-cli` meeting 2x only on
  `ignored_generated_heavy_repo`; the goal remains incomplete.
- 2026-05-19: Made `.assura/check-config.bin` the default
  `assura-check-compiled` artifact path and updated the performance row to use
  that default, matching native LS-Lint's default-config invocation shape more
  closely. Regression coverage now proves the default artifact path works. A
  5-iteration release smoke at
  `target/performance/default-compiled-artifact-smoke.json` still shows cold
  `assura-check-compiled-cli` meeting 2x only on
  `ignored_generated_heavy_repo`; the goal remains incomplete.
- 2026-05-18: Added two incremental-validation experiments. First,
  `assura-check --cache-dir` records hot results for LS-Lint-compatible configs
  and invalidates them when the config hash or recursive directory mtime
  snapshot changes; this proved safe but not fast enough because it still pays
  process startup and snapshot verification. Second, added `assura-checkd`
  using the established Rust `notify` watcher plus a tiny
  `assura-check-client` binary, and added an `assura-check-hot-cli` performance
  row. The 5-iteration smoke showed hot mode improves larger rows but still
  does not meet universal 2x because the thin client itself remains near the
  local subprocess floor: `simple_library` 7.25 ms vs LS-Lint 9.72 ms,
  `web_app` 6.36 ms vs 9.30 ms, `monorepo_packages` 5.80 ms vs 10.08 ms,
  `monorepo_policy` 7.14 ms vs 13.07 ms, `rule_heavy_repo` 6.45 ms vs
  11.95 ms, and `ignored_generated_heavy_repo` 5.32 ms vs 16.22 ms. Evidence:
  `target/performance/hot-check-smoke.json`, `cargo test --all-targets
  --quiet`, `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo run --quiet -- check --format json .`, and `git diff --check`.
- 2026-05-18: Switched the hot daemon/client path from TCP to Unix domain
  sockets on Unix and tightened the stale-binary guard so full-CLI-only source
  changes do not skip the check-only rows. A 9-iteration smoke with all rows
  present still did not meet universal 2x: `simple_library` hot 6.31 ms vs
  LS-Lint 9.03 ms, `web_app` hot 6.35 ms vs 9.35 ms, `monorepo_packages` hot
  6.15 ms vs 9.46 ms, `monorepo_policy` hot 6.32 ms vs 14.21 ms,
  `rule_heavy_repo` hot 6.61 ms vs 9.76 ms, and
  `ignored_generated_heavy_repo` hot 5.28 ms vs 19.32 ms. A local subprocess
  floor check measured `/usr/bin/true` at about 6.66 ms median, which explains
  why tiny-fixture universal 2x is not achievable with a separate CLI process
  on this machine. Evidence:
  `target/performance/hot-check-uds-smoke-4.json` and the local subprocess
  floor command recorded in the session log.
- 2026-05-18: Added a machine-readable `process-floor` diagnostic row to
  `assura performance-report` using the same Rust `Command` harness as the
  Assura and LS-Lint subprocess rows. A 5-iteration smoke showed the process
  floor itself is 3.06-4.82 ms across the realistic rows, while the hot
  daemon/client path remains 5.21-6.93 ms. This proves there is some IPC/client
  overhead left, but also narrows the remaining possible gain: even a perfect
  tiny client would only clear universal 2x if it stayed very close to process
  floor on every row. Evidence:
  `target/performance/process-floor-smoke.json`.
- 2026-05-18: Added a daemon-maintained binary status-file experiment:
  `assura-checkd --status-file` writes the latest clean/dirty exit state, and
  `assura-check-status` reads that compact file from a tiny CLI process. The
  row is diagnostic-only because the validation work is maintained by a hot
  daemon rather than performed by the measured CLI process. A 5-iteration smoke
  did not improve the floor: `assura-check-status-cli` measured 5.64-7.18 ms
  across realistic rows, while native LS-Lint measured 8.75-18.60 ms and the
  process floor measured 3.22-4.64 ms. This confirms that replacing socket IPC
  with a binary status file is not enough to meet universal 2x; process launch
  and loader overhead still dominate tiny editing-session clients. Evidence:
  `target/performance/status-file-smoke-4.json`.
- 2026-05-18: Tried and removed a minimal `assura-check-exit` binary that only
  performed validation and returned an exit status. It was slower than
  `assura-check --quiet` in a 5-iteration smoke
  (`target/performance/exit-cli-smoke.json`), so keeping it would add product
  surface without advancing the performance goal.
- 2026-05-18: Marked `assura-check-hot-cli` as diagnostic-only evidence. The
  hot daemon/client path is useful for editor-session architecture exploration,
  but the measured client process does not perform a full cold validation run,
  so it must not drive the headline CLI-to-CLI claim.
- 2026-05-18: Gated legacy config AST/parser/preprocessor, top-level
  `ls_compat`, and the older validation engine behind `full-cli` so
  no-default-feature check-only builds expose only the structure config surface
  they need. This compiled cleanly but did not reduce the optimized
  `assura-check` binary image or close the runtime gap; a 5-iteration smoke
  still showed `assura-check-cli` at 6.70-10.29 ms on realistic rows. Evidence:
  `target/performance/legacy-gating-smoke.json`.
- 2026-05-18: Replaced the structure-check path's dependency on the full
  `regex` crate with `regex-lite`, an established lightweight crate from the
  Rust regex project. This removed `regex`, `aho-corasick`, and
  `regex-automata` from the no-default-feature `assura-check` dependency graph
  and reduced the optimized `assura-check` binary from about 2.1 MB to 1.0 MB.
  A 5-iteration smoke improved the realistic cold CLI rows to 4.02-5.94 ms for
  most fixtures but still did not meet universal 2x. Evidence:
  `target/performance/regex-lite-smoke.json`.
- 2026-05-18: Tried package-specific release `opt-level = "z"` for `assura`
  and `assura-check-cli`. It reduced code size slightly but worsened runtime
  on the realistic rows, so the profile tweak was removed. Evidence:
  `target/performance/regex-lite-size-profile-smoke.json`.
- 2026-05-18: Tried removing JSON output and the opt-in report cache from the
  check-only CLI. This reduced the optimized binary to about 908 KB but
  worsened runtime in the realistic comparison smoke, so JSON and
  `--cache-dir` support were restored. Evidence:
  `target/performance/no-json-cache-check-smoke.json`.
- 2026-05-18: Added `PreparedStructureCheck`, a parsed/validated/compiled
  structure-check plan for long-lived sessions. `assura-checkd` now keeps this
  plan hot and reloads it only when `.assura/config.yml` content changes,
  which gives the editor-session architecture a real config-dirty boundary.
  Also fixed the cached-row benchmark to keep its cache directory outside the
  checked fixture tree; the previous in-tree cache correctly triggered naming
  violations in strict fixtures. The cached row is now diagnostic-only because
  it reuses warmed validation output rather than performing a full cold CLI
  validation. A 3-iteration smoke showed the cached row now passes across
  fixtures, but cold `assura-check-cli` still does not meet the universal 2x
  LS-Lint target. Evidence:
  `target/performance/prepared-plan-smoke-2.json`.
- 2026-05-18: Prototyped a precompiled-config validation path using
  `assura-check-compile-config` plus `assura-check-compiled`. A direct bincode
  artifact was rejected because the YAML-facing Serde config uses
  `skip_serializing_if`, which is not safe for positional binary round-trips;
  the prototype now uses a field-named compiled artifact so the validation
  process skips YAML parsing while still traversing and validating the tree.
  The compiled validator binary is smaller than `assura-check` (about 867 KB
  vs 1.0 MB), but a 3-iteration smoke showed mixed results and no universal
  2x win, so the row is diagnostic-only. Evidence:
  `target/performance/compiled-config-smoke-3.json`.
- 2026-05-18: Added a direct project-root config discovery fast path for the
  common `assura-check --quiet` case where the current checked directory is the
  project root. A 3-iteration smoke showed this is only a small cleanup, not a
  decisive runtime change: cold `assura-check-cli` still measured roughly
  7.1-8.6 ms on the realistic rows while native LS-Lint measured roughly
  8.6-15.7 ms. Evidence: `target/performance/direct-root-smoke.json`.
- 2026-05-18: Added explicit 2x feasibility annotations to performance rows:
  `two_x_target_runtime_ms`, `process_floor_runtime_ms`,
  `process_floor_to_two_x_target_ratio`, `process_floor_blocks_two_x`,
  `runtime_to_two_x_target_ratio`, and `meets_two_x_target`. This makes the
  completion gate auditable by showing when local process startup alone already
  consumes the target runtime budget and how far each measured row remains from
  the 2x target.
- 2026-05-18: Extended the prepared-check architecture for editor-style hot
  validation. `PreparedStructureCheck` can now validate one changed path against
  the compiled config without whole-project traversal, and `assura-checkd`
  exposes that through `assura-check-client <ADDR> [PATH]`. The daemon also
  tracks config dirtiness separately from project dirtiness, so ordinary file
  edits do not force config reload/recompile. This is a foundation for
  incremental validation and is intentionally separate from the cold CLI 2x
  headline claim.
- 2026-05-18: Current 3-iteration smoke after the hot-path slice still shows
  the cold check-only CLI faster than native LS-Lint on the realistic equivalent
  rows, but not universally 2x. `assura-check-cli` versus native LS-Lint:
  `simple_library` 6.14 ms vs 9.59 ms, `web_app` 7.82 ms vs 9.48 ms,
  `monorepo_packages` 8.87 ms vs 9.01 ms, `monorepo_policy` 9.69 ms vs
  14.48 ms, `rule_heavy_repo` 7.17 ms vs 10.80 ms, and
  `ignored_generated_heavy_repo` 8.45 ms vs 19.08 ms. Evidence:
  `target/performance/hot-single-path-slice.json`.
- 2026-05-18: Replaced LS-Lint-compatible rule-heavy filename pattern scans
  with a compiled suffix index. This avoids checking every suffix rule for each
  file in broad extension-specific configs. A 15-iteration report after this
  change measured `rule_heavy_repo` in-process at 1.47 ms, with
  `walk-and-validate` at 1.13 ms; before this slice the same local report had
  measured that row at 5.38 ms in-process and 4.18 ms walk/validate. Evidence:
  `target/performance/suffix-index-immediate-exit-15.json` for phase rows. The
  final code left in tree measured `rule_heavy_repo` at 1.50 ms in-process and
  1.17 ms walk/validate in a 5-iteration smoke. Evidence:
  `target/performance/suffix-index-final-smoke.json`. A trial `_exit`
  quiet-success shortcut did not materially change the objective and was
  removed to avoid keeping unsafe code for weak evidence.
- 2026-05-18: Routed single-path `assura-check` runs through the direct
  non-batch `run_structure_check` path and removed timing instrumentation from
  the public non-timing wrapper. This keeps the ordinary check-only CLI path
  aligned with its API contract while preserving timed attribution for
  `performance-report`. A 5-iteration smoke remained noisy and did not change
  the completion status: only `ignored_generated_heavy_repo` met the 2x target.
  Evidence: `target/performance/single-check-fast-path-smoke.json`.
- 2026-05-18: Removed per-entry `PathBuf` allocation from the
  LS-Lint-compatible fast walker by using borrowed relative paths through the
  validation path and allocating only when emitting a violation. The 5-iteration
  smoke improved `monorepo_policy` cold `assura-check-cli` from 9.10 ms in the
  prior smoke to 8.23 ms, with `walk-and-validate` at 1.81 ms. This still does
  not satisfy universal 2x. Evidence:
  `target/performance/borrowed-fast-rel-smoke.json`.
- 2026-05-18: Tested broadening the LS-Lint-compatible fast path to cover
  direct allowlist/denylist/allow-extra policies so `monorepo_policy` could use
  the fast walker. The implementation preserved existing CLI behavior tests but
  worsened measured performance (`monorepo_policy` cold `assura-check-cli`
  10.23 ms, `walk-and-validate` 2.07 ms), so the experiment was removed.
  Evidence: `target/performance/direct-policy-fast-path-smoke.json`.
- 2026-05-18: Removed avoidable cold-path planning work for LS-Lint-compatible
  configs. The ordinary `StructureChecker::new` path now consumes the freshly
  compiled config instead of cloning it, and it skips building the full
  structure rule-scope plan when the conservative LS-Lint fast plan is
  available. Prepared/editor sessions still keep the full plan so
  single-path hot validation remains correct. Focused tests passed, and a
  5-iteration smoke still showed only `ignored_generated_heavy_repo` meeting
  the 2x target. Evidence:
  `target/performance/cold-owned-fast-plan-smoke.json`.
- 2026-05-18: Reworked `assura-check-compile-config` /
  `assura-check-compiled` from JSON config artifacts to a bincode artifact over
  a dedicated binary-safe portable config mirror. This avoids positional
  serialization of the YAML-facing structs with `skip_serializing_if` while
  making the artifact an actual binary representation. The compiled validator
  shrank to about 796 KB, but 5-iteration smokes remained mixed and still did
  not satisfy universal 2x; the row stays diagnostic until it stores and
  executes a true compiled validation plan instead of converting back through
  the runtime config. Evidence:
  `target/performance/bincode-compiled-config-smoke.json` and
  `target/performance/compiled-direct-root-smoke.json`.
- 2026-05-18: Split the compiled artifact and runtime compiled-config support
  into dedicated modules after `assura check` caught line-limit violations in
  `src/cli/check.rs` and `src/cli/check/prepared.rs`. Final validation passed.
  A current-tree 3-iteration smoke still showed only
  `ignored_generated_heavy_repo` meeting the 2x target for `assura-check-cli`;
  `monorepo_policy` is close but not over the line (`5.62 ms` vs a `5.35 ms`
  2x target in that smoke). Evidence:
  `target/performance/final-current-tree-smoke.json`.
- 2026-05-18: Changed `assura-check-compiled` to use a one-shot
  already-parsed-config execution path instead of going through
  `PreparedStructureCheck`, which is intentionally optimized for editor/daemon
  reuse and keeps the full rule plan. The compiled binary shrank to about
  780 KB and some compiled rows improved, but the row remains mixed and
  diagnostic-only. A later no-canonicalization direct-root experiment worsened
  the smoke and was removed. Evidence:
  `target/performance/compiled-one-shot-smoke.json` and
  `target/performance/direct-root-no-canonical-smoke.json`.
- 2026-05-18: Added a compiled `has_direct_count_constraints` flag so the full
  structure engine skips `validate_directory_contents` entirely when no
  `files.exists` or `directories.exists` count rules exist anywhere in the
  config. This avoids per-directory rule resolution for count checks that
  cannot fire. Phase evidence improved `monorepo_policy`
  `walk-and-validate` from 1.49 ms in the prior smoke to 1.40 ms, but the
  cold CLI row remained noisy and still did not meet universal 2x. Evidence:
  `target/performance/direct-count-guard-smoke.json`.
- 2026-05-18: Removed cloned project-root and exclusion-pattern state from the
  immutable LS-Lint fast walker filter. The same borrow pattern is not valid in
  the mutable full-engine walker, so only the fast-path allocation was removed.
  A 5-iteration smoke improved several phase rows (`simple_library`
  `walk-and-validate` 0.16 ms, `monorepo_policy` 1.12 ms) and `monorepo_policy`
  cold `assura-check-cli` measured 5.17 ms, but only
  `ignored_generated_heavy_repo` still met the 2x target. Evidence:
  `target/performance/borrowed-fast-filter-smoke.json`.
- 2026-05-18: Tested a single-project quiet-success status shortcut that avoided
  full report construction for LS-Lint-compatible passing configs. The 5-iteration
  smoke did not move the objective and was worse/noisy on several cold
  `assura-check-cli` rows (`web_app` 5.30 ms, `monorepo_policy` 7.56 ms), so
  the shortcut was removed rather than carrying duplicated validation logic.
  Dedicated quiet CLI regression tests were kept because they protect the public
  behavior independently of the experiment. Evidence:
  `target/performance/quiet-status-smoke.json`.
- 2026-05-18: Added source-config fingerprints to compiled config artifacts so
  `assura-check-compiled --config <PATH>` can reject stale binary artifacts
  before validation. This answers the config-change tracking requirement for
  compiled-config execution, but it does not change the headline cold
  `assura-check-cli` row. Also applied the existing direct-count guard to the
  LS-Lint-compatible fast walker so naming-only configs skip per-directory
  count-rule probes. A 5-iteration smoke remained below universal 2x; only
  `ignored_generated_heavy_repo` met the target. Evidence:
  `target/performance/config-fingerprint-smoke.json` and
  `target/performance/fast-count-guard-smoke.json`.
- 2026-05-18: Replaced `walkdir` with a scoped `std::fs::read_dir` recursion
  inside only the LS-Lint-compatible fast walker, leaving the full engine
  traversal unchanged. This improved some walk phases (`rule_heavy_repo`
  0.68 ms, `ignored_generated_heavy_repo` 0.07 ms in the smoke) but still did
  not clear universal 2x. Also skipped canonicalizing the already-absolute
  current directory on the no-path check invocation used by `assura-check
  --quiet`; the phase row improved but the cold CLI row stayed process/startup
  dominated. Evidence: `target/performance/std-fast-walk-smoke.json` and
  `target/performance/current-dir-fast-path-smoke.json`.
- 2026-05-18: Ran a 15-iteration current-tree audit after the fast traversal
  and current-dir discovery changes. `assura-check-cli` still failed universal
  2x on five of six realistic rows (`simple_library` 4.47 ms vs 3.12 ms
  target; `monorepo_policy` 5.85 ms vs 4.64 ms target), while
  `ignored_generated_heavy_repo` passed. Replaced the status-file reader's
  heap allocation with a fixed stack buffer for the editor-session status CLI;
  a smoke remained below universal 2x for status rows. Evidence:
  `target/performance/current-tree-15-audit.json` and
  `target/performance/status-fixed-buffer-smoke.json`.
- 2026-05-18: Tested a narrower `assura-check-quiet` binary using the same
  validation engine and only the exit-status workflow. It was smaller than
  `assura-check` but not faster in the 5-iteration smoke (`simple_library`
  4.79 ms, `monorepo_policy` 6.93 ms), so the experiment was removed instead
  of adding a worse product/report surface. Evidence:
  `target/performance/quiet-binary-smoke.json`.
- 2026-05-18: Re-tested a quiet success clean-probe after the fast walker
  rewrite by splitting the status-only code into a separate module and running
  a 5-iteration smoke. The result still did not meet the 2x gate and several
  rows were worse than the prior 15-iteration audit (`web_app` 4.70 ms,
  `monorepo_policy` 7.39 ms, `rule_heavy_repo` 5.83 ms), so the duplicate
  status-only path was removed. Evidence:
  `target/performance/quiet-clean-probe-smoke.json`.
- 2026-05-18: Re-audited the process-floor evidence after removing the
  clean-probe experiment. The process-floor row is `/usr/bin/true`, and in the
  current-tree audit the smallest fixture's process launch floor already
  exceeded the 2x target (`simple_library` 2.48 ms floor vs 2.26 ms target in
  the latest smoke). Assura's in-process phase rows remain well below target,
  so the remaining cold CLI gap is dominated by process launch rather than
  validation engine work. A fresh review agent could not be spawned because the
  session is still at the agent thread limit.
- 2026-05-18: Removed pipe/read overhead from measured subprocess loops in the
  performance harness. Measured iterations now use exit status with
  stdout/stderr sent to null for Assura, native LS-Lint, process-floor, hot
  client, and status-client rows. This makes the CLI comparison fairer but did
  not change completion status: `assura-check-cli` still passed only
  `ignored_generated_heavy_repo` in a 5-iteration smoke. Evidence:
  `target/performance/status-measurement-smoke.json`.
- 2026-05-18: Regenerated tracked history and website performance data with the
  status-based harness. `assura-check-cli` remains faster than native LS-Lint
  on all six realistic rows but clears 2x only on
  `ignored_generated_heavy_repo`. The daemon-backed status CLI was also
  reviewed as a possible editor-session answer, but it still missed four of six
  rows (`simple_library`, `web_app`, `monorepo_packages`, and
  `rule_heavy_repo`), so it is not a universal 2x CLI completion path either.
  Evidence: `benches/history/current.json`.
- 2026-05-18: Re-tested removing `assura-check` JSON/YAML output and cache
  support after the status-based harness correction. The binary shrank to
  864 KB, but the 5-iteration smoke still passed only
  `ignored_generated_heavy_repo` and regressed some rows (`simple_library`
  5.03 ms, `ignored_generated_heavy_repo` 4.55 ms), so the supported output
  and cache surface was restored. Evidence:
  `target/performance/no-output-cache-status-harness-smoke.json`.
- 2026-05-18: Tested replacing the `assura-check` hot parser with `pico-args`
  while preserving JSON output and cache support. The parser swap kept the
  binary at roughly 1.0 MB and still passed only `ignored_generated_heavy_repo`
  in the corrected 5-iteration smoke, with mixed/noisy row movement
  (`monorepo_packages` 5.17 ms, `monorepo_policy` 6.18 ms). The experiment was
  reverted to avoid adding a second parser dependency without a decisive win.
  Evidence: `target/performance/pico-args-smoke.json`.
- 2026-05-18: Tested an exact `assura-check --quiet` pre-parser that bypassed
  the general `lexopt` parser for the benchmarked no-path quiet invocation while
  still using the normal validation engine. The 5-iteration smoke was worse
  across the realistic rows and still passed only `ignored_generated_heavy_repo`,
  so the special case was removed. Evidence:
  `target/performance/quiet-preparse-smoke.json`.
- 2026-05-18: Added first-class cold-start feasibility attribution to the
  performance schema and regenerated tracked release evidence. Rows now record
  the generic process floor, the smallest Assura Rust CLI status-check floor,
  runtime above process floor, and Assura CLI overhead. The current tracked
  result still passes 2x only for `ignored_generated_heavy_repo`; on four of
  six realistic fixtures the Assura Rust CLI floor alone is above the 2x target,
  so a universal cold Rust subprocess claim is not supported by current
  evidence. Evidence: `benches/history/current.json` and
  `docs/analysis/2026-05-18-native-ls-lint-performance-gap-review.md`.
- 2026-05-18: Reworked the Unix `assura-check-status` client into a raw
  entrypoint status-file reader and added daemon-backed regression coverage.
  This shrank the status client from about 293 KB to 8.4 KB and improved the
  daemon/status execution mode, but it still did not meet universal 2x in the
  tracked 5-iteration release report. The one-shot `assura-check-cli` row still
  passes only `ignored_generated_heavy_repo`, while `assura-check-status-cli`
  passes three of six rows. Evidence: `crates/assura-check-cli/src/status.rs`,
  `crates/assura-check-cli/tests/batch_cli.rs`, and
  `benches/history/current.json`.
- 2026-05-18: Tested a raw Unix entrypoint for the benchmarked one-shot
  `assura-check --quiet` invocation so it could bypass Rust's normal `main`
  wrapper and the general `lexopt` parser on that exact path. The smoke was
  worse and still passed only `ignored_generated_heavy_repo`, so the experiment
  was reverted. Evidence: `target/performance/raw-check-main-smoke.json`.
- 2026-05-18: Tested Cargo package-level `opt-level = "z"` for `assura` and
  `assura-check-cli` to see whether a smaller one-shot binary would reduce
  cold startup enough to meet the target. The check binary only shrank
  marginally and the one-shot row was worse, so the profile override was
  reverted. Evidence: `target/performance/opt-z-smoke.json`.
- 2026-05-18: Reworked the Unix hot daemon client into a raw no-std Unix socket
  client and selected it for `assura-check-hot-cli` measurements on Unix. A
  15-iteration smoke showed the hot daemon client passing five of six rows,
  missing `web_app`, while the one-shot `assura-check-cli` still passed only
  `ignored_generated_heavy_repo`. Evidence:
  `crates/assura-check-cli/src/unix_client.rs` and
  `target/performance/nostd-unix-hot-client-15.json`.
- 2026-05-18: Compared direct `_exit` with returning through the C runtime for
  the no-std Unix status/client binaries. The return-path variant dropped
  hot/status smoke pass counts to three of six, so direct `_exit` was kept for
  those tiny clients. A separate quiet-success `_exit` attempt in the ordinary
  one-shot `assura-check` and compiled-config CLIs did not improve the
  one-shot objective and was reverted. Evidence:
  `target/performance/nostd-direct-exit-hot-client-smoke.json`,
  `target/performance/nostd-return-hot-client-smoke.json`, and
  `target/performance/quiet-fast-exit-smoke.json`.
- 2026-05-18: Regenerated tracked release history and website performance data
  after the raw no-std Unix client work. The current `assura-check-cli` row is
  faster than native LS-Lint on all six realistic rows and clears 2x on
  `monorepo_policy` plus `ignored_generated_heavy_repo`, but it still misses
  universal 2x on four rows. The daemon-backed hot client passes three of six
  rows, and the status-file client passes three of six, so neither is a complete
  universal 2x completion path either. Evidence:
  `benches/history/current.json`.
- 2026-05-18: Tested `opt-level = "s"` for the `assura` and
  `assura-check-cli` release packages as a less aggressive size-oriented
  variant than the earlier rejected `opt-level = "z"` experiment. The binary
  size did not materially improve and the one-shot realistic pass count dropped
  to one of six, so the override was reverted. Evidence:
  `target/performance/opt-s-smoke.json`.
- 2026-05-18: Replaced `libc` crate calls inside the raw Unix status and hot
  client binaries with direct C FFI declarations. This keeps both tiny clients
  at about 8.4 KB and links `libSystem` on macOS instead of `libiconv`, while
  preserving the package tests. The tracked release report still does not meet
  universal 2x: `assura-check-cli` passes one of six realistic rows,
  `assura-check-hot-cli` passes three of six, and
  `assura-check-status-cli` passes four of six. Evidence:
  `crates/assura-check-cli/src/status.rs`,
  `crates/assura-check-cli/src/unix_client.rs`, and
  `benches/history/current.json`.
- 2026-05-19: Added a prompt-to-artifact completion audit after repeated
  implementation and measurement slices. The audit maps each explicit
  requirement to concrete repo evidence and concludes that the requested
  universal cold-subprocess 2x outcome is still not achieved. The remaining
  misses are dominated by process/startup overhead rather than validation work:
  all `assura-in-process` rows pass 2x, while `assura-check-cli` passes one of
  six realistic rows. Evidence:
  `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`.
- 2026-05-19: Added `two_x_claim_status` to performance rows so Assura CLI
  misses distinguish ordinary implementation misses from rows where the
  measured process floor or smallest Rust CLI floor already exceeds the
  2x target. The website now renders that status for headline rows, and the
  refreshed 5-iteration tracked report still shows only
  `ignored_generated_heavy_repo` meeting the `assura-check-cli` 2x target.
  Evidence: `benches/history/current.json`.
- 2026-05-19: Changed compiled config artifacts from parsed-config-only
  bundles into schema-v3 artifacts that also carry portable full rule scopes and
  LS-Lint-compatible fast scopes. `assura-check-compiled` now consumes the
  precompiled artifact plan instead of rebuilding those scopes from the
  YAML-facing runtime config. The checked-in 5-iteration report remains below
  the universal 2x completion gate: `assura-check-cli` and
  `assura-check-compiled-cli` each pass only `ignored_generated_heavy_repo`.
  Evidence: `src/cli/check/compiled_artifact.rs` and
  `benches/history/current.json`.
- 2026-05-19: Aligned the raw Unix `assura-check-status` reader with the
  compact 14-byte status-file layout already used by the daemon-side shared
  writer. This fixes the binary status-file contract and preserves the
  daemon/status regression, but it remains editor-session evidence rather than
  proof of the cold one-shot CLI target. Evidence:
  `crates/assura-check-cli/src/status.rs`,
  `crates/assura-check-cli/src/status_file.rs`, and
  `target/performance/status-compact-smoke.json`.
- 2026-05-19: Reduced LS-Lint-compatible compiled artifacts by omitting the
  unused fallback full rule-scope plan from the default fast-path artifact and
  precomputing required-directory checks for the fast validator. The fallback
  full rule plan is rebuilt only when needed for fail-fast/full-rule execution.
  Targeted validation passed, but a 5-iteration release smoke still left the
  cold `assura-check-cli` and `assura-check-compiled-cli` rows short of the
  universal 2x completion gate. Evidence:
  `src/cli/check/compiled_plan_artifact.rs`,
  `src/cli/check/compiled_config.rs`, `src/cli/check/ls_fast.rs`, and
  `target/performance/compact-plan-smoke.json`.
- 2026-05-19: Extended the compiled artifact optimization so
  LS-Lint-compatible fast artifacts omit the portable fallback config entirely
  and execute from the compiled plan with an empty runtime config. The
  `assura-check` and `assura-check-compiled` measured binaries now use the
  established low-overhead `pico-args` parser instead of bespoke parsing or a
  heavier CLI surface. A 5-iteration release smoke still did not satisfy the
  universal cold CLI 2x gate: the headline `assura-check-cli` row passed only
  `monorepo_policy` and `ignored_generated_heavy_repo`, while
  `assura-check-compiled-cli` passed the same two rows. Evidence:
  `src/cli/check/compiled_artifact.rs`, `src/cli/check/artifact_check.rs`,
  `crates/assura-check-cli/src/main.rs`,
  `crates/assura-check-cli/src/compiled.rs`, and
  `target/performance/pico-check-smoke.json`.
- 2026-05-19: Investigated why the compiled-artifact binary still pays a large
  startup payload despite avoiding YAML parsing at runtime. Direct release
  binary inspection showed `serde_yaml` / `unsafe-libyaml` strings still linked
  into `assura-check-compiled`. Decoupling `ConfigError::Yaml` from
  `serde_yaml::Error` was validated but did not remove libyaml from the
  compiled binary. Candidate crate review found `serde_yaml_ng` is a maintained
  API-compatible fork but still uses `unsafe-libyaml`; `noyalib` is pure Rust
  but currently requires Rust 1.85, above Assura's declared 1.70 MSRV. This
  points the next serious slice toward feature-gating YAML-dependent check
  modules or splitting the compiled-artifact runtime into a separate no-YAML
  package, rather than swapping to another parser blindly. Evidence:
  `src/cli/config.rs`, `src/config/loader.rs`, `strings
  target/release/assura-check-compiled`, `cargo info serde_yaml_ng`, and
  `cargo info noyalib`.
- 2026-05-19: Simplified common LS-Lint regex naming conventions in the fast
  compiled plan so patterns such as `regex:^$`, `regex:^(README|AGENTS)$`, and
  grouped literal alternatives run as direct empty/exact/contains checks instead
  of runtime regexes. Fast-only compiled artifacts now retain only regex
  patterns that truly require regex evaluation. Focused tests and compiled CLI
  regressions passed, but the 5-iteration release smoke still missed the
  universal cold CLI 2x gate: `assura-check-compiled-cli` passed only
  `ignored_generated_heavy_repo`. Evidence:
  `src/cli/check/ls_fast_plan.rs`,
  `src/cli/check/compiled_plan_artifact.rs`,
  `src/cli/check/compiled_config.rs`, and
  `target/performance/regex-simplified-fast-smoke.json`.
- 2026-05-19: Reduced hot daemon request handling from one-byte reads to a
  buffered request read and added a default relative status-file path for
  `assura-check-status` so editor-session status checks can avoid passing an
  absolute status path on every invocation. Regression coverage passed and the
  daemon-backed CLI rows improved on several fixtures, but this remains
  diagnostic/editor-session evidence rather than completion of the cold
  comparable CLI objective: `target/performance/status-default-path-smoke.json`
  still shows `assura-check-cli` and `assura-check-compiled-cli` missing four
  of six realistic-equivalent rows. Evidence:
  `crates/assura-check-cli/src/server.rs`,
  `crates/assura-check-cli/src/status.rs`,
  `crates/assura-check-cli/tests/batch_cli.rs`,
  `src/cli/performance_report/hot_cli.rs`,
  `target/performance/daemon-buffered-read-smoke.json`, and
  `target/performance/status-default-path-smoke.json`.
- 2026-05-19: Replaced compiled-artifact version/path strings with a compact
  numeric Assura-version hash and source-config content hash, bumping the
  artifact schema and keeping stale-config rejection intact. This is a cleaner
  binary-artifact contract, but it did not materially reduce the compiled
  binary size or satisfy the cold CLI 2x gate: the release binary remained
  about 514 KB and `target/performance/compact-artifact-header-smoke.json`
  still shows `assura-check-compiled-cli` passing only
  `ignored_generated_heavy_repo`. Evidence:
  `src/cli/check/compiled_artifact.rs`,
  `crates/assura-check-cli/src/compiled.rs`, and
  `crates/assura-check-cli/tests/batch_cli.rs`.
- 2026-05-19: Rejected a direct simple-fast artifact runner experiment after
  measurement. The prototype bypassed `StructureChecker` for fast-only compiled
  artifacts, but it duplicated fast-validation logic, kept `regex-lite` linked,
  grew `assura-check-compiled` from about 514 KB to about 547 KB, and worsened
  the compiled CLI smoke. The code was backed out; the evidence remains as a
  negative result for future architecture work. Evidence:
  `target/performance/simple-fast-artifact-runner-smoke.json` and
  `target/release/assura-check-compiled` rebuilt back to about 514 KB.
- 2026-05-19: Split Assura's YAML/JSON surfaces behind `yaml-config` and
  `json-output` features so `cargo check -p assura --no-default-features`
  proves the structure-check core can compile without `serde_yaml`,
  `serde_json`, `notify`, or full CLI dependencies. A follow-on attempt to emit
  `assura-check-compiled` from a narrower package was rejected after
  measurement: the dependency tree was cleaner, but the stripped binary grew to
  about 519 KB and `target/performance/minimal-compiled-package-smoke.json`
  still passed only `ignored_generated_heavy_repo`. The narrower package and
  performance-build change were backed out; the feature boundary remains
  because it is neutral for the current measured binary and preserves a cleaner
  future split point. Evidence: `Cargo.toml`, `src/config/mod.rs`,
  `src/cli/check.rs`, `src/cli/mod.rs`,
  `cargo tree -p assura --no-default-features --edges normal`, and
  `cargo check -p assura --no-default-features`.
- 2026-05-19: Changed the hot status-file version guard from a runtime FNV loop
  to a compile-time constant in both the daemon writer and tiny status client.
  The focused status-file regression and clippy passed. A fresh 5-iteration
  release smoke still did not meet the universal 2x gate and was noisy enough
  that the status row passed three of six realistic-equivalent fixtures while
  cold `assura-check-compiled-cli` still passed only
  `ignored_generated_heavy_repo`. Evidence:
  `crates/assura-check-cli/src/status.rs`,
  `crates/assura-check-cli/src/status_file.rs`, and
  `target/performance/status-const-version-valid-smoke.json`.
- 2026-05-19: Removed the explicit Unix `close(2)` from the no-std
  `assura-check-status` success/error path because the process immediately
  exits via `_exit`, and refactored the LS-Lint fast walker to derive file
  validation names directly from `DirEntry::file_name` instead of allocating an
  absolute child path and stripping the project root for every file. Focused
  checks passed (`cargo check -p assura --no-default-features`, `cargo test -p
  assura ls_fast --quiet`, `cargo clippy -p assura-check-cli --bin
  assura-check-compiled --bin assura-check-status -- -D warnings`, and the
  status integration test). A 7-iteration release smoke improved several
  in-process walk rows, including `monorepo_policy` walk-and-validate at
  0.89 ms and `rule_heavy_repo` at 0.55 ms, but the cold compiled CLI still
  missed the universal 2x gate and the status row remained process-floor-bound
  on the tightest small fixtures. Evidence:
  `crates/assura-check-cli/src/status.rs`,
  `src/cli/check/ls_fast.rs`, and
  `target/performance/fast-walker-name-smoke.json`.
- 2026-05-19: Rejected a versioned clean-status marker experiment for the
  no-arg `assura-check-status` path. The prototype preserved binary
  status-file fallback semantics, but the measured status row became noisier
  and passed fewer fixtures, so the marker code was backed out. Evidence:
  `target/performance/status-marker-smoke.json`.
- 2026-05-19: Replaced fast naming suffix lookup with an ordered suffix list
  instead of a per-run `HashMap` and changed configured-directory membership
  from a per-process `HashSet<PathBuf>` to a sorted vector with binary search.
  This keeps the fast artifact setup smaller and avoids scanning each filename
  by character index for suffix-pattern rules. Focused no-default, fast-path,
  and compiled/status clippy checks passed. A 7-iteration release smoke kept
  `assura-check-compiled` at about 504 KB and improved the larger in-process
  rows (`monorepo_large` walk-and-validate at 8.93 ms and `rule_heavy` at
  3.98 ms), but cold `assura-check-compiled-cli` still passed only 2 of 11
  fixtures and therefore does not complete the universal 2x goal. Evidence:
  `src/cli/check/ls_fast_plan.rs`,
  `src/cli/check/compiled_plan_artifact.rs`,
  `src/cli/check/compiled_config.rs`,
  `src/cli/check.rs`,
  `src/cli/check/rules.rs`,
  `src/cli/check/validators.rs`, and
  `target/performance/fast-plan-compact-state-smoke.json`.
- 2026-05-19: Changed compiled exclusion matching to test `dir/**` exclusions
  as `Path` prefixes before falling back to glob string matching. This avoids
  converting every walked relative path into a slash-normalized string for the
  common prefix-exclusion case while preserving glob behavior for patterns such
  as `**/*.tmp`. Focused exclusion tests, no-default check, and compiled/status
  clippy passed. A 7-iteration release smoke kept `assura-check-compiled` at
  about 504 KB and improved the walk-heavy rows enough for `monorepo_large`
  `assura-check-compiled-cli` to meet 2x in that run, but cold compiled still
  passed only 2 of 11 fixtures. Evidence:
  `src/cli/check/rules.rs` and
  `target/performance/prefix-exclusion-fast-smoke.json`.
- 2026-05-19: Rejected stripping raw naming fields out of fast artifact
  effective bundles after measurement. The idea preserved fast-path semantics
  in focused tests, but the stripped binary grew to about 512 KB and the cold
  compiled row still passed only 2 of 11 fixtures, so the code was backed out.
  Evidence: `target/performance/fast-artifact-stripped-naming-smoke.json`.
- 2026-05-19: Reused the current directory's resolved fast rules while walking
  LS-Lint-compatible directories instead of resolving the active scope for every
  child file. This removes repeated scope scans in file-heavy directories while
  keeping directory validation and recursive child scopes intact. A 7-iteration
  release smoke kept `assura-check-compiled` at about 504 KB and improved
  walk-heavy rows (`monorepo_large` walk-and-validate at 8.33 ms and
  `rule_heavy` at 3.82 ms), but cold compiled still passed only 2 of 11
  fixtures. Evidence:
  `src/cli/check/ls_fast.rs` and
  `target/performance/fast-dir-rule-reuse-smoke.json`.
- 2026-05-19: Added a common-case default-artifact read path to
  `assura-check-compiled`: when invoked with no path and no explicit compiled
  artifact, it now tries to read `.assura/check-config.bin` from the current
  directory before falling back to project discovery. This avoids an extra
  config-discovery probe for the benchmarked default-root invocation and keeps
  fallback behavior for subdirectory or explicit-path use. The existing default
  artifact integration test and compiled clippy passed. The 7-iteration smoke
  remained mixed and did not complete the goal; cold compiled still passed only
  2 of 11 fixtures. Evidence:
  `crates/assura-check-cli/src/compiled.rs` and
  `target/performance/compiled-default-read-smoke.json`.
- 2026-05-19: Replaced `Path::file_stem` on fast-walker `DirEntry` filenames
  with a filename-only helper that preserves the relevant `Path` stem semantics
  for ordinary, dotted, hidden, and trailing-dot names. This removes per-file
  `Path` construction in the LS-Lint fast path. Focused stem tests and
  no-default check passed. A 7-iteration release smoke kept
  `assura-check-compiled` at about 504 KB and improved some walk-heavy rows
  (`monorepo_large` walk-and-validate at 8.04 ms), but cold compiled still
  passed only 2 of 11 fixtures. Evidence:
  `src/cli/check/ls_fast.rs` and
  `target/performance/fast-stem-helper-smoke.json`.
- 2026-05-19: Rejected making `StructureChecker` timing instrumentation
  optional in ordinary check paths. The prototype avoided `Instant::now()` when
  callers did not consume phase timings, but the 7-iteration release smoke
  regressed the current realistic-equivalent target from 2 of 6 to 1 of 6
  cold `assura-check-compiled-cli` rows meeting the 2x gate, including
  `monorepo_policy` moving from about 4.93 ms to about 6.08 ms. The timing
  API change was backed out and profiling remains the only consumer that
  reports phase timing values. Evidence:
  `target/performance/no-timing-ordinary-check-smoke.json`.
- 2026-05-19: Kept a compact Unix hot-client protocol for
  `assura-check-unix-client` and `assura-checkd`. The Unix client now sends a
  one-letter project-check request and the server returns a one-byte status for
  that compact request, while the existing text protocol remains available for
  the portable client and path-specific checks. Focused daemon client/status
  integration tests passed, including direct Unix-socket coverage for the
  compact client protocol. A 15-iteration release smoke improved the hard
  `rule_heavy_repo` hot row enough to meet the 2x gate (`3.04 ms` vs a
  `3.26 ms` target), but hot validation still met only 4 of 6
  realistic-equivalent rows because `simple_library` and `web_app` remained
  above target. This is useful hot-session progress, not completion evidence
  for the cold comparable CLI goal. Evidence:
  `crates/assura-check-cli/src/unix_client.rs`,
  `crates/assura-check-cli/src/server.rs`, and
  `target/performance/hot-compact-protocol-smoke-2.json`.
- 2026-05-19: Rejected a relative-path default artifact branch for
  `assura-check-compiled --quiet`. The prototype avoided `current_dir()` for
  the benchmarked no-argument default artifact shape by using `.` and
  `.assura/check-config.bin`, but the release smoke was mixed: the compiled
  binary grew to about 506 KB and cold `assura-check-compiled-cli` met the 2x
  gate on only one of six realistic-equivalent rows. The branch was backed out;
  the existing default-artifact path remains the lower-risk measured route.
  Evidence: `target/performance/compiled-relative-default-smoke.json`.
- 2026-05-19: Rejected simplifying the raw Unix `assura-check-status` reader
  from a short read loop to a single `read` call. The change reduced the
  status client's `__text` section slightly, but a 7-iteration release smoke
  regressed the status row from the tracked 5 of 6 realistic-equivalent 2x
  passes to 4 of 6. The loop was restored because status-file read mechanics
  are not the remaining bottleneck. Evidence:
  `target/performance/status-single-read-smoke.json`.
- 2026-05-19: Hardened performance-row classification so daemon-backed
  `assura-check-hot-cli` and `assura-check-status-cli` rows are explicitly
  covered by the diagnostic-row regression test. This preserves the audit rule
  that near-miss hot/status evidence must not be accepted as headline
  completion evidence for the cold comparable CLI objective. Evidence:
  `src/cli/performance_report/rows.rs` and `cargo test -p assura
  synthetic_and_diagnostic_families_are_not_headline_rows --quiet`.
- 2026-05-19: Strengthened the hot/editor changed-path path so
  `PreparedStructureCheck::check_changed_path` now reuses the validated
  compiled config, rechecks configured structure requirements, and validates
  the changed path's direct parent aggregate rules without walking the whole
  project. Deleted paths are resolved through their nearest existing ancestor
  so file removals can invalidate parent `exists_count` policies instead of
  returning `MissingPath`. This is incremental-session architecture progress,
  not cold subprocess 2x completion evidence. Evidence:
  `src/cli/check/prepared.rs`, `src/cli/check/traversal.rs`,
  `cargo test -p assura prepared_check --quiet`,
  `cargo test -p assura-check-cli --test batch_cli --quiet`, and
  `cargo clippy -p assura-check-cli --bin assura-checkd --bin
  assura-check-unix-client -- -D warnings`.
- 2026-05-19: Split the stable check report/error types out of the
  check-engine coordinator into `src/cli/check/report.rs`, preserving the
  public `assura::cli::{CheckError, StructureCheckReport}` exports while
  bringing `src/cli/check.rs` back under the Assura line-limit policy. This is
  validation hygiene for the performance branch, not a new speed claim.
  Evidence: `src/cli/check.rs`, `src/cli/check/report.rs`, `wc -l
  src/cli/check.rs src/cli/check/report.rs`, and `target/release/assura-check
  --quiet docs/archive/assura-native-ls-lint-performance-rearchitecture.md
  src/cli/check.rs src/cli/check/report.rs src/cli/check/prepared.rs
  src/cli/check/traversal.rs`.
- 2026-05-19: Split LS-Lint fast naming compilation/matching into
  `src/cli/check/ls_fast_naming.rs` and moved fast-plan unit tests to
  `src/cli/check/ls_fast_plan_tests.rs`, keeping `src/cli/check/ls_fast_plan.rs`
  focused on scope compilation and below the Assura line-limit policy. The
  repo self-check now passes for the full checkout again. Evidence:
  `src/cli/check/ls_fast_plan.rs`, `src/cli/check/ls_fast_naming.rs`,
  `src/cli/check/ls_fast_plan_tests.rs`, `wc -l src/cli/check.rs
  src/cli/check/report.rs src/cli/check/ls_fast_plan.rs
  src/cli/check/ls_fast_naming.rs src/cli/check/ls_fast_plan_tests.rs`,
  `cargo test -p assura ls_fast --quiet`, and
  `target/release/assura-check --quiet .`.
- 2026-05-19: Rebuilt release binaries after the incremental-path and module
  split work and ran a 3-iteration native-binary comparison smoke. The smoke
  confirmed the branch remains blocked for the cold comparable CLI goal:
  `target/performance/post-incremental-split-smoke.json` reports
  `two_x_claim_verdict=not-complete`, `two_x_pass_count=2`, and aggregate
  speedup `1.57x` for the `assura-check-cli` headline row. This evidence
  supports keeping the checked-in `not-complete` verdict rather than updating
  benchmark history. Evidence: `cargo build --release -p assura --bin assura`,
  `cargo build --release -p assura-check-cli`, and
  `target/performance/post-incremental-split-smoke.json`.
- 2026-05-19: Removed the explicit Unix `close(2)` call from the raw
  `assura-check-unix-client` hot-client path, matching the existing
  `assura-check-status` pattern where the process exits immediately through
  `_exit`. Focused clippy and daemon integration tests passed, and the
  release smoke kept the tiny hot client at about 8.4 KB, but the cold
  headline gate remains `not-complete`: `target/performance/hot-client-no-close-smoke.json`
  reports `two_x_claim_verdict=not-complete` and `two_x_pass_count=1` for
  `assura-check-cli`. Evidence: `crates/assura-check-cli/src/unix_client.rs`,
  `cargo clippy -p assura-check-cli --bin assura-check-unix-client -- -D
  warnings`, `cargo test -p assura-check-cli --test batch_cli --quiet`, and
  `target/performance/hot-client-no-close-smoke.json`.
- 2026-05-19: Ran the broad local correctness gates after the performance,
  incremental-validation, and module-boundary work. The branch now passes full
  Rust tests, all-target/all-feature clippy, repo self-check, and whitespace
  checks. This is readiness evidence for the implementation, not proof of the
  cold 2x target. Evidence: `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `target/release/assura-check --quiet .`, and `git diff --check`.
- 2026-05-19: Rechecked whether package-level dependency weight still implied
  a useful binary-splitting opportunity. `cargo tree -p assura-check-cli
  --edges normal` still lists YAML, JSON, notify, and parser dependencies
  because the package owns multiple binaries, but release binary inspection
  shows the relevant artifacts link only `libSystem`; `strings` on
  `assura-check-compiled` no longer shows `serde_yaml`, `unsafe-libyaml`,
  `notify`, or `serde_json`, and the raw hot/status clients remain tiny
  no-std binaries. This makes another package split a poor next bet without
  new measurements. The performance skill now records this guardrail. Evidence:
  `.agents/skills/assura-performance-reporting/SKILL.md`, `cargo tree -p
  assura-check-cli --edges normal`, `otool -L target/release/assura-check
  target/release/assura-check-compiled target/release/assura-check-unix-client
  target/release/assura-check-status`, `strings target/release/assura-check-compiled
  | rg "serde_yaml|unsafe-libyaml|notify|crossbeam|serde_json"`, and `size
  target/release/assura-check target/release/assura-check-compiled
  target/release/assura-check-unix-client target/release/assura-check-status`.
- 2026-05-19: Strengthened compiled config artifact invalidation for the
  incremental/config-cache path. Artifacts now include canonical project-root
  and source-config-path fingerprints in addition to schema/version/config-byte
  guards, and `assura-check-compiled` rejects artifacts used against a different
  project root or a different source config path even when the YAML bytes match.
  The incremental strategy note was updated from "no cache implemented yet" to
  the current partial-foundation state so future agents do not repeat stale
  analysis. Evidence: `src/cli/check/compiled_artifact.rs`,
  `crates/assura-check-cli/src/compiled.rs`,
  `crates/assura-check-cli/src/compile_config.rs`,
  `crates/assura-check-cli/tests/compiled_config_cli.rs`,
  `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`,
  and `cargo test -p assura-check-cli --test compiled_config_cli --quiet`.
- 2026-05-19: Hardened the opt-in cached check result freshness check so it no
  longer relies on directory modified time alone. Cached directory snapshots now
  include direct child count plus a stable fingerprint of each child name and
  file type, which better matches the incremental strategy's directory-level
  dependency model and reduces stale-cache risk on coarse timestamp file
  systems. Evidence: `src/cli/check/cache.rs`,
  `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`,
  `cargo test -p assura child_fingerprint --quiet`, and `cargo test -p
  assura-check-cli --test batch_cli
  cache_dir_reuses_report_but_invalidates_on_directory_change --quiet`.
- 2026-05-19: Tightened changed-path benchmark attribution for the
  editor-session path. The row now chooses a deterministic candidate path,
  ignores generated/config/status files during candidate selection, and dirties
  that file after the daemon is warm so the measured client request better
  represents an edit-loop validation. This remains diagnostic-only evidence and
  is not counted toward the cold CLI headline claim. Evidence:
  `src/cli/performance_report/changed_path_cli.rs`.
- 2026-05-19: Aligned cached-result freshness checks with ignored-path pruning.
  Cached check reports now store the config exclude patterns and use them while
  rebuilding directory snapshots, so warm cache validation does not recurse into
  ignored generated output and ignored-path churn does not invalidate an
  otherwise fresh LS-Lint-compatible report. Evidence: `src/cli/check/cache.rs`
  and `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`.
- 2026-05-19: Added an explicit `validation_execution_mode` field to generated
  performance rows so cold CLI, warm cache, precompiled-config, hot daemon,
  changed-path, status-file, in-process, phase, traversal, and diagnostic
  strategy evidence cannot be conflated during future audits. This is evidence
  hygiene and does not change the headline claim gate, which remains tied to
  cold `assura-check-cli` versus native `ls-lint-cli`. Evidence:
  `src/cli/performance_report/rows.rs`,
  `benches/history/ls-lint-comparison.schema.json`, and
  `website/src/components/performance-evidence.astro`.
- 2026-05-19: Backfilled `validation_execution_mode` into checked-in current
  and historical performance data for both benchmark history and website data,
  and added a contract test that verifies every checked-in row carries the
  expected execution mode. The public website still builds from the updated
  data. Evidence: `benches/history/current.json`,
  `website/public/data/performance/current.json`,
  `benches/history/ls-lint-comparison-history.jsonl`,
  `website/public/data/performance/ls-lint-comparison-history.jsonl`,
  `tests/performance_report_contract_tests.rs`, and `pnpm --dir website build`.
- 2026-05-19: Tightened the opt-in cached check path's config-dirty behavior.
  Cache misses now parse with semantic config validation, so an unchanged config
  can reuse a fresh report while a modified config is revalidated before any
  report reuse. Added CLI regression coverage proving that a valid config edit
  recomputes against the new rule set, a changed invalid config is not hidden by
  a previously successful cache entry, and corrupt cache entries fall back to a
  real check. This is incremental/cache correctness work, not evidence that the
  cold CLI 2x gate is complete. Evidence: `src/cli/check/cache.rs`,
  `crates/assura-check-cli/tests/batch_cli.rs`,
  `cargo test -p assura-check-cli --test batch_cli cache_dir --quiet`,
  `cargo test -p assura-check-cli --test batch_cli --quiet`,
  `cargo test -p assura directory_snapshot_prunes_excluded_children --quiet`,
  `cargo clippy -p assura --lib -- -D warnings`, `cargo clippy -p
  assura-check-cli --test batch_cli -- -D warnings`, and
  `target/release/assura-check --quiet src/cli/check/cache.rs
  crates/assura-check-cli/tests/batch_cli.rs`.
- 2026-05-19: Removed unnecessary parent-process cwd setup from the diagnostic
  status-file performance row. The row now invokes
  `assura-check-status <status-file>` directly instead of changing into the
  status directory and relying on the default relative filename, matching the
  already-supported CLI surface and avoiding an avoidable measurement artifact.
  A 3-iteration release smoke still left the goal blocked:
  `target/performance/status-explicit-path-smoke.json` reports
  `two_x_claim_verdict=not-complete`, with the cold `assura-check-cli` row
  meeting 2x on only one of six realistic-equivalent fixtures and the
  status-file row still missing 2x on the tiny fixtures. Evidence:
  `src/cli/performance_report/hot_cli.rs`,
  `cargo test -p assura-check-cli --test batch_cli
  status_cli_reads_clean_daemon_status_file --quiet`,
  `cargo test -p assura validation_execution_mode --quiet`, `cargo clippy -p
  assura --lib -- -D warnings`, `cargo build --release -p assura --bin
  assura`, `cargo build --release -p assura-check-cli`, and
  `target/performance/status-explicit-path-smoke.json`.
- 2026-05-19: Re-tested replacing deprecated `serde_yaml` with `serde_yml` via
  a crate alias so the check-only binary could keep the existing Serde-facing
  loader API while using a maintained parser package. The candidate compiled
  and passed focused compatibility tests, but it was rejected because it grew
  `target/release/assura-check` from about 1.08 MB to about 1.10 MB, retained
  a libyaml-style parser payload (`libyml` strings), and regressed the
  3-iteration release smoke: `target/performance/serde-yml-parser-smoke.json`
  still reported `two_x_claim_verdict=not-complete` with one of six headline
  2x passes. The dependency change was reverted to the retained `serde_yaml`
  parser. Evidence: temporary `Cargo.toml` / `crates/assura-check-cli/Cargo.toml`
  alias test, `cargo check -p assura-check-cli`, `cargo check -p assura
  --no-default-features`, `cargo test -p assura-check-cli --test batch_cli
  --quiet`, `cargo test -p assura-check-cli --test compiled_config_cli
  --quiet`, `cargo test -p assura --lib ls_fast --quiet`, `cargo clippy -p
  assura --lib -- -D warnings`, `cargo clippy -p assura-check-cli --bins
  --tests -- -D warnings`, `size target/release/assura-check
  target/release/assura-check-compiled target/release/assura-check-compile-config`,
  `strings target/release/assura-check | rg
  "serde_yaml|unsafe-libyaml|serde_yml|libyml|pico-args"`, and
  `target/performance/serde-yml-parser-smoke.json`.
- 2026-05-19: Fixed a retained LS-Lint-compatible fast-path correctness and
  pruning gap: direct `exists` count checks now apply configured excludes before
  counting direct child files/directories. This prevents ignored files from
  making count checks fail and avoids spending count work on excluded generated
  entries. Added CLI regression coverage for a root `*.ts: "1"` count where an
  ignored sibling `.ts` file must not be counted. Evidence:
  `src/cli/check/ls_fast.rs`,
  `crates/assura-check-cli/tests/batch_cli.rs`, `cargo fmt --all -- --check`,
  `cargo test -p assura-check-cli --test batch_cli
  check_fast_exists_counts_prune_excluded_children --quiet`, `cargo test -p
  assura-check-cli --test batch_cli --quiet`, `cargo test -p assura --lib
  ls_fast --quiet`, `cargo clippy -p assura --lib -- -D warnings`, and
  `target/performance/fast-count-exclude-smoke.json`. The smoke still reported
  `two_x_claim_verdict=not-complete` with one of six headline 2x passes, so
  checked-in benchmark history was not refreshed.
- 2026-05-19: Applied the same excluded-child count semantics to the full
  structure checker's direct-content path. Full non-fast checks now prune
  configured excludes before evaluating direct file and directory `exists`
  counts, matching the LS-Lint-compatible fast path and avoiding stale
  full-versus-fast behavior differences. Added a full CLI regression that
  forces the non-fast path with `max_lines` and proves excluded files and
  excluded directories are not counted. Evidence:
  `src/cli/check/direct_contents.rs`, `tests/direct_count_exclusion_tests.rs`,
  `tests/cli_check_tests.rs`,
  `cargo test -p assura --test cli_check_tests
  check_validates_file_and_directory_exists_counts --quiet`, `cargo test -p
  assura --test cli_check_tests check_supports_wildcard_extension_rules
  --quiet`, `cargo test -p assura --test direct_count_exclusion_tests
  --quiet`, `cargo test -p assura --test ls_lint_parity_regression_tests
  exists --quiet`, `cargo test -p assura-check-cli --test batch_cli
  check_fast_exists_counts_prune_excluded_children --quiet`, `cargo fmt --all
  -- --check`, and `cargo clippy -p assura --lib -- -D warnings`.
- 2026-05-19: Rejected a compiled `has_configured_structure_constraints` flag
  intended to skip `validate_configured_structure` for naming/count-only
  configs. The candidate was architecturally plausible and passed targeted
  tests, but it increased `target/release/assura-check` by one page, left the
  release smoke at `two_x_claim_verdict=not-complete` with one of six headline
  2x passes, and did not materially improve phase timings because the
  configured-structure phase was already near zero on affected fixtures. The
  code and artifact schema bump were removed. Evidence:
  `target/performance/configured-structure-skip-smoke.json`, `cargo test -p
  assura-check-cli --test compiled_config_cli --quiet`, `cargo test -p assura
  prepared_check --quiet`, `cargo test -p assura --test cli_check_tests
  check_fails_missing_required_file --quiet`, `cargo clippy -p assura --lib
  -- -D warnings`, and `size target/release/assura-check
  target/release/assura-check-compiled target/release/assura-check-compile-config`.
- 2026-05-19: Reduced duplicated work in the full direct-content count path.
  Full validation now scans each directory once for direct child file and
  directory names, applies configured excludes once, and reuses those child
  lists for both file `exists` and directory `exists` count constraints. This
  aligns the full path with the already-specialized fast path shape without
  changing count semantics or checked-in benchmark history. A 3-iteration
  release smoke still reported `two_x_claim_verdict=not-complete`, with the
  headline `assura-check-cli` row meeting 2x on one of six fixtures. Evidence:
  `src/cli/check/direct_contents.rs`,
  `cargo test -p assura --test direct_count_exclusion_tests --quiet`,
  `cargo test -p assura --test cli_check_tests
  check_validates_file_and_directory_exists_counts --quiet`, `cargo test -p
  assura --test ls_lint_parity_regression_tests exists --quiet`, `cargo test
  -p assura-check-cli --test batch_cli
  check_fast_exists_counts_prune_excluded_children --quiet`, `cargo fmt
  --all -- --check`, `cargo clippy -p assura --lib -- -D warnings`, `cargo
  build --release -p assura-check-cli`, `target/release/assura-check --quiet
  src/cli/check/direct_contents.rs tests/direct_count_exclusion_tests.rs
  docs/archive/assura-native-ls-lint-performance-rearchitecture.md`, and
  `target/performance/direct-count-one-pass-smoke.json`, and `git diff
  --check`.
- 2026-05-19: Reduced duplicated traversal work in the LS-Lint-compatible fast
  path for direct count rules. Count-bearing directories now derive direct
  child file/directory count inputs from the same `read_dir` pass used for
  traversal, while count-free directories keep the previous streaming path to
  avoid materializing entries unnecessarily. Split the count helper into
  `src/cli/check/ls_fast_counts.rs` so `src/cli/check/ls_fast.rs` stays under
  the Assura file-size policy. A 3-iteration release smoke still reported
  `two_x_claim_verdict=not-complete`; current misses are concentrated in
  traversal-heavy rows or small rows dominated by process/CLI floor rather than
  config validation. Evidence: `src/cli/check/ls_fast.rs`,
  `src/cli/check/ls_fast_counts.rs`, `src/cli/check.rs`, `cargo test -p
  assura-check-cli --test batch_cli
  check_fast_exists_counts_prune_excluded_children --quiet`, `cargo test -p
  assura --lib ls_fast --quiet`, `cargo test -p assura --test
  ls_lint_parity_regression_tests exists --quiet`, `cargo test -p
  assura-check-cli --test batch_cli --quiet`, `cargo clippy -p assura --lib
  -- -D warnings`, `cargo build --release -p assura-check-cli`,
  `cargo build --release -p assura --bin assura`,
  `target/release/assura-check --quiet src/cli/check/ls_fast.rs
  src/cli/check/ls_fast_counts.rs src/cli/check/direct_contents.rs
  tests/direct_count_exclusion_tests.rs
  docs/archive/assura-native-ls-lint-performance-rearchitecture.md`,
  `target/performance/fast-count-one-pass-smoke.json`, and `git diff --check`.
- 2026-05-19: Tightened the fast-count helper after the one-pass traversal
  refactor by removing duplicate child-name vectors for count-bearing
  directories. Count validation now reuses the materialized fast directory
  entries directly, avoiding extra `String` clones while preserving the
  count-free streaming traversal path. The check-only binary text segment
  dropped to about 1.077 MB after the cleanup. A 3-iteration release smoke
  still reported `two_x_claim_verdict=not-complete`, with only
  `ignored_generated_heavy_repo` satisfying the realistic-equivalent 2x claim;
  checked-in benchmark history was not refreshed. Evidence:
  `src/cli/check/ls_fast.rs`, `src/cli/check/ls_fast_counts.rs`, `cargo fmt
  --all -- --check`, `cargo test -p assura-check-cli --test batch_cli
  check_fast_exists_counts_prune_excluded_children --quiet`, `cargo test -p
  assura --lib ls_fast --quiet`, `cargo test -p assura --test
  ls_lint_parity_regression_tests exists --quiet`, `cargo test -p
  assura-check-cli --test batch_cli --quiet`, `cargo clippy -p assura --lib
  -- -D warnings`, `cargo build --release -p assura-check-cli`, `cargo build
  --release -p assura --bin assura`, `size target/release/assura-check
  target/release/assura-check-compiled target/release/assura-check-compile-config`,
  `target/release/assura-check --quiet src/cli/check/ls_fast.rs
  src/cli/check/ls_fast_counts.rs src/cli/check/direct_contents.rs
  tests/direct_count_exclusion_tests.rs
  docs/archive/assura-native-ls-lint-performance-rearchitecture.md`,
  `target/performance/fast-count-entry-reuse-smoke.json`, and `git diff
  --check`.
- 2026-05-19: Rejected a dot-suffix lookup map for LS-Lint fast filename
  naming. The candidate replaced the ordered suffix scan for patterns such as
  `*.test.ts` and `*.kind-05.ts` with a filename dot-position lookup and kept
  the old scan as a fallback for non-dot suffixes. It passed focused LS fast,
  batch, parity, and clippy gates, but grew `target/release/assura-check` by
  one page and did not improve the 3-iteration release smoke enough to affect
  the universal 2x verdict. The change was reverted rather than keeping more
  startup/binary-size pressure for noisy rule-heavy movement. Evidence:
  temporary `src/cli/check/ls_fast_naming.rs` experiment,
  `target/performance/fast-dot-suffix-smoke.json`, `size
  target/release/assura-check target/release/assura-check-compiled
  target/release/assura-check-compile-config`, `cargo test -p assura --lib
  ls_fast --quiet`, `cargo test -p assura-check-cli --test batch_cli --quiet`,
  `cargo test -p assura --test ls_lint_parity_regression_tests exists
  --quiet`, and `cargo clippy -p assura --lib -- -D warnings`.
- 2026-05-19: Confirmed the remaining small-fixture misses are bounded by the
  subprocess/runtime floor rather than config validation or check logic alone.
  The `assura-check-status` diagnostic binary is already a raw Unix status
  reader with a 4 KB text segment and no linked YAML, JSON, regex, glob, or
  argument-parser payload, yet the latest 3-iteration release smoke still had
  `assura-check-status-cli` above the 2x target for `simple_library` and
  `web_app`. This makes further cold-start micro-binary work a poor fit for
  the universal claim; the product path should shift toward explicit compiled
  config, hot daemon/status, and changed-path incremental modes with separate
  claims. Evidence: `target/performance/fast-count-entry-reuse-smoke.json`,
  `size target/release/assura-check-status target/release/assura-check-unix-client
  target/release/assura-check`, `otool -L target/release/assura-check-status`,
  and `strings target/release/assura-check-status | rg
  "serde|yaml|json|regex|pico|lexopt|notify|assura"`.
- 2026-05-19: Added conservative dirty-path tracking to `assura-checkd` so a
  project check after safe file-level watcher events can reuse a previously
  clean full-project result and validate only the changed files plus their
  direct parent aggregate rules. The daemon still falls back to a full project
  check for config changes, directory events, ambiguous watcher events, too
  many accumulated paths, or a previously failing project result. This advances
  the editor-session/incremental path without adding weight to the cold
  `assura-check` binary. Validation passed: `cargo fmt --all -- --check`,
  `cargo test -p assura-check-cli --bin assura-checkd --quiet`, `cargo test
  -p assura-check-cli --test batch_cli
  hot_client_can_validate_one_changed_path_without_project_check --quiet`,
  `cargo test -p assura-check-cli --test batch_cli --quiet`, `cargo clippy
  -p assura-check-cli --bins --tests -- -D warnings`, `cargo build --release
  -p assura-check-cli`, and `cargo build --release -p assura --bin assura`.
  A 3-iteration release smoke at
  `target/performance/incremental-dirty-path-project-smoke.json` still reports
  `claim_summary.two_x_claim_verdict=not-complete`; the headline cold
  `assura-check-cli` row met 2x on one of six realistic-equivalent fixtures,
  while hot/status rows remained near the subprocess floor on the smallest
  fixtures.
- 2026-05-19: Rejected adding a watcher-driven
  `assura-check-dirty-project-cli` performance row for now. The candidate row
  tried to mark a fixture file, wait for `assura-checkd` to write a dirty
  status file, and then time a project check that reused dirty-path tracking.
  On this macOS smoke every fixture timed out waiting for the watcher dirty
  status, so keeping the row would add skipped/noisy evidence rather than a
  reliable benchmark. The implementation was removed from the reporting path;
  explicit changed-path CLI evidence remains the stable incremental diagnostic
  row. Evidence: rejected `src/cli/performance_report/changed_path_cli.rs`
  experiment and `target/performance/dirty-project-row-smoke.json`.
- 2026-05-19: Added an explicit dirty-path project check protocol for editor
  integrations that already know which path changed. `assura-check-client
  --dirty-project-path <PATH>` sends `CHECK-DIRTY-PROJECT-PATH` to
  `assura-checkd`; the daemon reuses the previous clean project result by
  validating that path and its direct parent aggregate rules, then updates the
  cached project/status-file exit code. If config changed, the prior project
  was failing, or the daemon lacks a clean cached result, it falls back to a
  full project check. This keeps the watcher-based behavior as an opportunistic
  fast path while giving editors a deterministic incremental project-status
  interface. Evidence: `crates/assura-check-cli/src/client.rs`,
  `crates/assura-check-cli/src/server.rs`,
  `crates/assura-check-cli/src/server_dirty.rs`, `cargo test -p
  assura-check-cli --test batch_cli
  hot_client_can_check_project_from_explicit_dirty_path --quiet`, `cargo test
  -p assura-check-cli --test batch_cli --quiet`, and `cargo clippy -p
  assura-check-cli --bins --tests -- -D warnings`.
- 2026-05-19: Added a deterministic
  `assura-check-dirty-project-cli` diagnostic performance row for the explicit
  dirty-path project-status protocol. Unlike the rejected watcher-driven row,
  this row warms the daemon with a clean project check, mutates a deterministic
  fixture file, then times `assura-check-client --dirty-project-path <PATH>`.
  The row emitted pass measurements for every generated fixture in the
  3-iteration release smoke and is labeled
  `hot-daemon-dirty-project-cli`/diagnostic so it cannot affect the cold
  headline claim. The smoke at
  `target/performance/explicit-dirty-project-row-smoke.json` still reports
  `claim_summary.two_x_claim_verdict=not-complete`; the cold
  `assura-check-cli` headline row met 2x on one of six realistic-equivalent
  fixtures. Evidence: `src/cli/performance_report/changed_path_cli.rs`,
  `src/cli/performance_report/fixture_rows.rs`,
  `src/cli/performance_report/rows.rs`,
  `tests/performance_report_contract_tests.rs`, `cargo test -p assura --lib
  performance_report::rows_tests --quiet`, `cargo test -p assura --test
  performance_report_contract_tests history_rows_include_execution_mode_metadata
  --quiet`, `cargo clippy -p assura --lib -- -D warnings`, `cargo build
  --release -p assura-check-cli`, `cargo build --release -p assura --bin
  assura`, and `target/performance/explicit-dirty-project-row-smoke.json`.
- 2026-05-19: Hardened the explicit dirty-path daemon protocol so it does not
  discard pending watcher-derived dirty state. `CHECK-DIRTY-PROJECT-PATH` now
  falls back to a full project check when the daemon has a full dirty state,
  preserves and combines pending file-path dirty state with the explicit path,
  and still uses the explicit path alone when the daemon was otherwise clean.
  Evidence: `crates/assura-check-cli/src/server.rs`, `cargo test -p
  assura-check-cli --bin assura-checkd --quiet`, and `cargo test -p
  assura-check-cli --test hot_cli --quiet`.
- 2026-05-19: Tightened the raw Unix hot-client protocol for explicit
  dirty-project checks. `assura-check-unix-client --dirty-project-path <PATH>`
  now sends compact `D\t<PATH>\n`, and `assura-checkd` answers that compact
  request with the same single-byte status response used by compact project
  checks. The readable `CHECK-DIRTY-PROJECT-PATH` protocol remains accepted for
  the portable client. This keeps the editor-session path close to the process
  floor, but it does not change the cold headline outcome. The release smoke at
  `target/performance/compact-dirty-protocol-v3-smoke.json` reports
  `claim_summary.two_x_claim_verdict=not-complete`; cold `assura-check-cli`
  still met 2x on only one of six realistic-equivalent fixtures, while the
  dirty-project diagnostic row met 2x on three of six. Evidence:
  `crates/assura-check-cli/src/server.rs`,
  `crates/assura-check-cli/src/unix_client.rs`, `cargo fmt --all -- --check`,
  `cargo test -p assura-check-cli --bin assura-checkd --quiet`, `cargo test -p
  assura-check-cli --test hot_cli --quiet`, `cargo clippy -p
  assura-check-cli --bins --tests -- -D warnings`, `cargo build --release -p
  assura-check-cli`, and
  `target/performance/compact-dirty-protocol-v3-smoke.json`.
- 2026-05-19: Hardened `assura-checkd` so watcher events caused by its own
  status-file writes do not dirty the project. This makes in-tree status-file
  placement safe for callers that explicitly choose it. A candidate benchmark
  change that read the default relative `assura-check.status` from the fixture
  root was rejected because `target/performance/default-status-file-smoke.json`
  regressed the status diagnostic and still met 2x on only three of six
  realistic-equivalent fixtures. The retained release smoke at
  `target/performance/status-ignore-retained-smoke.json` still reports
  `claim_summary.two_x_claim_verdict=not-complete`; cold `assura-check-cli`
  met 2x on one of six realistic-equivalent fixtures. Evidence:
  `crates/assura-check-cli/src/server.rs`, `cargo fmt --all -- --check`,
  `cargo test -p assura-check-cli --bin assura-checkd --quiet`,
  `cargo clippy -p assura-check-cli --bins --tests -- -D warnings`, `cargo
  build --release -p assura-check-cli`, `cargo build --release -p assura --bin
  assura`, and `target/performance/status-ignore-retained-smoke.json`.
- 2026-05-19: Rejected versioned status marker files as a status-client fast
  path. The candidate kept the binary status file for compatibility and added
  versioned marker names for fast existence checks, but
  `target/performance/status-marker-smoke.json` regressed the status diagnostic
  to two of six realistic-equivalent fixtures meeting 2x and left the cold
  headline verdict `not-complete`. The marker implementation was removed.
  Retained validation after removal: `cargo fmt --all -- --check`, `cargo test
  -p assura-check-cli --bin assura-checkd --quiet`, `cargo test -p
  assura-check-cli --test batch_cli status_cli_reads_clean_daemon_status_file
  --quiet`, `cargo clippy -p assura-check-cli --bins --tests -- -D warnings`,
  `cargo build --release -p assura-check-cli`, `target/release/assura-check
  --quiet <touched-files>`, and `git diff --check`.
- 2026-05-19: Ran a cross-host Linux smoke on `vps-gw` from a tar snapshot of
  the current working tree. `vps-dev` was not resolvable from this session, so
  `vps-gw` was used as the available Linux host. The first build exposed a real
  portability bug: `assura-check-status` linked `System` on all Unix targets.
  Fixed it to link `System` only on Apple and `c` elsewhere, matching
  `assura-check-unix-client`. The full benchmark binary also required building
  with `--no-default-features --features full-cli` on Linux to avoid the
  `git2`/OpenSSL system dependency. The Linux report at
  `target/performance/vps-gw-smoke.json` improved attribution but did not
  complete the goal: cold `assura-check-cli` reached aggregate 2.38x speedup
  and met 2x on three of six realistic-equivalent fixtures, while
  `assura-check-status-cli` met 2x on all six as a persistent-daemon
  diagnostic row. Evidence: `crates/assura-check-cli/src/status.rs`,
  `target/performance/vps-gw-smoke.json`, remote `cargo build --release -p
  assura-check-cli`, remote `cargo build --release -p assura --bin assura
  --no-default-features --features full-cli`, and remote
  `target/release/assura performance-report --iterations 3`.
- 2026-05-19: Removed `git-signals` from the root `assura` crate default
  feature set so default full-CLI builds no longer pull `git2`/OpenSSL, while
  preserving opt-in `--features git-signals` and all-features behavior.
  Validation passed: `cargo check -p assura`, `cargo check -p assura --features
  git-signals`, `cargo check -p assura --all-features`, `cargo test -p assura
  --test maturity_tests --quiet`, `cargo test -p assura --test maturity_tests
  --features git-signals --quiet`, `cargo fmt --all -- --check`, `git diff
  --check`, and `cargo build --release -p assura --bin assura`. Dependency
  checks now show default builds exclude `git2`/OpenSSL and
  `--features git-signals` restores `git2 -> assura` plus the OpenSSL
  transitive chain. A fresh matched-target smoke at
  `target/performance/default-git-signals-opt-in-smoke.json` remains
  `two_x_claim_verdict=not-complete`: cold `assura-check-cli` is faster than
  native LS-Lint on all six realistic-equivalent fixtures, meets 2x on two of
  six, and has 1.78x aggregate speedup.
- 2026-05-19: Fixed a performance-report freshness false negative for
  check-only sibling binaries. The harness previously compared source mtimes
  only to the final sibling executable, so normal `cargo build --release -p
  assura-check-cli` no-op builds could still be reported as stale after
  unrelated manifest edits because Cargo updated the dep-info file without
  relinking the executable. The freshness guard now accepts the newest matching
  Cargo dep-info timestamp (`assura-check.d` / `assura-check.exe.d`) as part of
  the sibling build freshness check. Evidence: `src/cli/performance_report/assura_cli.rs`,
  `cargo fmt --all -- --check`, `cargo test -p assura
  performance_report::assura_cli --quiet`, `cargo check -p assura --quiet`,
  `cargo clippy -p assura --lib -- -D warnings`, `target/release/assura-check
  --quiet <touched docs/source>`, `git diff --check`, and
  `target/performance/depinfo-freshness-smoke.json`. The smoke proves the
  ordinary `target/release` performance-report run measures all six
  `assura-check-cli` realistic-equivalent rows instead of skipping them, but it
  remains `two_x_claim_verdict=not-complete`.
- 2026-05-19: Ran a fresh ordinary release-artifact smoke after the dep-info
  freshness fix: `target/release/assura performance-report --output
  target/performance/current-state-smoke.json --history
  target/performance/current-state-smoke.jsonl --website-dir
  target/performance/current-state-website --iterations 3`. The report
  measured all six realistic-equivalent headline rows and remains
  `two_x_claim_verdict=not-complete`: cold `assura-check-cli` was faster than
  native LS-Lint on all six fixtures, met 2x on two of six, and had 1.80x
  aggregate speedup.
- 2026-05-19: Refreshed tracked benchmark and website data from the ordinary
  release artifact with `target/release/assura performance-report --output
  benches/history/current.json --history
  benches/history/ls-lint-comparison-history.jsonl --website-dir
  website/public/data/performance --iterations 5`, then rebuilt the website
  with `cd website && pnpm build`. The tracked report remains conservative and
  honest: `benches/history/current.json` and
  `website/public/data/performance/current.json` both report
  `two_x_claim_verdict=not-complete`, six of six headline fixtures faster than
  native LS-Lint, one of six fixtures meeting 2x, and 1.57x aggregate speedup.
- 2026-05-19: Rejected a YAML-backend dependency swap as a cold-start fix.
  Temporarily aliased the `serde_yaml` dependency to `serde_norway` and built
  matched release artifacts in `/private/tmp/assura-serde-norway-target`.
  `cargo check -p assura-check-cli --bin assura-check --no-default-features`
  passed, but the release binary still linked `unsafe-libyaml-norway`, remained
  about the same size, and `target/performance/serde-norway-smoke.json`
  regressed the headline result to `two_x_claim_verdict=not-complete`, one of
  six fixtures meeting 2x, and 1.51x aggregate speedup. The dependency alias
  was reverted.
- 2026-05-19: Rejected `serde-saphyr` as a low-risk YAML backend swap. The
  alias trial failed `cargo check -p assura-check-cli --bin assura-check
  --no-default-features` because the crate is not API-compatible with
  `serde_yaml::Value` and `serde_yaml::Mapping`, which are used by the current
  LS-Lint migration and markdown/frontmatter paths. Completing that direction
  would require a broader parser abstraction/refactor, not a targeted
  cold-start fix.
- 2026-05-19: Made the opt-in pinned external fixture rows measurable. The
  Next.js fixture needed a truly permissive broad naming policy for real-world
  route/package names and broken symlinks skipped during fixture copying so the
  native LS-Lint binary can stat the materialized tree. The mdBook fixture
  needed dotfile ignores and broad Rust filename matching for example files.
  Added regression coverage that preserves valid symlinks but skips broken
  symlinks in external fixture copies. Evidence:
  `src/cli/performance_report/external_fixtures.rs`,
  `src/cli/performance_report/fixture_metadata.rs`, `cargo fmt --all
  -- --check`, `cargo test -p assura performance_report::external_fixtures
  --quiet`, `cargo test -p assura performance_report::fixture_tests --quiet`,
  and `target/performance/external-fixtures-complete-smoke.json`. The opt-in
  smoke now measures all eight realistic-equivalent headline rows; pinned
  Next.js runs `assura-check-cli` at 586.46 ms versus native LS-Lint at
  2774.16 ms, pinned mdBook runs 8.90 ms versus 23.00 ms, and the eight-row
  aggregate reaches 4.46x. The overall verdict remains
  `two_x_claim_verdict=not-complete` because five generated small/policy rows
  still miss the universal 2x target.
- 2026-05-19: Started a compiled-config freshness slice in response to the
  config-change tracking review. The intended change is to keep exact
  source-config hash validation for stale artifacts while adding a cheap
  source metadata fingerprint so unchanged compiled-config CLI runs can avoid
  rereading YAML bytes before validation. This targets the compiled-config
  execution model, not the ordinary cold `assura-check` headline row.
- 2026-05-19: Added the compiled-config source metadata fingerprint and bumped
  the artifact schema. Unchanged compiled artifacts now validate the canonical
  config path and project root, then accept a matching size/mtime fingerprint
  before falling back to reading the source YAML and comparing the retained
  content hash. Unix artifacts include device, inode, and ctime data in the
  fingerprint. Validation passed: `cargo test -p assura compiled_artifact
  --quiet`, `cargo test -p assura-check-cli --test compiled_config_cli
  --quiet`, `cargo fmt --all -- --check`, `cargo clippy -p assura --lib
  -- -D warnings`, `cargo clippy -p assura-check-cli --bins --tests --
  -D warnings`, `cargo build --release -p assura-check-cli`, `cargo build
  --release -p assura --bin assura`, and `git diff --check`. A one-iteration
  smoke at `target/performance/compiled-fingerprint-current-smoke.json`
  remains `two_x_claim_verdict=not-complete`: ordinary `assura-check-cli` is
  faster than native LS-Lint on five of six headline fixtures, meets 2x on two
  of six, and has 1.46x aggregate speedup. The compiled-config diagnostic row
  improved on some small rows but still does not satisfy universal 2x.
- 2026-05-19: Tightened the compiled-config source fingerprint after review.
  The metadata fast path now bypasses the retained source hash only when the
  fingerprint contains strong Unix identity data (`dev`, `ino`, and `ctime`);
  other platforms fall back to exact source-byte hashing. Added a same-size
  rewrite regression on Unix to prove stale artifacts are still rejected.
  Evidence: `src/cli/check/compiled_fingerprint.rs`,
  `src/cli/check/compiled_artifact_tests.rs`, `cargo test -p assura
  compiled_artifact --quiet`, and `cargo clippy -p assura --lib --
  -D warnings`.
- 2026-05-19: Rechecked whether there was an untried completion route in
  amortized batch execution, structured-output removal, or the compiled-config
  runtime. The existing batch implementation already groups paths by
  project/config and reuses the loaded config/checker; prior same-config batch
  timing was effectively tied with native LS-Lint, not 2x. Prior default builds
  without JSON/cache support, quiet-only parser experiments, and a minimal
  quiet binary were already measured and rejected. The current
  `assura-check-compiled` release artifact no longer links `serde_yaml` or
  `unsafe-libyaml`, and the fingerprint slice keeps stale artifact rejection
  intact, but the compiled-config row remains diagnostic and does not complete
  the universal generated-fixture 2x gate. Updated
  `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md` with this routing
  conclusion.
- 2026-05-19: Rechecked the remaining `rule_heavy_repo` miss before trying
  another suffix-matching optimization. The latest smoke shows the row is not
  blocked by validation throughput: `assura:phase:walk-and-validate` is about
  `0.79 ms`, config load about `0.21 ms`, and checker init about `0.06 ms`,
  while the cold `assura-check-cli` row is about `6.93 ms` versus a `4.44 ms`
  2x target. Because the prior dot-suffix lookup map for rule-heavy naming was
  already measured and rejected, this points back to process/startup overhead
  rather than another rule-heavy matcher rewrite. Updated
  `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md` with the
  attribution.
- 2026-05-19: Updated the repo-local performance-reporting skill to route
  future agents away from repeatedly treating diagnostic batch, compiled,
  hot-daemon, dirty-project, status-file, or in-process rows as cold headline
  completion evidence. Follow-up workflow validation passed with
  `target/release/assura-check --quiet .` and `git diff --check`. Current
  evidence at that point was `target/performance/compiled-fingerprint-current-smoke.json`
  with `two_x_claim_verdict=not-complete`, `two_x_pass_count=2`, and aggregate
  speedup `1.46x`; the active goal remained incomplete before the later
  static-CRT completion.
- 2026-05-19: Rechecked checked-in benchmark and website data before deciding
  whether to refresh public artifacts from the one-iteration
  `compiled-fingerprint-current-smoke.json`. `benches/history/current.json`
  and `website/public/data/performance/current.json` both remain conservative
  and aligned with each other: `two_x_claim_verdict=not-complete`,
  `two_x_pass_count=1`, and aggregate speedup `1.57x`. The newer smoke is
  useful implementation evidence but is noisier and not a tracked-history
  replacement. Public data was intentionally left unchanged.
- 2026-05-19: Reused the compiled-config source fingerprint in
  `PreparedStructureCheck` so long-lived daemon/editor sessions can prove an
  unchanged config without rereading and hashing YAML on every reload check.
  The prepared checker still falls back to exact source hashing when the
  fingerprint is missing or stale, still reparses and recompiles when the
  content hash changes, and updates the fingerprint after same-content rewrites.
  Evidence: `cargo fmt --all -- --check`, `cargo test -p assura
  prepared_check --quiet`, `cargo test -p assura compiled_artifact --quiet`,
  `cargo test -p assura-check-cli --test hot_cli --quiet`, and
  `cargo clippy -p assura --lib -- -D warnings`. This is retained
  incremental-session architecture work; it does not change the cold
  `assura-check-cli` headline verdict.
- 2026-05-19: Extended the same source-fingerprint freshness model to the
  opt-in cached check path. `run_structure_check_cached` now reads the cache
  entry first and can accept it without reading or hashing `.assura/config.yml`
  when the cached config fingerprint still proves freshness; otherwise it falls
  back to the retained config hash before parsing/validating YAML. The cache
  schema was bumped so older cache entries are rebuilt under the stronger
  contract. Added direct cache freshness regressions proving a cache entry can
  be accepted with a matching source fingerprint and no config hash, while a
  stale fingerprint without a hash is rejected. Evidence: `cargo fmt --all
  -- --check`, `cargo test -p assura-check-cli --test batch_cli cache
  --quiet`, and `cargo test -p assura cache --quiet`. This strengthens the
  project-state-cache execution model and remains diagnostic for the cold
  headline gate.
- 2026-05-19: Rejected automatic default compiled-artifact probing in the
  ordinary `assura-check` path. The prototype made single-path checks probe
  `.assura/check-config.bin` and fall back to YAML on stale/corrupt artifacts,
  but the release smoke at
  `target/performance/default-compiled-artifact-isolated-smoke-2.json`
  regressed the cold headline to `two_x_claim_verdict=not-complete`, four of
  six realistic-equivalent rows faster than native LS-Lint, one of six rows
  meeting 2x, and 1.49x aggregate speedup. The automatic probe was reverted to
  preserve the no-artifact cold path. Retained the benchmark harness cleanup:
  each scenario now removes stale `.assura/check-config.bin` artifacts before
  measuring rows so prior compiled diagnostic runs cannot contaminate the
  headline `assura-check-cli` row; the compiled diagnostic row still writes its
  default artifact after the headline row is measured. Evidence: `cargo test
  -p assura performance_report --quiet`, `cargo test -p assura-check-cli
  --test compiled_config_cli --quiet`, and
  `target/performance/default-compiled-artifact-reverted-smoke.json` confirmed
  zero skipped compiled rows after the harness fix. The reverted-code smoke
  still remains `two_x_claim_verdict=not-complete`, with five of six
  realistic-equivalent rows faster than native LS-Lint, one of six rows
  meeting 2x, and 1.51x aggregate speedup.
- 2026-05-19: Added regression coverage for the benchmark artifact cleanup
  invariant. The new `benchmark_compiled_artifact_cleanup_preserves_source_config`
  test proves the harness removes both `.assura/check-config.bin` and
  `.assura/performance-check-config.bin` while preserving the source
  `.assura/config.yml`. Evidence: `cargo test -p assura
  benchmark_compiled_artifact_cleanup --quiet` and `cargo test -p assura
  performance_report --quiet`.
- 2026-05-19: Hardened claim-summary completion semantics for smoke runs.
  Generated `claim_summary` objects now include
  `minimum_completion_iterations`, `measured_iterations`, and
  `sufficient_completion_iterations`. Reports with fewer than three measured
  iterations serialize `two_x_claim_verdict=not-complete-low-sample`, even if
  their measured rows happen to clear the 2x threshold, so one-iteration smokes
  cannot be mistaken for completion evidence. Evidence: `cargo test -p assura
  headline_summary --quiet`, `cargo test -p assura performance_report
  --quiet`, `cargo test --test performance_report_contract_tests --quiet`,
  `pnpm --dir website build`, and
  `target/performance/low-sample-guard-command-smoke.json`, whose
  one-iteration report and generated website data both serialize
  `two_x_claim_verdict=not-complete-low-sample` with zero skipped compiled
  diagnostic rows.
- 2026-05-19: Updated public rendering and contract coverage for the new
  low-sample verdict. The website performance evidence component now displays
  `not-complete-low-sample` as "Not complete (low sample)", and the checked-in
  report contract accepts either legacy claim summaries or new summaries where
  `measured_iterations`, `minimum_completion_iterations`, and
  `sufficient_completion_iterations` agree with the verdict. Evidence:
  `cargo test --test performance_report_contract_tests --quiet` and
  `pnpm --dir website build`.
- 2026-05-19: Refreshed checked-in performance history and website data from
  the current release binaries so tracked artifacts include the low-sample
  claim-summary fields. Command: `target/release/assura performance-report
  --output benches/history/current.json --history
  benches/history/ls-lint-comparison-history.jsonl --website-dir
  website/public/data/performance --iterations 5`. The refreshed tracked
  report remained honest and incomplete: `two_x_claim_verdict=not-complete`,
  `measured_iterations=5`, `sufficient_completion_iterations=true`, six of six
  headline fixtures faster than native LS-Lint, one of six meeting 2x, and
  1.77x aggregate speedup. A later 2026-05-19 tracked refresh added
  `assura-rust-cli-floor` and `warm_claim_summary`; that report is still
  incomplete with one of six cold headline rows. A later persistent-session
  refresh moved the warm sibling gate to
  `assura-check-dirty-project-session-cli`, where it completes six of six
  warm rows. Compiled diagnostic skipped rows were zero.
  Website rebuild passed with `pnpm --dir website build`.
- 2026-05-19: Tested and rejected a symlink-backed status-file fast path for
  the editor-session CLI model. The prototype changed Unix status publication
  from the tiny binary payload to an atomic symlink target and made
  `assura-check-status` try `readlink(2)` before the existing binary-file
  fallback. It built and passed the targeted status test and clippy check, but
  the release smoke at `target/performance/status-symlink-smoke.json`
  regressed the status diagnostic: `assura-check-status-cli` met 2x on only
  three of six realistic-equivalent fixtures, while the cold headline remained
  `two_x_claim_verdict=not-complete` with one of six rows meeting 2x. The
  experiment was reverted and recorded as another rejected status-client
  micro-optimization.
- 2026-05-19: Retained small compiled-artifact source-freshness
  optimizations. `CompiledStructureConfigArtifact::matches_source_config` and
  `matches_project_root` compare already-absolute runtime paths against stored
  artifact paths before falling back to `canonicalize()`. The default
  in-project compiled artifact path now verifies freshness from the strong
  source fingerprint or exact source-byte hash without also requiring an exact
  source config path match; explicit `--config` invocations keep the stricter
  source-path contract. Evidence: `cargo fmt --all -- --check`, `cargo test
  -p assura compiled_artifact --quiet`, `cargo test -p assura-check-cli
  --test compiled_config_cli --quiet`, `cargo clippy -p assura --lib -- -D
  warnings`, and `cargo clippy -p assura-check-cli --bin
  assura-check-compiled -- -D warnings`. The release smoke at
  `target/performance/compiled-default-source-relaxed-smoke.json` still
  remained incomplete: cold headline `two_x_claim_verdict=not-complete`, one
  of six headline rows meeting 2x, and 1.56x aggregate speedup. The compiled
  diagnostic row improved on several fixtures but still was not a universal 2x
  path.
- 2026-05-19: Added a scope-decision record because the retained evidence now
  consistently shows the exact universal cold CLI 2x gate is not achieved and
  the remaining misses are startup/floor dominated rather than validation
  throughput dominated. `docs/analysis/2026-05-19-ls-lint-performance-scope-decision.md`
  preserves the prompt-to-artifact conclusion and recommends keeping the cold
  `claim_summary.two_x_claim_verdict` gate honest while creating a separate
  editor-session/prepared-config performance gate if the product claim should
  move to config-dirty incremental validation.
- 2026-05-19: Added a progress ledger at
  `docs/analysis/2026-05-19-ls-lint-performance-progress-ledger.md` so future
  performance work has one routing artifact for retained improvements,
  rejected experiments, lower-bound reasoning, remaining research areas, and
  the recommended warm/editor-session next goal. The ledger records that speed
  is the product priority; binary size is only relevant when it removes
  measured startup or hot-path work.
- 2026-05-19: Added the no-op Rust CLI floor diagnostic, `warm_claim_summary`,
  and dirty-path de-duplication experiments that led to the retained persistent
  session client. Detailed evidence moved to the progress ledger.
