# Reliability and scope work packet

Read [global constraints](execution-backlog.md) and the linked strategy first. Paths below are relative to the current Assura execution checkout, not the older strategy checkout. New files are explicitly marked Create. Checkboxes are execution steps, not completed work.

## B00

**Outcome:** A current, isolated baseline with known failing/passing evidence and no duplicate implementation effort. **Own:** Git/worktree setup and task evidence only.

- [ ] Run `pwd`, `git status --short`, `git remote -v`, `git worktree list`, `git ls-remote origin refs/heads/master`, `gh pr list --repo rothnic/assura --state open`, and the workflow gate. Classify existing strategy files as owned planning work; leave other work untouched.
- [ ] Fetch `origin` and create a fresh worktree from `origin/master` using the worktree skill. Record the fetched SHA. Keep the execution backlog available by copying only these task documents, preserving links, or committing the planning documents first under the repository workflow. Never reset the older checkout or copy its source over master.
- [ ] Inspect PR #142 before installer/CI edits. It currently has passing performance but failing macOS and Alpine adoption checks; determine which fixes it already carries. Record file ownership and avoid working on its branch without coordination.
- [ ] Inspect `/Users/nroth/workspace/nickroth-assura-case-study`, branch `codex/assura-case-study`; it already contains the case study. Record the corresponding remote PR/deployment state for W03.
- [ ] Build `cargo build --locked --bins`, record both binary versions, and run focused baseline tests plus the self-check. Record hosted CI at the fetched SHA. If Cargo fails, use `assura-local-build`; do not rewrite the lockfile as a bootstrap fix.

**Accept:** Evidence includes exact source SHA, isolated path, binary paths, open overlapping work, current CI failures, and next-ready IDs. Existing user work remains recoverable. **Next:** P01/R01/R02; F01 can proceed independently.

## P01

**Outcome:** Canonical roadmap and marketed scope agree. **Own:** `.trellis/spec/assura/roadmap.md`, `docs/data/public-roadmap.json`, `docs/data/release-surfaces.json`, `docs/support-policy.md`, `docs/goals/assura-claim-complete-v0-4-and-v1.md`; Create `docs/analysis/assura-scope-decisions.md`.

- [ ] Make a row for every command/marketed feature: implementation evidence, published version, supported/experimental/deferred status, consumer references, owner card. Start from the release manifest rather than constructing a second command inventory.
- [ ] Set the growth priority to repository conventions, local patterns, agent initialization and deterministic feedback/gates. Freeze new semantic search, knowledge platform, maturity-score and orchestration work. Retain existing supported behavior pending consumer review; do not relabel an unsupported feature as supported to satisfy the site.
- [ ] Mark completed merged landing/onboarding prerequisites with their actual evidence; replace stale active-branch instructions with current task ownership. Link this execution backlog, preserving historical goal supersession links.
- [ ] Record support narrowing proposals separately from changes already authorized. For four-host support, either keep all four proof obligations or obtain a specific narrowed-support decision. No silent omission.
- [ ] Run `cargo xtask target-state`, `cargo xtask evidence`, `cargo xtask docs` and the current-source self-check. Inspect generated docs/manifest changes for consistency.

**Accept:** Each public claim has a version/evidence row; no active goal points an agent to already-completed implementation; deferred features have no new execution tasks except containment/removal decisions. This document is a product decision record, not proof of adoption.

## R01

**Outcome:** Watch honors directory scope without losing config updates or overflow safety. **Own:** `src/cli/watch.rs`, `src/cli/watch_tests.rs`, `tests/watch_cli.rs`; `crates/assura-watch-state/src/*` only if the reproduced defect is in shared normalization.

Observed hosted failure: `watch_honors_the_requested_directory_scope`, extra report after creating `docs/BadName.ts` while watching `src`. `record_message` already filters paths and preserves `need_rescan`; do not assume missing path filtering is the cause.

- [ ] Improve `WatchProcess::assert_no_event` to include the unexpected JSON event and distinguish timeout from disconnected reader. A disconnected process is a test failure, not proof of silence.
- [ ] Run `cargo test --test watch_cli watch_honors_the_requested_directory_scope -- --exact --nocapture` on macOS. Capture event paths, event kind/rescan flag and config-generation state at the normalization boundary using test-only instrumentation.
- [ ] Add deterministic tests through existing `record_message`/`test_watch_context` helpers for: outside `docs` event ignored; inside `src` event retained; unchanged config-parent event ignored; changed config reloads; explicit overflow/backend error still forces a full requested-scope check.
- [ ] Choose the fix from evidence: normalize equivalent canonical paths if paths differ; filter a known irrelevant event before recording it if it is spuriously retained; distinguish a genuine rescan from an ordinary empty-after-filter event. Never discard all `need_rescan` events. If the error is late startup/backend notification, establish a tested readiness/baseline boundary rather than adding arbitrary sleeps or ignoring arbitrary reports.
- [ ] Run the focused test 20 consecutive times after the final change, then the whole `watch_cli` target and unit tests. Obtain macOS/Linux/Windows hosted proof before claiming supported-platform closure.

