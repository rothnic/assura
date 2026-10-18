# SP04: Correct native performance history admission

## Objective

Correct the native baseline loader so repeated JSONL rows from one measured
execution cannot satisfy the existing minimum of two checked reports or count
its samples twice. Keep this bounded to the existing
`src/cli/performance_report/native_regression.rs` and
`native_regression_tests.rs`; only add a short row-count documentation
clarification if needed. Do not change serialized report fields, public APIs,
thresholds, hardware keys, checked history, current report, baseline promotion,
or the history retention policy.

## Owner, base, and phase

- Candidate: `10-18-sp04-native-admission`.
- Owner: `sp04-native-admission`; root owns the parent Beads issue `ae-mwl`,
  native execution, PR/push/integration, and delivery closure.
- Branch: `fix/sp04-native-admission`.
- Base ref: `refs/remotes/origin/master` at
  `74d9917dd39e599bbd43c3a5760b71644b2cf91b` (verified in this worktree; do not
  use the stale local `master` ref).
- Current phase: implementation and one focused RED/GREEN local test lane are
  authorized. Root owns PR, push, native validation, integration, and delivery
  closure. No benchmark, report promotion, hardware experiment, or Beads
  operation is part of this candidate.
- Candidate implementation files: the two existing native regression files
  above. No new module, dependency, schema, CLI option, report identity field,
  or history rewrite.

## Observable contract

For each existing full `NativeBaselineKey`, admit a measurement contribution
once per opaque `(timestamp, commit_sha)` execution descriptor. Do not collapse
different timestamps merely because they have the same executing commit.
Compare the measurement payload used by aggregation (median, min, max, sample
values and sample count), ignoring derived regression annotations that can
change when the report is replayed.

- Identical duplicate rows for one key and descriptor contribute one report
  and one row's samples, independent of duplicate order.
- Contradictory measurement payloads for one key and descriptor fail closed
  through the existing loader/status path; do not choose a first or last row.
- Exclude the exact current target key/descriptor from both JSONL history and
  checked `native-current.json` fallback. Do not exclude other timestamps at
  the same commit, other keys, or rows based on source commit, branch, CPU, or
  patch identity.
- Preserve the explicit current-row provisional fallback for an unseen
  environment: one report, its measured sample count, and provisional status.
- Missing/unusable identity must not earn independent-report credit. Keep
  identity strings opaque; same-commit runs with a same-second timestamp remain
  ambiguous under the existing schema and contradictory collisions must fail
  closed.
- Preserve all existing missing-row, unusable-row, environment/profile scope,
  formula, output, status, and CLI exit behavior. The existing xtask report
  schema/status/threshold checks remain unchanged.

## Test-first proof plan

Add real loader/annotation tests before changing production logic. Feed small
JSONL fixtures through the history loader and annotation path, with deterministic
temporary files under the existing test approach and no new dependency.

1. **Duplicate execution control:** derive the real annotated result from one
   row A and A copied twice. Before implementation, assert the current behavior
   demonstrates the false calibration (`report_count=2`, doubled samples, and
   calibrated status); preserve that failing receipt. After implementation,
   require one report, one row's samples, provisional classification, and the
   same result when duplicate order is reversed.
2. **Independent same-source control:** A and B have the same commit and key,
   distinct opaque timestamps and distinct valid measurements. Require two
   reports and both sample contributions with the unchanged calibrated
   max-derived threshold.
3. **Conflict control:** same key and descriptor, different measurement
   payloads, including reversed input order. Require the loader to reject the
   ambiguous execution and let the existing annotation failure/status model
   fail closed.
4. **Exact target control:** provide independent A and B plus target C, with C
   deliberately slow and sharing the exact target descriptor. Exercise both
   history loading and checked-current fallback; exclude only C and require the
   actual annotation to classify the slowdown against A+B. An unchanged
   candidate below the same derived threshold is the positive holdout.
5. **Compatibility controls:** preserve different keys and profiles, same
   commit/different timestamp history, unrelated current rows, strict
   same-environment missing-key classification, unusable rows, and current-row
   provisional fallback for a previously unseen environment. Include no usable
   current identity so it cannot silently gain extra history credit.

Use values consistent across each JSONL row and its distributions. The RED
case is executed test output, not the already approved source/math review. Do
not weaken assertions to make existing code pass.

## Validation after lane release

Run focused Rust tests first against the approved candidate SHA. The sibling
test module is included under the library target (`src/cli/mod.rs` exposes the
performance-report module); `--bin assura-full` selects a binary with zero
unit tests. The focused command is
`cargo +1.94.1 test --lib --jobs=2 native_regression::tests:: -- --test-threads=2`.
Then run the smallest existing xtask-gate compatibility controls: positive and
negative synthetic native reports through
`cargo xtask native-performance-no-regression target/performance/native-unchanged.json`
and
`cargo xtask native-performance-no-regression target/performance/native-seeded-slowdown.json`,
plus `cargo test -p xtask native_performance_gate`. The native gate must accept
the unchanged control and reject the seeded slowdown while retaining its full
required row matrix and status/median/threshold/delta invariants. Do not
regenerate or alter checked history/current bytes.

After focused proof, use the applicable repository Rust tier and final-source
checks selected by the coordinator; no benchmark is needed for a history
admission correctness fix. Preserve exact cwd, SHA, binary/toolchain identity,
commands, actual exits, and unfavorable controls. When a validation lane is
released, run the local preflight wrapper with the exact admitted argv and the
repo's required Rust profile; do not start any other heavy job concurrently.

## Acceptance

1. Executed test-first loader/annotation controls show the duplicate and exact-
   target defects on the unchanged source, then pass after the minimum fix.
2. Identity-aware duplicate collapse, conflict rejection, exact-target
   filtering in both sources, and distinct-timestamp retention satisfy the
   observable contract above without changing threshold formulas or public
   metadata.
3. Existing fallback/missing/unusable semantics remain covered, including the
   unseen-environment current-row provisional path.
4. Focused Rust proof and positive/negative native xtask gate compatibility
   pass on the final implementation SHA. No checked report bytes are rewritten
   or promoted.
5. Independent review checks the final tested source and reports any remaining
   concerns to the root owner. Root retains PR, native, integration, and closure
   authority.

## Candidate execution evidence

- Candidate started at `5c1d77d0`; final implementation/test source SHA-256 values
  and raw receipts are recorded in
  `/Users/nroth/workspace/.assura-review-20261017/sp04/run-sp04-native-admission-5c1d77d/`.
- Final focused run: `cargo +1.94.1 test --lib --jobs=2 --offline --locked
  native_regression::tests:: -- --test-threads=2`, 17 passed, 0 failed, 544
  filtered. Final scoped Clippy `cargo +1.94.1 clippy --lib --tests --jobs=2
  --offline --locked -- -D warnings` passed. Existing xtask gate tests passed
  4/4; the unchanged 150-row report passed and the seeded slowdown was rejected
  as expected. Those xtask runs preceded the final `#[cfg(test)]` annotation on
  a private loader convenience wrapper, which does not affect non-test builds;
  see the evidence manifest for exact bindings and receipts.
- The unchanged-source four-control RED run was 1 passed / 3 failed (exit 101);
  its intermediate test-source hash was not captured. The separately bound
  unchanged-source two-control RED was 0 passed / 2 failed (exit 101). The
  mistaken `--bin assura-full` selection ran zero tests and receives no proof
  credit. The first export-enabled GREEN attempt had 16 passed / 1 setup failure
  because its requested output directory was absent; after three setup reviews,
  one run against a verified, owned `0700` directory passed all 17 and emitted
  both real annotated rows. A socket-profile preflight was also rejected before
  launch because the unrelated profile projected a Darwin socket path one byte
  over capacity. These receipts are preserved separately.
- Existing checked history and current report bytes remain unchanged. The two
  cohesive Rust files now exceed the existing 500-line advisory: production is
  558 lines (base 456), tests are 518 (base 290). No lines were compressed and
  no gate was raised. Defer any natural module split to a specifically scoped
  NU09 follow-up and an explicit structure/spec review; this candidate adds no
  module.