**Acceptance cases:** outside edit → no diagnostic event; inside invalid edit → `file_naming` failure scoped to `src`; config change → reloaded full requested-scope report; overflow → observable full fallback; sustained edits → bounded feedback. **Stop:** if no cause is reproduced, preserve logs and mark blocked with exact host/event needed; do not implement speculative debounce changes.

## R02

**Outcome:** One failed performance comparison cannot erase independent native/warm evidence. **Own:** `.github/workflows/ci.yml`, existing CI/evidence guards in `xtask/src/main.rs`, `tests/performance_report_contract_tests.rs` if report behavior changes.

- [ ] Read the performance job. It currently generates LS-Lint data, enforces its gate, then generates native and warm data. A failed first gate skips later generation but upload steps still demand the absent files.
- [ ] Give each generation step an ID. Run independent generation after a successful bundle build even if an earlier comparison gate failed, using `if: ${{ !cancelled() && steps.perf_build.outcome == 'success' }}` with an actual `perf_build` step ID added to the build.
- [ ] Run each gate only when its own report generation succeeded; preserve nonzero failures. Upload an artifact only when its producing step succeeded and the expected report exists. If generation failed, summarize `generation_failed`; if prerequisite build failed, summarize `not_run_build_failed`. Do not manufacture a passing empty report or change the job to `continue-on-error`.
- [ ] Test a workflow fixture/dry-run guard where LS-Lint gate fails but native/warm generation succeeds: overall result remains failing, and both independent artifacts exist. Test a failed build: no misleading missing-file uploads. Existing workflow contract checks must still reject missing mandatory steps.

**Accept:** Root failure is visible; successful independent evidence is retained; absent evidence has a reason; no green result from skipped required gates. This closes artifact cascading only, not performance regression.

## R03

**Outcome:** Resolve measured many-scope overhead while preserving equivalent policy behavior. **Own:** relevant `src/cli/check/ls_fast_plan.rs`, `ls_fast_scope_composition.rs`, `scope_patterns.rs`, `compiled_config.rs`, benchmark fixture/report code only after phase attribution.

Hosted row: `many_configured_scopes_regression`, Assura 19.074 ms vs LS-Lint 17.988 ms. Treat these as one hosted sample, not a universal timing result.

- [ ] Generate the exact current CI comparison command from `.github/workflows/ci.yml` using release binaries. Run `cargo xtask performance-no-slower target/performance/ls-lint-comparison.json`; retain all rows and phase timings.
- [ ] Repeat paired cold runs on the same hardware without concurrent workloads, alternating order. Separate startup, YAML normalization, scope matching, traversal and reporting. Use the existing performance-reporting skill; do not replace the benchmark protocol with a favorable one.
- [ ] If per-scope matching dominates, precompile/cache selectors once per prepared policy and avoid scanning unrelated scopes per path. If config normalization dominates, memoize reusable rule expansion within one config compile. If startup dominates, inspect launcher/feature linkage first. Implement only the attributed branch; preserve the plain checker as an oracle.
- [ ] Add correctness cases for overlapping scopes, inherited rules, exclusions, missing directories and intentionally invalid names. Compare normalized diagnostics between plain/fast/compiled paths; ignore only timings and temporary absolute prefixes.
- [ ] Rerun all accepted comparison rows, native/warm regression gates and cold-check correctness. Report absolute medians/p95 and raw sample count.

**Accept:** Existing performance contract passes on its required runner and no correctness regression. **Escalation:** If the absolute no-slower requirement is dominated by confirmed runner noise, propose a separate contract amendment with variance data; do not widen tolerance, drop the row or relabel the result unilaterally.

## R04

**Outcome:** An honest, continuously tested Rust minimum. **Own:** workspace `Cargo.toml` files, `Cargo.lock` only if needed, `.github/workflows/ci.yml`, contributor/install docs.