- Independent review `independent-review-ded91108.md` identified one low proof
  gap in the duplicate fixture. The test now annotates a distinct execution
  against the real duplicate-collapsed loader output and asserts provisional
  status, one report, five samples, the unchanged derived threshold, and
  duplicate-order invariance. The focused 17-test suite and scoped Clippy were
  rerun successfully with no production-source change; updated hashes and raw
  receipts are in the evidence directory above. Root owns scoped rereview.

## Evidence and authority references

- Approved slice research: `/Users/nroth/workspace/.assura-review-20261017/sp03/admission-correctness-slice-research.md`.
- Independent design review: `/Users/nroth/workspace/.assura-review-20261017/sp03/admission-correctness-design-review.md`.
- Parent issue and delegation authority: root-owned Beads `ae-mwl`; candidate
  code ownership delegated to `/root/native_admission_implement` for this
  worktree only.
- Bead description / scope: `/Users/nroth/workspace/.assura-review-20261017/sp03/admission-correctness-bead-description.md`.
- Preparation packet: `/Users/nroth/workspace/.assura-review-20261017/sp04-native-admission-preparation.md`.

Research and review establish source behavior and acceptance criteria, not
executed RED proof. Current checked-history census, checked-current replay math,
hardware comparability, source/patch provenance, the 1,000-row retention cap,
holdout selection, and baseline promotion are separate work and are not
silently accepted by this task.

## Current-base integration checkpoint

The reviewed59f2f062 candidate is refreshed onto master31c043eb. Its Rust,
Cargo/config, benchmark and protected performance-history/report bytes remain
unchanged. The existing full PR boundary proof is reused:1476 passed,0 failed,
1 manual ignore across110 harnesses, with8 empty harnesses uncredited. Local
focused17, scoped Clippy and xtask4 controls remain distinct. A read-only census
comparison confirms these counts match the historical source, rather than the
different NU16/W02 candidate counts.

Fresh structure/task/workflow/diff checks passed. The unchanged prebuilt xtask
matched all seven compile inputs and revalidated the retained150-row positive
report and expected seeded-slowdown rejection; this is gate compatibility, not
a new benchmark or history promotion. Eighteen low line findings remain
advisory, including the two SP04 files tracked for later NU09 maintenance.

External integration evidence is
`/Users/nroth/workspace/.assura-review-20261017/sp04/current-base-31c-20261018T133726Z/`.
Its root correction distinguishes historical registered base29f32078 from
registered tip74d9917d and current integration base31c043eb. The native receipt
is unchanged at generation0. Keep executable context entries repository-relative;
this Markdown reference preserves external raw evidence discovery.

Independent current-base/correction review, exact final-head hosted checks and
root native delivery acceptance/close remain before merge and bead closure.
SP03 fixed-host experiments remain separately gated and unrun.

## Current-base refresh after NU16 integration

The candidate was merged onto `origin/master` at NU16 integration base
`46e1e13b9f66012dd07ffeed0a0272b31582b374` in merge
`7fd064e58a7b8b3b49391cd0bffca1c0faf18d4e`. The parent task child list is the
base list (DX02, DX03, NU16), with no other parent-task JSON changes. This
refresh adds NU16's `src/cli/watch.rs` and `src/cli/watch_signal.rs` changes;
the SP04 native loader/test files and protected Cargo/config/benchmark/history
inputs remain byte-identical to reviewed `59f2f062`. See
`/Users/nroth/workspace/.assura-review-20261017/sp04/current-base-46e-20261018T140302Z/integration-proof.md`
and its adjacent raw receipts.

The seven preserved xtask compile inputs still match their manifest, so the
existing binary could be reused for the unchanged 150-row positive report
(exit 0) and seeded-slowdown control (expected exit 1). These are gate
compatibility results, not new benchmark or native-history evidence. Existing
SP04 focused test/Clippy proof remains tied to its recorded source; NU16's
hosted full-PR proof is separate. No combined full-PR run on this refreshed
candidate is claimed. Independent review of this new base delta and exact-tip
hosted checks remain before integration. Root retains native record/close,
archive ownership, and PR authority; historical receipt base/tip and generation
0 remain unchanged.