- [ ] Run `cargo metadata --offline --locked --format-version 1` and `cargo tree --locked -i icu_normalizer`. Distinguish target-specific dependencies from the default host graph. Current default graph contradicts Rust 1.70.
- [ ] Prefer raising the MSRV over downgrading maintained dependencies solely to preserve an untested historical claim. Start verification with Rust 1.86, the known default-path floor; run `cargo +1.86.0 check --locked --workspace --all-targets` and the minimal launcher feature build. If another host dependency requires higher, record it and test that exact version. The supported minimum is the first version proven across the promised matrix, not an estimate from metadata.
- [ ] Update every workspace rust-version and contributor mention to that tested minimum; add an MSRV job pinned to it. Keep current-stable jobs too. Do not use only `stable` in the MSRV lane.
- [ ] Test default full CLI and `--no-default-features --features json-output,yaml-config`; test optional feature combinations actually promised. Run root/workspace manifest consistency guards and locked builds.

**Accept:** Clean-checkout instructions build on the declared minimum; CI fails if the lockfile requires newer Rust; all documented version floors agree. If maintaining Rust 1.70 is a real external requirement, stop with the dependency tradeoff instead of silently breaking that requirement.

## R05

**Outcome:** Finish installer hardening already in PR #142. **Own:** that PR's scoped installer changes in `website/public/install.sh`, `install.ps1`, install docs and smoke jobs, coordinated through B00.

- [ ] Inspect the PR diff and current macOS/Alpine failure logs. Reuse changes already proven; distinguish failures shared with master (R01) from installer-specific defects.
- [ ] Ensure version selection is explicit: default published release, requested `ASSURA_VERSION` exact tag, unsupported platform error before mutation. Verify checksums and stage downloads in a temporary directory before replacing installed files.
- [ ] Install `assura` and `assura-full` as a consistent pair. Preserve a recoverable old pair until the new pair is validated. On failed download, checksum or extraction, leave the old installation runnable. Test paths containing spaces and non-writable destinations.
- [ ] Extend existing archive/adoption smoke to run a full-CLI command as well as `check`; verify Linux glibc/musl, macOS Intel/ARM and Windows artifacts. Diagnose the actual Alpine error; do not skip that job.
- [ ] Run the existing release-size/release-smoke targets plus installer tests. Provide PR-ready evidence; merging remains a maintainer action unless explicitly authorized.

**Accept:** Clean install and replacement install succeed on each advertised platform; injected failure preserves the old pair; shell/PowerShell documentation matches behavior. Cancelled Windows CI is not a pass.

## R06

**Outcome:** The public installation delivers every retained advertised capability. **Own:** release manifests/checklist, tag/release preparation, site release-status projection; publication has an external authority boundary.

- [ ] Record one candidate SHA and tag. Verify all dependency cards' relevant changes are present; run `cargo xtask pr`, required OS tests, `release-size`, `release-smoke`, `release-readiness`, performance gates and generated-site/example checks against it.
- [ ] Generate a release decision bundle: checks/URLs, known limits, changelog, checksums, platform list and public quickstart commands. A new commit invalidates affected evidence and requires rerun.
- [ ] Before publication, label candidate-only site commands explicitly or present supported published-release alternatives. Do not fetch live GitHub state client-side to hide an inconsistent build; use the existing release manifest/build checks.
- [ ] Prepare the exact tag/release/site deployment actions. Obtain publication authority; no agent should push a tag merely because a local checklist is green.
- [ ] After authorized publication, run `ASSURA_VERSION=<actual-tag> cargo xtask release-live`, then the default public installer in a clean environment. Execute displayed onboarding, check, explain and supported integration paths. Verify site label, downloads and release API agree.

**Accept:** Public installed version and retained claims match, with platform evidence and a working upgrade path. Before authorization, record `blocked: publish_authority` with a complete reviewable release bundle; do not mark this card done.

## R07

**Outcome:** Every current self-check advisory has a deliberate disposition. **Own:** only files identified by current-source JSON plus narrowly scoped `.assura/config.yml` exceptions.

- [ ] Run `cargo run --bin assura-full -- check --format json .`; retain every finding and distinguish blocking from advisory. Current snapshot has 16 advisory violations.
- [ ] For each finding, select: simplify/move by responsibility; repair outdated docs; or retain a narrowly scoped exception with rationale and review date. Use `assura-structure-fit` before changing allowances.
- [ ] For length findings, preserve semantic cohesion. Do not split with `include!` or insert broad excludes simply to obtain green output. If a generated fixture is intentionally large, scope the exception to the fixture and retain generator validation.
- [ ] Run relevant tests for any Rust change and the self-check again. Add a neighboring negative fixture showing the rule still catches unintended growth.

**Accept:** Zero unclassified advisories; zero blocking findings; exceptions are visible and bounded. Report zero violations only if the output actually has zero.
