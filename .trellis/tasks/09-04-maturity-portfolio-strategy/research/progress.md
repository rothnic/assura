# Maturity execution train progress ([latest recovery](recovery-evidence.md); [older entries](progress-history-02.md), [preserved history](progress-history-03.md))

## Iteration 88 — 2026-09-09 — merged privacy correction and screening-manifest routing

- PR #235 merged as `5329abd` on refreshed `origin/master`; the independently
  rereviewed A07 parent-isolation/privacy slice passed its local and applicable
  hosted gates. Private evaluator, fixture, contract, event and identity
  provenance is no longer recorded in durable public evidence.
- The live A07 canary remains valid pre-screening evidence only. A protocol audit
  found that the packet's two product-input conditions were never named or
  evidenced; inventing them from historical run names would invalidate the
  screening design. The new screening-manifest contract requires exactly two
  named conditions, one changed variable, supplied-input proof, private mapping,
  and a reviewed 30-cell matrix without changing any thresholds.
- The layered context-routing reference now separates universal, goal/phase,
  card, special-lane and private-evaluator context. AGENTS remains a router;
  detail stays in skills and the task contract. Context level: not exposed.
- Scoped process review found the private-manifest reviewer boundary was
  ambiguous. The routing now defines a separate isolated protocol reviewer who
  may inspect only manifest schema/mapping/matrix metadata and returns redacted
  findings; raw oracle, child transcript and fixture contents remain private.
- Root's unknown file and stale missing-gitdir registration remain untouched;
  topology report/strict retains those ownership exceptions. The earlier
  canary was against historical `77b41fe`; the then-live baseline was
  `5329abd`, now historical after PR #237. Current routing is recorded in
  `recovery-plan.md` at `8cabc536`. Next action, in order: create and validate
  the private two-condition manifest, obtain isolated protocol-review `PASS`,
  refresh the candidate-bound canary against `8cabc536`, then run the
  authorized 30-cell screen with one sibling-free fixture/child per cell.

## Iteration 87 — 2026-09-09 — current-master A07 canary and parent-isolation correction

- Refreshed `origin/master=77b41fed7ee625333ea97ef2791da609f0ed5cc4` and built
  the release candidate in a clean detached checkout with
  `cargo build --release` (exit 0, 0.53s). Assura 0.4.0 and the candidate
  SHA-256 are recorded in `evidence/A07.md`.
- The first fresh child was correctly rejected as no-credit protocol evidence:
  its source-only fixture shared a temporary parent whose recursive discovery
  exposed sibling private worktree names. A second context-clean attempt used
  a non-frozen source fixture and therefore could not satisfy the frozen
  preservation contract. Both raw event streams remain preserved; neither was
  evaluated as screening credit.
- A third fresh one-shot child used the exact frozen Rust source hashes, a
  sibling-free disposable parent, isolated child home, minimal login-shell-safe
  PATH and the fixed public prompt. Candidate identity matched in the child
  environment; targeted event scanning found no evaluator, coordinator,
  memory, or out-of-scope parent-path access.
- The full private evaluator exited 0 with all seven dimensions passing and no
  critical failures. This is a valid candidate-bound canary only; A07 remains
  active with zero screening/holdout/final-batch credit. The runner reference,
  executor prompt and continuation prompt now state the sibling-free-parent
  requirement. Next action: record the screening budget and run the authorized
  30-run batch with one isolated owner/fixture/child per run.
- The root unknown file and stale prunable registration remain untouched.
  Topology `--report` and `--strict` still exit 128 for those pre-existing
  ownership exceptions; no cleanup mutation was performed.

## Iteration 86 — 2026-09-09 — durable goal checkpoint and owned-topology closure

- PR #233 merged the reviewed reconciliation as `1f9ebf2` on top of
  `0c0eae8`; the current master tree includes the runner-isolation contract,
  exact A02 proof, and preserved progress history. Its hosted CI Scope,
  Documentation Scope, Security Scope, Evidence Gates, and GitGuardian checks
  passed; scope-skipped product, release, performance, and Rust jobs remain
  explicitly skipped rather than counted as passes.
- The five goal-owned branch worktrees were clean, matched their merged PR
  trees, and were removed with their local refs deleted. The superseded
  detached A07 build at `f1595fc` was removed; the clean `922d7f0` candidate
  build is retained as a bounded evidence archive for the next canary.
- The root unknown file and pre-existing prunable registrations remain
  untouched. The topology report is therefore expected to retain those
  ownership exceptions; no strict-green claim is made from their presence.
  A07 remains `active` with zero screening/holdout/acceptance credit. The next
  exact action is a fresh one-shot source-only canary from current master with
  minimal login-shell-safe PATH and same-environment executable identity proof.

## Iteration 85 — 2026-09-09 — reviewed routing and proof correction

- Independent review accepted P1/P2 and scoped rereview returned PASS: the
  required A07 plan supersedes stale routing, and A02 `done` has an exact
  command/exit proof table. Historical failures, skipped-check accounting and
  no-credit rules remain preserved.
- PR #232 merged as `0c0eae8` on top of `922d7f0`; the merged tree matches the
  reviewed candidate and its applicable hosted checks pass. The owned process
  branch/worktree is clean; next action is exact cleanup/topology verification,
  then a fresh context-isolated A07 canary before any screening allocation.

## Iteration 84 — 2026-09-09 — runner-context isolation correction

- The merged A02 correction is current at `origin/master=922d7f0`; its hosted
  and local gates are recorded with PR #231. A fresh initializer attempt was
  retained as invalid protocol evidence because it explicitly selected an
  ambient Assura binary and inspected evaluator-only context; no A07 credit was
  assigned, even though later disposable checks used the intended candidate.
- The durable goal, executor prompt and goal-execution skill now route through
  one runner-isolation reference: source-only fixture, fixed public prompt,
  fresh one-shot child, minimal login-shell-safe `PATH`, same-environment
  executable identity, and separate evaluator/redacted evidence. Forbidden
  context, global-binary fallback, or missing identity invalidates a run rather
  than becoming a green result through evaluator re-invocation.
- Context level: not exposed. The correction addresses a distinct runner
  contamination failure; it does not relax A07's screening, holdout or final
  acceptance thresholds. Next: independently review and integrate this process
  slice, then run a fresh candidate-bound canary before any screening.

## Iteration 83 — 2026-09-07 — isolated diagnostic tool preparation

- A04 local master refresh completed80ad01d with normal hook30653/fdc8a7,
  clean/Ready yes, current-master ancestry and unchanged behavior source.
  Independent metadata re-review closed the stale R01-to-W02 routing finding.
  No push, hosted update, master merge, new worktree or cleanup followed.
- R03's hardware denial changed the next action. Root selected a bounded
  isolated software-tool prerequisite, not another timing attempt. Independent
  review corrected inherited-path tool selection and absent debug inventory
  before execution. The final reviewed driver used fixed system tools, a
  cleared child environment, exact versions/SHA256, and one bounded download.
- Setup14035/e063e1 exited0: two verified Ubuntu packages extracted only under
  owned /home/ubuntu/data/projects/assura-r03-valgrind-preflight.m3DTGw,
 129196KiB including downloads. Debug inventory273files plus launcher/engine
  hashes and raw log are recorded in evidence/R03.md. No package executable,
  product command, fixture, profiler or counter collection ran. No system
  installation, permissions/config/services or global environment changed.
- Post-setup libc6 remains2.42-0ubuntu3.1. The initial --show query's empty
  Valgrind version was ambiguous; --status515574 exited1 explicitly saying
  not installed. The earlier query=present label is not package-install proof.
- Current original binaries are stripped static-PIE, confirmed read-only.
  Tool extraction does not prove compatibility or source-line mapping. Root's
  next internal step is a reviewed tool-health/static-PIE observer protocol;
  event counts must not be equated with native CPU cost. The initial broad
  proposal's unexamined exact cache-vector/DWARF prerequisites are not silently
  adopted or later relaxed to fit results. No product collection approved yet.
- Queue unchanged14done/1active/4blocked/4implemented/9pending. A04 ownership
  must integrate before its next behavior slice; W02 local packet remains
  externally blocked. Publication approval/verified separation, required hosted
  gates, R01 failing-cause proof and R03 performance acceptance remain open.
  All51registered worktrees and A05/user work are preserved; the new remote
  tool prefix is separately owned and must be accounted for at final cleanup.
- Context review before handoff: level not exposed. All invoked sessions are
  terminal. The recurring issue is evidence scope, addressed with exact source/
  runner identity, independent scoped review and explicit prerequisite versus
  acceptance labels. Existing goal/build/performance guidance remains sufficient;
  this one-off setup is discoverable through R03 evidence, not a new AGENTS
  procedure or speculative project skill. Next periodic review84.

Metadata validation8278/0e9b01 exited0: structure1787files/393dirs/0violations
and cargo xtask evidence, with existing A04 target/jobs4. Final diff/source
checks b02422 exited0. Independent three-file metadata audit CLEAN on frozen
diff24387e3e5e84b58ff4ebadaf6d60226ee4557cd1f4911ae9b6fcaf3744a524f3;
root accepts its scoped evidence/authority/resource-accounting verdict.

## Iteration 82 — 2026-09-07 — current-master A04 refresh and R03 prerequisite result

- Previous turn made progress: W02 local closure/queue committed790b0a9 with
  independent audit. Context review81 confirmed serial A04 ownership/path
  sequencing and no further website expansion. Latest W02 evidence stays with
  its named owner; the local pointer below does not import W02 behavior.
- Fresh90313/f16c41: origin/master98bd187, releasev0.3.0 (July2), PR184 still
  d42094c/DIRTY with Ubuntu failure and cancelled macOS/Windows. Registered
  worktree inventory51; no creation, cleanup or unknown/dirty path mutation.
  The existing clean goal-owned A04 branch9c9a896 entered a local master merge.
  Five metadata conflicts were resolved; unique old evidence remains, including
  unfavorable R03 diagnostics. Product source/tests/workflows remain unchanged.
- Existing9c9a896 full PR log recount is1310passed/0failed/1ignored across
 104binaries,8zero-pass not coverage. Both preserved binaries still match
  recorded SHA256. Covering74364/1013c3 metadata gates passed1787files/393dirs/
  zero violations plus evidence policy/JSON counts. Do not repeat the unchanged
  Rust tier or call this fresh hosted/platform/performance proof. Independent
  resolution review and local checkpoint follow; A04 remains active, not done.
- Root made and executed the next R03 internal method decision: one independently
  reviewed unprivileged instruction-counter access probe on /usr/bin/true,
  never a product/fixture/timing run. Review fixed a5second timeout-grace excess
  before execution. Probe28740/3d547b exited255 with access denied on the exact
  VPS user/session; driver0 reports that failure, not a passing measurement.
  All identities, commands, errors and hashes are in evidence/R03.md. No retry,
  permission change, profiler attachment, optimization or acceptance change.
- Hardware counter collection is unavailable under unchanged runner policy.
  Valgrind is absent, strace present; the syscall-count fallback does not answer
  the selected userspace-work question and is not approved. Read-only package
  metadata93127/6dcb18 identifies Valgrind1:3.25.1-0ubuntu1 and matching
  libc6-dbg2.42-0ubuntu3.1 for installed libc6. This is prerequisite information,
  not a download/install or instrumented execution. Root is evaluating a finite
  isolated software event-count method, with explicit observer/compatibility
  controls; no product collection is authorized by this checkpoint.
- Queue14done/1active/4blocked/4implemented/9pending. Publication approval/
  verified no-deploy separation remains unresolved. No push, merge to master,
  release or deployment occurred. A04 ownership must integrate before its
  effective-path slice; A05 dirty work is preserved; A06/A07 stay dependent.
  R01 still requires a causal failing native-event trace, not an equivalent
  passing capture. R03's method decision remains root-owned, not automatically
  a human approval blocker.
- Context level not exposed. Existing goal/build/performance/harness guidance
  covers this work; no new skill or AGENTS expansion. All described processes
  are terminal. One truncated reference read was completed in a bounded second
  read; two guessed artifact/glob paths were corrected through file inventory.
  Neither is a product failure/pass. Next periodic context review84.

Final metadata gates35481/8eb1f6 passed on staged tree
0bab03b29be97e953f1a8a7aef1186f4534dd5ec with the same1787file/393directory
structure result and evidence policy. Independent review of frozen diff
bf2cc872406359f457db0a172c9a67643889249de923aea369b11b66efef3a84 caught one
stale leading cross-card sentence imported with R01 evidence: W02 was still
labeled an active implementation lane. Root corrected that current routing to
locally prepared/externally blocked, preserving historical records. No source,
acceptance or publication authority changed; scoped re-review is required.

## A04 merge reconciliation pointer — 2026-09-07

This checkout is resolving master98bd187 into A04 ownership head9c9a896;
the merge is not yet committed or newly gated. Only five conflicting task
metadata files are being reconciled. Product ownership source is unchanged.
Later effective-hook-path/runtime slices remain serialized until ownership
integration. Root owns the next proof/runtime entry (iteration82).

The latest controller progress/queue is preserved in local commit790b0a9 on
goal/w02-release-aware-installation. Read that commit's same-task
research/progress.md (iteration80), research/progress-history-01.md, and
research/evidence/W02.md for the newer local packet and exact proof identities.
The local W02 evidence file brought by this merge is an older snapshot; the
790b0a9 pointer is the newer source of evidence, not W02 behavior ancestry.
W02 is locally complete but externally blocked, not done: Cloudflare approval
or verified no-deploy separation and exact-head hosted/integration gates remain.
R03's stopped A/A controls and narrow offline analysis are retained in the
updated R03 evidence; no new method or run is approved by this reconciliation.
Queue:14done/1active(A04)/4blocked(R01/R03/W02/F01)/4implemented/9pending.

All old A04 HEAD progress paragraphs already occur verbatim in the following
master98bd187 reconciled log; it is preserved without copying a second ledger.
Its historical active/next/clean statements do not supersede the newer state
above. The older planning source_snapshot is not a branch-ancestry claim.

## Iteration 62 — 2026-09-07 — Evidence reconciliation and bounded stops

- Previous goal turn: progress. Independent R03 raw audit confirmed invalid
  perturbation controls and correct early stop; independent W02 re-review
  approved the source correction but not final production acceptance.
- Refreshed master3d9a255, releasev0.3.0 (July2), PR185/184/166/164/158 and
  preserved user PR142. Master Rust CI34084659580 remains failing. No existing
  PR is represented as merge-ready merely from local review.
- Created docs/maturity-evidence-handoff at current master, worktree
  /Users/nroth/.codex/worktrees/assura-maturity-evidence-handoff
  (789419 exit0). Inventory grew from50 to51; no cleanup/deletion performed.
  This documentation-only handoff consolidates later R01/A04/R03 records and
  W02 ownership without product patches. Source worktrees remain intact.
- Queue correction:14done,2active(A04/W02),3blocked(R01/R03/F01),
  4implemented(Q02/A05/Q04/Q06),9pending. R01 and A04 historical done states
  are superseded; A05 is partial, not a satisfied A07 dependency.
- R03 run25650 stopped at48Assura+48LS, three of four observer controls
  failed; independent audit ac2749 passed. No causal/noise inference, no
  rerun, no performance waiver. See evidence/R03.md for hashes and next
  measurement-method decision. Rejected product source remains reverted.
- W02 corrected tablet source passed23focused checks and independent review.
  Earlier110tests missed real component clipping; the new geometry checks
  cover actual text/cell bounds. Final production verification is separately
  owned in W02; this handoff does not mark W02 done.
- Integration order: reconcile evidence first, then finish the bounded W02
  claim correction; broader W02 and held behavior cards remain open.
  One active implementation lane plus independent review; no speculative R03
  optimization or R01 debounce changes. User-owned A05/PR142 work preserved.
- Context health: level not exposed. The repeated failure is distributed
  ledger drift, not a missing product abstraction. Durable routing below
  requires reconciling newer evidence before selecting dependencies; operational
  detail stays in existing goal/performance/build skills, no AGENTS expansion.
  Next periodic review63; no goal completion or external publication claimed.

## Reconciled branch-local history

Local handoff validation22748 terminated0: structure checked1778files/393dirs
with0violations; `CARGO_BUILD_JOBS=4` with shared A03 `CARGO_TARGET_DIR` ran
`cargo xtask evidence` successfully (xtask compiled6.21s). Checker binary
was the preserved default full binary with SHA-256
`af8f5479095d0095ff23ec9b1c296263c1743f131d9c86215847477c5cd4ebe1`.
`git diff --quiet 3d9a255 2debe29 -- src/cli/check.rs src/cli/check src/config`
exited0, establishing unchanged checker/config source, not whole-binary or
release equivalence. No Rust/CI/config/website source changes in this handoff;
heavy Rust/performance/website suites are outside its existing classified
scope, not counted as passing. Initial workflow gate Ready yes; subsequent
dirty warnings are the explicitly owned handoff paths, resolved by its commit.
Final changed-file checks and independent review remain required before PR.

The following previously unmerged records are preserved from goal-owned R03,
R01, A04 and W02 worktrees. Original iteration labels and unfavorable results
are retained; the current summary above and per-card latest evidence take
precedence over historical status claims. Original source branches remain
recoverable. This reconciliation adds records rather than rewriting outcomes.

## Iteration 61 — 2026-09-07 — W02 claim containment and journey review

- Started independent W02 in clean goal/w02-performance-claim-containment at
  refreshed master3d9a255 (worktree creation79893 terminal0). Worker owns
  website source/tests; root owns planning/evidence. No VPS competition with
  R03 diagnostic preparation; A05 and user-owned work remain untouched.
- Actual browser RED62654 proved the missing under-review state. Initial
  component refinement passed5 focused tests48396 at exact hashes recorded in
  evidence/W02.md. Setup and test-authoring failures remain distinct from
  product failures; final production visual/build/full-suite gates are pending.
- Independent review rejected linked-journey acceptance: both destinations
  still presented unqualified current/release claims, and a test preserved the
  old headline. Root accepted the finding and authorized test-first historical
  qualification of those two pages, retaining all measured tables/raw data.
  W02 is active/partial, not done or PR-approved.
- Source audit found --released compares claims to local Cargo version rather
  than independently published GitHub release. Preserve the gate, but require
  actual install-route proof for remaining W02/R06 acceptance.
- R03 temporary probe source passed review; its runner review found failed
  sample persistence, attribution-verdict, tool-order, deadline and opt-in
  control gaps. Corrections and exact-hash re-review remain required before
  measurements. Rejected product source remains reverted; no acceptance retry.
- Next: independently review the corrected diagnostic runner and W02 linked
  journey, then run their distinct bounded gates. Context review remains due63.

## Iteration 60 — 2026-09-07 — Course correction and next independent slice

- Rejection/rollback metadata is independently reviewed and committed
  b2b3632183230ebd717dc594819545dba8b122e2. Structure checked1777files/393dirs
  with0violations; evidence and normal pre-commit gates passed (session7383,
  terminal0). Fresh fetch75981 confirms master3d9a255 and unchanged open PR
  heads; workflow Ready yes, branch clean, product src equals master.
- R03 diagnostic preparation remains remote and unmeasured. Independent
  probe-only review found no source issue, but runner/controls/full identity
  review is outstanding. Absolute perturbation bounds apply in both directions
  to original/disabled/enabled controls; block-level results cannot be hidden
  by aggregation. No approval to run measurements, retry acceptance or merge.
- Read-only independent queue review identifies W02 homepage claim containment
  as the next ready local slice: render the prescribed "Performance evidence
  under review" state and retain methodology/history, rather than unqualified
  current speed claims. P01/R02/W01 dependencies are in ancestry. Preserve
  generated benchmark data; W02 remains incomplete until provenance and
  release-aware commands are also proven. No W02 implementation started here.
- W03 must reuse NickRoth PR60/canonical article; absence from main is not a
  sorting defect. Assura merge authority does not authorize NickRoth publication.
  Website deployment side effects must be checked before integration: no
  explicit master deployment or GitHub deployment record was found, but that
  does not prove external deployment integration is absent.
- Context health: level not exposed. Fourteen done/eighteen unfinished; R01
  packet-blocked, R03 rejected source restored, A04 held, A05/user work intact.
  Local free space13GiB; prune was dry-run only, nothing removed. Repeated
  setup errors came from guessed paths/schema keys and shell-sensitive names,
  not product failures. Use rg inventory, inspect JSON keys and explicit Bash
  before commands; existing performance/build/worktree guidance covers the
  reusable workflow. No new skill or AGENTS expansion is justified. Next
  context review63; next immediate action is final diagnostic runner review.

## Iteration 59 — 2026-09-07 — Reject spillover and restore source

- R03 full run55014 terminated1: many-scopes improved25.755%/7.1383865ms,
  but multipart slowed14.868%/0.6735275ms. Both392-row reports are retained;
  independent raw audit recomputed medians and confirmed rejection. Candidate
  absolute gate passed; that does not clear the before/after spillover failure.
  No second full run, retry or native/warm acceptance followed.
- Source-only revert69dfc4a restores the entire src tree byte-for-byte to3d9a255;
  net branch diff is metadata only. Workflow/structure/evidence/pre-commit
  passed. Rejected commit/binaries/full artifacts remain recoverable, including
 52MiB local audit copy. No PR, push or merge for this candidate.
- Preserve caveats: build commands selected1.98.1 but report runtime field is
 1.95-nightly; generated full-report config bytes were not retained. Observed
  failure is neither proven stable causation nor confirmed noise. The existing
  phase/context differences leave an attribution gap, not a multipart fix.
- One bounded public-path diagnostic is authorized after probe review: fixed
  fixture bytes, existing walk timer, original/disabled/enabled overhead
  controls, max96 measured launches. It cannot reinstate the rejected patch;
  failure/inconclusive localization stops rather than repeating acceptance.
- R01 remains packet-blocked at independently reviewed2debe29; its PR body
  now records the failed cold gate and unresolved native cause without another
  push/CI run. Queue now14done,2active,2blocked,4implemented,10pending.

## Iteration 58 — 2026-09-07 — Validate the measurement decision itself

- Exact bundles built in68398 exit0. Independent review caught false-pass
  paths in the one-off before/after evaluator; actual RED112a94 and nine
  corrected controls reject duplicates, missing/relabeled rows, invalid samples
  and improper skips. Root reran9 controls in e2b4fa exit0.
- Final script/evaluator review passed0626f667/3f5aa3dd. All14x28 rows and
  fixed eight accepted fixtures are enforced. Candidate absolute and unchanged
  target AND/spillover OR rules remain binding; baseline failure is diagnostic.
  Runtime checkpoint58 retains hashes and bounded first-run authorization.

## Iteration 55 — 2026-09-07 — Source review and public performance screen

- R03 complete-descendant reuse passed actual allocation RED/GREEN, focused
  correctness, broader tests, Clippy and zero-violation structure gates.
  Independent source review found no findings. Cohesive modules preserve the
  existing line limits; no policy allowance or evaluator changed.
- Two fixed-fixture balanced public quiet comparisons improved medians5.31%
  and12.38%. These are exploratory source-bound measurements, not full
  performance acceptance. Reviewed source is committed5eb0e339; full392-row/
  no-spillover plus cold/native/warm proof remain next. Failed keep bars stop
  the attempt.
- R01 exact reviewed0f0fe960 passed fast99082, PR80503 and final default docs
  24607, all exit0. Normal push93967 passed pre-push checks and published that
  exact head to PR185. One hosted diagnostic capture is now authorized; strict
  silence is unchanged and a pass without the raw cause remains inconclusive.
- Master remains3d9a255. No merge or new done card. A04 remains held for R01
  integration/current-master revalidation and unresolved full-card outcomes;
  preserved A05 and user-owned PR142 are untouched. Next context review57.
- Reconciled this current-master branch's stale queue entries with the live
  PR inventory and preserved A04 ownership ledger: R01/A04 active, Q02/Q04/Q06
  implemented but unmerged, A05 partial implemented. Fourteen done and eighteen
  unfinished; no readiness may be inferred from historical done snapshots.
- Metadata review corrected an overclaim:0.398ms is an instrumented bucket
  median, not a public-savings ceiling. Clone-only rejection remains a bounded
  prioritization decision. A05 uses the executor glossary's implemented state
  (patch exists, proof incomplete), explicitly partial and not PR-ready; it
  does not count as done or satisfy A07's dependency.

## Iteration 53 — 2026-09-07 — Reject small fix, test complete reuse

- R03's allocator-free timing deprioritizes the clone-only guard: observed
  strip-bucket median0.398ms, not a public-savings upper bound. Whole duplicate
  descendant construction measures
  about0.956ms plus unquantified teardown. A fresh uninstrumented static build
  and32-command paired control exposed substantial probe/host variation; no
  production savings is claimed from internal buckets.
- Authorized one test-first candidate for complete identical-descendant reuse.
  Direct-content policies, inheritance/reset, composition, serialized plans
  and plain/fast/compiled oracles must remain correct. Public before/after
  measurements decide retention; no threshold, row or execution-model change.
- R01 raw captures and seven replay controls distinguish ordinary external
  drops from real root/rescan/config signals. One27-test suite and20 prescribed
  exact local captures passed without reproducing the hosted failure. Approved
  one bounded, structure-compliant diagnostic-only hosted head after review
  and local gates; strict silence remains and tracing may affect timing.
- PR185's latest performance job passed8/8 at18.6063435ms versus20.1631345ms;
  the prior19.364834/17.6041085 failure remains. macOS external-config silence
  still fails; Linux/Windows were cancelled. Watch75876 terminated1, not pass.
- Master remains3d9a255; current release reconfirmedv0.3.0, published July2.
  Existing PR142/user work and A05 remain untouched. No new card is done.
- Next: source-bound R03 RED/GREEN/public comparison and reviewed R01 hosted
  event-origin capture. No merge is approved on incomplete platform evidence.

## Iteration 57 — 2026-09-07 — Enforce packet stop and review context health

- R01's explicit no-reproduced-cause stop rule now applies. Marked the card
  blocked on the named macOS raw event evidence or a specific maintainer
  native-readiness decision. No further diagnostic reruns. Preserved the
  reviewed loop-characterization design but did not authorize its speculative
  four-file refactor; it cannot close native acceptance by itself.
- R03 remains independent and executable. Exact final archive d2a9ca7 retains
  reviewed behavior source5eb0e339. VPS baseline bundle completed; original
  build68398 has live candidate rustc processes, not an expired-observation
  failure. No duplicate build or performance report has been launched.
- Context health: clean source ownership and exact handles remain explicit.
  Worktree inventory49; dry-run lists only two previously known prunable
  records, neither removed. Local disk14GiB free, so heavy work remains on VPS.
  A05 partial work, PR142 and unknown work are preserved. Q02 inspection found
  its existing administrator-protection decision and R03 hold unchanged;
  no premature rebase or extra implementation lane was started.
- Repeated native passes cannot establish an unobserved event cause. The
  existing packet stop rule and goal/performance/build skills already cover
  this decision; no new skill or AGENTS expansion is warranted. Next: full
  R03 report review and retain/reject decision, then the next dependency-ready
  integration. Goal remains active; this is not whole-goal external blockage.

## Iteration 56 — 2026-09-07 — Preserve failed gate, bound remaining diagnosis

- R01 exact0f0fe96 passed independent review, fast99082/PR80503/docs24607 and
  normal push93967. Single hosted run34094418980 completed: all platform tests
  and adoption/release jobs passed; performance alone failed. Observer68967
  exited1. No retry or merge; source remains unchanged.
- Cold many-scopes18.7323025ms versus18.4814565ms fails the unchanged gate.
  All392rows and native/warm artifacts are retained; diagnostic zero-sample
  rows are not passes. Prior failed and passed measurements are not erased.
- The external-config test passed but emitted no captured successful-test raw
  events, so its earlier cause remains inconclusive. Independent design review
  proposes a minimal actual-loop injection seam, not backend-quiescence proof.
  Preparation is bounded; no permissive rescan acceptance or further CI chase.
- R03 reviewed source5eb0e339 and separately reviewed metadata d2a9ca7 are
  committed clean. Two exploratory fixed-fixture quiet comparisons improved
 5.31%/12.38%; full exact-bundle392-row and cold/native/warm acceptance is still
  required. Original VPS build68398 remains the owned observation handle.
- Next: compare complete performance evidence, finalize the minimal R01 loop
  contract design, and perform context/repeated-failure review at57. No new
  done card, release, deployment, invitation or cleanup is claimed.

## Iteration 54 — 2026-09-07 — Bounded hosted diagnosis and context health

- R01's20 prescribed local captures all passed without reproducing the hosted
  full-rescan event. Seven replay controls preserve the actual policy branches;
  no silence assertion or source classifier was relaxed. Diagnostic-only
  source267857f adds explicit bounded/raw-path-safe capture and processing
  identity, with unchanged normalization output in a cohesive sibling module.
- Independent review passed the diagnostic diff after three concrete fixes.
  Focused units24, native tests29 and structure0violations passed; final exact
  head review/gates precede ONE fresh hosted capture. Passing without a cause
  remains inconclusive. No diagnostic-only commit is approved for merge.
- R03 has one authorized complete-descendant-reuse candidate, test-first,
  behind unchanged public keep bars. Clone-only optimization was rejected.
  Public before/after proof, not allocation counts alone, decides retention.
- Context budget is not exposed. Master3d9a255 and releasev0.3.0 were refreshed;
 49worktrees remain after the last inventory, with A05/unknown work preserved.
  Parent owns review/merge judgment; R01 local build cache and R03 remote
  candidate targets remain separately owned. A04 gates passed but merge is held.
- Repeated failures show why startup receipt, a successful policy result and
  a passing diagnostic run are not causal evidence. Existing goal/build/hook/
  performance/structure-fit skills and executable diagnostics cover this work;
  no new skill or AGENTS expansion is justified. Keep exact session exits and
  avoid broad or guessed cross-worktree reads.
- Next: R01 exact-head gates/one hosted capture and R03 first public candidate
  measurement. No new completion, threshold waiver, release or deployment.

## Iteration 52 — 2026-09-07 — Strict silence and performance attribution

- R01 exact reviewed3ec25fd passed fast42820, PR51849 and final default docs22923;
  normal push75325 succeeded and PR185 now targets that head. Fresh macOS CI
  passed root/directory corrections but failed external-config silence,26/27.
  Linux/Windows were cancelled. The unexpected full-root report has three
  coalesced events; actual diagnostic paths are unknown, not invented.
- Approved diagnosis before another repair: preserve strict silence, capture
  failure diagnostics, discriminate genuine external-scope regression from
  delayed in-scope setup. Independent review is designing that experiment;
  closed169's permissive acceptance is not revived. No green retry or merge.
- R03 structural probe confirms800 reset children, one inherited root, zero
  composition targets and801 shared Arc observations (strong count3).
  Descendant construction adds9,603 allocations. Counting overhead prevents
  a timing conclusion; an allocator/counter-free timing variant is required
  before retaining any optimization. All accepted performance gates remain.
- A04 reviewed9c9a896 passed fast8047 and PR25099; its PR update/rebase remains
  held behind R01/R03. Full hook-path/host acceptance is still unfinished.
  Runtime checkpoint51 records context/repeated-failure review and inventory49;
  no new skill, unknown cleanup, release or public action was warranted.
- Next: decisive R01 event-origin evidence and R03 counter-free timing, then
  the smallest independently reviewed repair supported by those observations.

## Iteration 46 — 2026-09-07 — Post-merge watch evidence correction

- Master `3d9a255f832733082c864edf703fc98c854e7f6e` has the same tree as
  reviewed PR #183 head `9d4bf50`, whose 24 hosted checks passed. Its later
  Rust CI run `34084659580` failed macOS job `101626309708`; Windows was
  cancelled, not passed. Performance, security and documentation passed.
- The failure compared diagnostic paths `src` versus `src/BadName.ts`, not
  the report's changed paths. Read-only independent tracing found that the
  harness consumes one permitted successful full-rescan predecessor from
  stdout but leaves its diagnostic in the separate stderr queue. Folder
  events force full checks and cannot explain the incremental report that
  already passed. Precise OS notification timing is not established.
- Reopened R01 in a clean current-master worktree at
  `/Users/nroth/.codex/worktrees/assura-r01-diagnostic-association`. Require a
  deterministic association RED, bounded report/diagnostic accounting and
  negative controls for missing diagnostics, disconnection, wrong scope and
  extra predecessors. Strengthen actual violation-path evidence; preserve
  config reload, pathless/excluded rescans and overflow safety.
- A04's independently reviewed `d42094c` passed exact-head fast (session 29959)
  and PR (56973) tiers, both terminal exit 0. Its ownership slice stays separate
  and cannot merge by waiving the current-master failure. A05 is untouched;
  R03's fresh measurements remain diagnostic evidence, not an optimization.
- Inventory is 48 worktrees after the new isolated repair. The two pre-existing
  prune-dry-run findings and unrelated paths remain untouched. Next: reviewed
  R01 correction, hosted proof, then rebase/revalidate the A04 ownership slice.

### Review decisions and subsequent hosted evidence

- A04 PR #184 is open at reviewed `d42094c`, with explicit merge holds. Its
  Linux job `101630975101` in run `34086323317` failed executable raw-byte hook
  launch with OS 26 (`Text file busy`); 23/24 ownership tests and 10/10 lifecycle
  tests passed. macOS and Windows were cancelled by fail-fast, not independently
  passing. The owning investigation must distinguish fixture/process lifetime
  from product publication behavior; no retry-to-green or weaker shell-launch
  substitute is approved. Existing local passes remain historical evidence.
- Independent performance review found no justified optimization from the
  current profile. R03's remaining bounded experiment is two unchanged-source
  16-iteration comparisons in a verified quiet interval, keeping the existing
  balanced per-iteration tool alternation and every row. Serialize this after
  Linux hook diagnostics. Two passes can establish no reproduced current
  regression, not prove historical runner noise. Strict cold 2x is not a gate.
- Next A04 slice is effective Git hook-path resolution and preservation at that
  path, with real Git-event proof, after ownership integration. Current source
  has no supported manager adapter; fixture-manager metadata is not integration
  proof. Relative/worktree/absolute configuration, configuration errors and
  symlink ancestry require explicit tests. Manager support and host runtime
  proof remain separate acceptance gaps; do not silently count them complete.

## Iteration 50 — 2026-09-07 — Fixture isolation and bounded attribution

- A04 test-only d79a80c removes the proven in-process inherited-writer window
  using a fixture guard. Linux 27/27, macOS 22/22 and focused lint passed;
  independent review passed. Exact hosted holder remains unknown. The original
  OS 26 failure and intentional direct-execution negative control remain.
- Created clean current-master R03 worktree `assura-r03-scope-rules-attribution`
  on its matching goal branch, 3d9a255; session 82658 exited 0 and workflow gate
  is ready. Inventory is now 49. No unknown or dirty path was removed.
- Approved diagnostic-only counters/timing to test duplicate exact/descendant
  naming-plan construction. Actual generated fixture identity, inherit-reset
  counts and public cold cost must be established before an optimization.
  Parent retains review authority; VPS work must preserve immutable binaries
  and unrelated workloads. No benchmark threshold or supported row changes.
- Next: R01 final review/gates, R03 measured hypothesis decision, A04 final
  metadata/rebase/gates. A05 remains untouched; no card is newly done.

## Iteration 49 — 2026-09-07 — Complete diagnostic-consumer correction

- PR #185 head 1852b61 passed final local PR tier 82802 (exit 0), then failed
  a distinct hosted root-debug assumption. The root path is `[""]`, not `[]`;
  hosted report mode/count are unknown. No stale-predecessor explanation was
  invented. Linux/Windows fail-fast cancellations are not passing platform proof.
- R01 fb724fd audits all diagnostic readers and validates complete FIFO batches
  with separate full-root and incremental contracts. Actual reader RED/GREEN,
  26 watch tests, 20 watch units and 40 stop-on-failure focused runs passed.
  Exact support-module structure allowance retains limits and negative control.
  Independent review and exact-head gates are pending; PR remains held.
- Hosted many-scopes performance failed at 19.364834 ms versus LS-Lint
  17.6041085 ms, 7/8 accepted comparisons, 392 retained rows. Native/warm passes
  do not clear it. Wider-stack profiling produced 3,430 samples with zero lost
  but still unreliable callers; attribution must use a discriminating probe.
- No green rerun, release, deployment or scope reduction was used for closure.

## Iteration 48 — 2026-09-07 — Hook diagnostics and context health

- PR #184 `d42094c` failed Ubuntu executable-hook launch with OS 26
  (`Text file busy`); 23/24 ownership and 10/10 lifecycle tests passed.
  macOS/Windows were cancelled, not passed. Five adoption smoke lanes and
  performance passed but cannot replace the failed suite/platform proof.
- Passing Linux traces show close-before-rename and direct wrapper/sidecar
  execution, excluding a persistent leaked writer in those observations, not
  a transient race in the failing hosted schedule. A writable-inode negative
  control reproduced errno 26. No production source cause is established.
- Approved bounded failure-only Linux diagnostics: invoke the installed
  wrapper once, capture executable identity and same-inode writer evidence,
  then return failure. Exclude command lines/environments, bound enumeration
  and label missing/racy data. No retries, sleeps or shell-launch bypass.
- R03 quiet probe on `vps-9cb01956`, session 10542 exit 0, found continuing
  Node/Temporal/background activity. No benchmark was started, no services
  stopped, and no exclusive-runner or historical-noise claim was made.
- Context level: not exposed. Master `3d9a255`; reviewed R01 `1852b61` has
  final PR-tier bootstrap running; A04 owns diagnostics; A05 is preserved;
  inventory remains 48 worktrees. Repeated failures require exact stream
  association and discriminating execution evidence, not green reruns.
  Executable helpers and existing goal/local-build/performance skills are the
  appropriate homes; no new skill or AGENTS expansion is warranted.
- Next: R01 integration, A04 diagnostic review, current-master rebase and
  fresh hosted proof without waivers.

## Iteration 47 — 2026-09-07 — R01 review correction

- R01 `0ccebea` reproduced stale diagnostic association before repair, then
  passed 20 prescribed stop-on-failure runs, the 20-test watch suite and
  library checks. Parent review added actual checked-path/violation proof.
- Independent review found a 20 ms queued-test deadline could spuriously fail
  and broad panic checks could accept the wrong rejection. `1852b61` uses
  the standard deadline and specific reasons; source/docs re-review PASS.
- Exact-head fast session 52386 exited 0. PR session 46503 passed Rust,
  structure, evidence, target-state and Clippy, then failed because the docs
  helper expected cwd/target while Cargo used the preserved explicit cache.
  Approved real ignored artifact directories with hash-equal binaries;
  unchanged-head PR rerun 82802 remains pending. The first failure is retained.

## Iteration 45 — 2026-09-07 — Safe legacy upgrade and context health

- Fresh-source tests confirmed the unsafe exact legacy wrapper, including a
  real shell-substitution sentinel. Source fix `258282a` now distinguishes
  ownership from current safe content and upgrades proven legacy pairs by
  default through both direct and bulk APIs. It preserves custom/drifted pairs,
  removal authority and transactional rollback. Focused, library, Clippy and
  structure checks passed; final-head full gates and re-review remain required.
- Context health: the budget is not exposed. Current base is merged A03
  `3d9a255`; A04 owns the active repair; A05 remains preserved; R03 retains
  fresh hosted and VPS passes without claiming an optimization/noise amendment;
  47 worktrees remain after owned merged cleanup, with unrelated state intact.
- Repeated issue reviewed: exact ownership was incorrectly conflated with safe
  readiness. The distinction now lives in executable legacy-upgrade tests and
  the existing harness hook spec. The existing hook skill already routes that
  spec, so another operational skill would duplicate it rather than prevent
  rediscovery. Final approval must explicitly review legacy as well as newly
  generated wrappers.
- Next: finish independent review and final committed-head fast/PR gates, then
  a fully gated ownership PR. A04's effective hook path/manager integration and
  host permission/runtime evidence remain subsequent current-master slices.

## Iteration 44 — 2026-09-06 — A03 integration and legacy-path correction

- PR #183 merged as `3d9a255f832733082c864edf703fc98c854e7f6e`. All 24
  hosted checks passed on independently reviewed and locally PR-tier-tested
  `9d4bf50643155e8cd14e37b1a601f5a7b2a7399f`; the parent also reran all three
  focused tests and target-state successfully. Fetch and ancestry verification
  exited 0. No release, tag or deployment was performed.
- A04 was cleanly rebased onto that merge, resolving only the explicit task
  branch binding to `goal/a04-hook-ownership-repair`; workflow gate is ready.
- Linux negative controls on `783ff05` with `07599a9` tests failed both actual
  raw-byte invocation and lossy legacy ownership assertions, session 17578 exit
  101, two failures, zero ignored tests. Candidate suites are running separately;
  this does not replace final rebased gates.
- A parent-triggered legacy-path review superseded the previous clean A04
  verdict: exact UTF-8 legacy wrappers at shell-metacharacter paths still report
  ready/current while executing command substitution. The owning repair must
  distinguish proven ownership from safe/current state and upgrade exact legacy
  wrappers by default. A04 remains unapproved; no dangerous fixture content was
  merged. This contract is now in the bounded repair plan.
- Preserved A03 runtime evidence under this goal's runtime area, then removed
  its clean merged worktree and exact-SHA-leased remote branch. The first local
  `branch -d` was correctly refused from the older strategy checkout because
  that checkout lacks the merge; deletion was retried only from the descendant
  A04 checkout after verifying ancestry against both HEAD and `origin/master`.
  No force deletion was used. Inventory is 47 worktrees; shared build cache,
  unknown paths and both pre-existing prune-dry-run findings remain untouched.

## Iteration 42 — 2026-09-06 — Acceptance correction and queue reconciliation

- The preceding review was progress: it found concrete A04 contract violations
  on master `6d613f8`, changing the next action from A05 expansion to A04 repair.
  A04 is reopened; its prior merge evidence is retained with explicit supersession.
- Q02/#158, Q04/#164 and Q06/#166 are implemented but unmerged, not unowned pending.
  Current failed/canceled checks are recorded in their evidence. A05's six dirty
  paths remain untouched in its owned worktree; partial implementation is not proof.
- Closed obsolete goal-owned PR #169 after comparing its test-only relaxation
  with merged #173 (`b3002b1`, ancestry command exit 0). Current master retains
  the stricter no-event assertion and fixes external rescan filtering itself.
  The closed PR and its unmerged branch retain historical evidence.
- Inventory: 52 worktrees before this repair, plus its one new isolated worktree;
  32 prior goal branches. Clean/dirty state and master ancestry were checked.
  Prune dry-run found the same two pre-existing missing-path records; neither
  was pruned. Build-cache ownership is checked before any goal-owned cleanup.
- Context health: context budget is not exposed. Known state is now routed by
  the correction plan, evidence and exact-ID queue records. Repeated observation
  timeout confusion requires retaining tool session IDs and terminal exits;
  no duplicate Cargo run should be started against a live session.
- Reusable workflow review: use the existing local-build guidance for platform
  recovery and the correction plan for this acceptance repair. Session handling
  is a general orchestration rule, not another Assura-specific skill.
- Next: independently reviewed ownership repair, then effective hook manager
  integration and host evidence; R03 remains an independent attribution task.

### Iteration 42 execution updates

- Closed #169 without merging its obsolete test relaxation; its unmerged branch
  remains. Removed six clean, merged goal-owned documentation worktrees and
  local branches: a01-postmerge-record (`0cf2eb0`), a03-postmerge-closure
  (`3c1b92c`), p01-postmerge-closure (`812aa35`), q07-postmerge-record
  (`d0454a3`), r04-postmerge-closure (`9a16ba5`), w01-postmerge-closure
  (`35d7628`). Each cleanup rechecked ancestry and exited 0; committed files
  remain recoverable from Git. Inventory is now 47 worktrees. Unknown paths,
  the two old prunable entries, A05, and the worker's A03 build cache are untouched.
- The generic SDD scratch path failed Assura's root policy (7 violations).
  Moved only this goal's scratch to `.trellis/.runtime/a04-orchestration`.
  No config policy changed; the subsequent structure check exited 0 with zero
  violations. Ruling: reuse the repository's runtime area, not a new root
  exclusion; cost if wrong is a scratch-pointer update, not relaxed policy.
- R03 runner availability was rechecked using current compiler processes and
  sampled CPU use, not process names in stale evidence. Its isolated immutable
  master diagnostic build is now live; the default nightly compiler and shared
  services prevent claiming hosted stable-toolchain parity. See R03 evidence.
- The three still-present remote documentation branches (A01/A03/Q07 above)
  were deleted with exact expected-SHA leases after ancestry verification;
  pre-push validation and deletion exited 0. The other three were already absent.
  A separate current-master A03 target-state repair worktree brings the inventory
  to 48. The verifier still required retired questionnaire text, independently
  reproduced on unchanged master; correcting its owning contract is required,
  not a waiver of `cargo xtask pr`.
- Ownership review found arbitrary-byte custom hooks and Unix project paths
  were not safely preserved/resolved. The file-content repair is committed;
  raw path invocation and deterministic rollback checks remain under correction.
  A04 stays active after this slice: effective hook paths and host evidence are
  still required. One worker bypassed the commit hook during cache contention;
  this was explicitly rejected as a procedure, recorded, and does not waive any
  final gate or approval requirement.
- A03 fast checks completed with exit 0 before cloning the idle Cargo cache
  into A04's own target. The clone exposed stale shared fingerprints, so the
  first A04 run is invalid evidence; only its local Assura package artifacts
  are being cleaned before the actual RED run. Cache separation avoids lock
  contention but does not prove source/binary identity.
- The immutable master VPS baseline also passed with the exact hosted Rust
  1.98.1 compiler: eight accepted no-slower rows, independent native gate, and
  all five warm p95 rows. All command exits and hashes are in R03 evidence.
  This is not a source optimization or proof that historical failures were noise.

## Iteration 33 — 2026-09-06 — A01 current-master integration

- PR [#162](https://github.com/rothnic/assura/pull/162) merged as `fdd0e76426c9ca6916fa72cdb3948378ad3a92e3`; a fresh fetch proved that merge is reachable from `origin/master`. The exact independently reviewed head was `1522352cb8b817620c4ea773780877332e122919`.
- Rust CI, Documentation, Security Scope, and GitGuardian hosted checks passed. The scope-directed Security Audit job was skipped and is not represented as a passing test. A01 is now done as the partial evaluator contract, not as end-to-end initialization acceptance.
- Context health: the long release/build commands in isolated worktrees can outlive an output window, so completed exits are being re-captured individually rather than inferred. This is an execution-observation issue, not a new reusable project skill. Next: finish current-master Q04 verification/review while A02 is dependency-ready.

## Iteration 32 — 2026-09-06 — A01 evaluator trust-boundary closure

- Rebased A01 onto current master `6f72bf3`, then independent review identified and the candidate repaired absolute candidate-binary enforcement, named-rule matching for negative probes, required negative policy evidence, stdout/stderr zero-test detection, and required SHA-256 fixed-prompt provenance. The final reviewed SHA is `83687478a7b399e318f917edfa5641cce4ad95d1`; final independent review found no remaining findings.
- Focused RED tests were observed for every review condition. The final local evaluator suite passed 27 tests; Python compilation, repository structure check, evidence policy, and the 48-page documentation build passed. Rust, TypeScript, and Python policy-only partial controls all recorded matched named-rule negative probes and remain ineligible by scope; those runs do not assert unrequested native, guidance, or hook dimensions. A separate full Rust control records native, guidance, and hooks as unavailable; TypeScript/Python full-run evidence remains separately incomplete.
- Context health: earlier Q02 and unrelated-card evidence show R03's no-slower gate can fluctuate on the same fixture, so hosted proof remains mandatory. Existing evaluator and performance workflows already cover the new findings; no new skill is warranted. Next: await the exact-SHA hosted matrix, then merge only if every required job passes and the candidate remains current-master based.

## Iteration 33 — 2026-09-06 — A03 guidance-evaluator evidence repair

- A03 implementation merged in `2e882ae` after independent review and a fully green hosted retry. Its required partial evaluator run exposed the remaining honest gap: Contract v1 named `guidance` but had no assertion type, so it reported `unavailable` rather than a false pass.
- A focused evaluator repair now adds optional fixture-owned textual guidance assertions, with passing, missing-fragment, and unsafe-path tests. The evaluator suite passed 30 tests and Python compilation passed. The exact A03 binary-backed disposable proof is pending: local storage had only 118 MiB free and Cargo failed with `No space left on device`; the local `vps-dev` SSH alias was unavailable. The failed temporary build and fixture were removed, restoring 128 MiB, still inadequate. This is an environmental evidence gap, not a passing result.
- Context health: the repeated issue is constrained local disk, already visible in prior A01 docs observations. No new reusable skill is warranted; next is an adequately provisioned runner for the exact current-master binary proof, then independent review and hosted gating of this repair.

## Historical index

- Initial B00/R01 iterations 1–4 are preserved in
  [`progress-early-history.md`](progress-early-history.md) so this live journal
  remains within its configured structural limit.

## Iteration 5 — 2026-09-06 — R01 current-master integration

- PR #155 (`ca3d4efb1ca3366bb157074d81601693347eb723`) passed the full hosted matrix, including macOS, Ubuntu, Windows, installer/adoption, release, evidence, and performance checks.
- Merged as `fc5dd2214483b463be6c8a0b6823adf810b36388`, then verified reachable from `origin/master`. The card remains awaiting a separate post-merge evidence/state closure.

## Iteration 6 — 2026-09-06 — Active repair context health

- Q03's hosted Windows failure is a test-only extended-path formatting mismatch; its correction is under independent review and must honestly dispose of the Q03-introduced loader-size advisory before it can proceed.
- R03 is independently reproducing the many-scope release comparison. It has preserved the failing hosted rows and is investigating launcher/scope-plan attribution; the evaluator and threshold remain unchanged. `vps-dev` is currently unavailable, so it is not counted as verification.
- Context health: current-master integration and independent-review gates are functioning. No new reusable skill is warranted; the existing performance and structure-fit skills cover the active decisions. The next action is Q03 repair review/hosted proof, then R03's evidence-backed outcome; R05 waits for both its current-master refresh and the performance gate.

## Iteration 7 — 2026-09-06 — Q03 integration and R03 hypothesis rejection

- Q03's Windows path-contract correction and loader-size remediation passed independent review and the complete hosted matrix, then merged as `7a06b345d47521ede6b5e6c7cdc06e1128883774`.
- R03 rejected, rather than merged, a locally favorable HashSet experiment because stronger same-fixture VPS public-command evidence records a regression. The experiment was reverted with no policy/evaluator change.
- Next: merge this evidence closure after review; then refresh R05 onto current master and select only the next dependency-ready card. The exact R03 protocol remains required for a genuinely distinct hypothesis.

## Iteration 8 — 2026-09-06 — R05 installer integration

- R05's first current-master matrix exposed a real Alpine fixture collision; the archive installer itself had completed. The repair retained dedicated Alpine musl coverage under `/tmp` and removed only the duplicate generic collision path.
- Independent review also found and repaired a pair-preservation hole during the second backup move. Unix and hosted Windows controls now prove both old companion binaries survive that injected failure.
- PR #153 merged as `2ee15e42c5b3bfdfcaf8c2ba8a2aa8f789c78356` after its full hosted artifact matrix passed. Next ready cards are Q02, A01, W01, and F01; choose one behavior surface at a time after this evidence closure is reviewed.

## Iteration 9 — 2026-09-06 — W01 CTA contract and context health

- W01 started in isolated current-master worktree `assura-w01-marketing-ctas` at `09310aa`. A route-wide Playwright fragment contract first failed on the real `/compare/ls-lint/#onboard` dead destination, then passed after the shared focused-page destination was corrected to `/#onboard`.
- The complete marketing suite passed 105 tests. `cargo xtask docs`, repository structure check, and evidence gates passed. Manual keyboard checks confirmed both desktop and 375px About-page Start actions invoke the setup dialog while retaining the real no-JavaScript homepage target.
- Context health: the only repeated execution friction was an overlapping cold Cargo docs invocation; the active first invocation was allowed to finish and the duplicate waiter then completed, with no changed-command retry or output claim. Existing isolation, test-first, browser, and evidence workflows are sufficient; no new skill is justified.
- Next: independent review of the exact W01 candidate, then hosted PR proof. Q02 remains separately in its final hosted matrix and must not be merged until every required job is terminal green.

## Iteration 10 — 2026-09-06 — W01 current-master closure

- PR #159 merged as `8eaa31434528ba2a877284929723fba708d944b3` after independent review and all targeted hosted website/evidence/scope/security/external-build checks passed. The skipped Rust, release, and performance jobs were outside the website-only scope and are not recorded as passing.
- Fetch confirmed the merge SHA is reachable from `origin/master`; W01 is now done. No deployment, release, tag, or public communication was initiated.
- Next: rebase the independently prepared F01 kit onto current master before review, while Q02 remains held by the honest R03 performance gate failure.

## Iteration 11 — 2026-09-06 — F01 pilot-kit preparation

- F01's interview kit was prepared in isolated worktree `assura-f01-pilot-kit`, independently reviewed, and corrected to exclude re-identification data from repository evidence. The consented, Nick-controlled follow-up boundary is explicit; no external outreach or participant selection occurred.
- The rebased candidate `877acebb2e8ef53cfb638acd14c290f46e2e43fc` passed docs, structural, and evidence gates locally against `9b032d3`. F01 remains blocked on Nick's explicit authorization to select and invite participants; that external authority was not inferred.
- Next: independent final review of the rebased exact SHA, then documentation-only PR evidence. Q02 remains blocked by R03's honest hosted performance regression.

## Iteration 15 — 2026-09-06 — A01 evaluator contract and context health

- A01 is active in isolated worktree `assura-a01-init-evaluator` from current master `80f42e9`. Test-first evaluator contracts now cover permissive false-green negative probes, partial-scope ineligibility, preservation hashes, unavailable native commands, structural paths, and required positive probes.
- Context level: not exposed. Relevant working facts are: (1) A01 owns a Python stdlib evaluator and fixtures, not agent orchestration; (2) each covered contract was observed RED before its minimal implementation; (3) Q02 remains held by the honest R03 performance failure; (4) F01 is merged preparation but blocked on Nick's outreach authority; (5) no release, deployment, tag, or public communication authority has been used.
- The repeated evaluator contract pattern belongs in A01's tests and documentation rather than a reusable skill: it has not yet been rediscovered outside this one card. Continue with fixture-backed hook/zero-test/wrong-cwd contracts before broader verification and review.

## Iteration 18 — 2026-09-06 — A01 fixture freeze and context health

- A01 now has a Python-stdlib evaluator contract slice, frozen Rust/TypeScript/Python baselines, tracked existing-config and hook-manager variations, and a boundary document. Ten focused tests cover false-green, partial, structure, preservation, positive/negative policy, native unavailable/zero-test, hook-path, and schema rejection paths.
- Context level: not exposed. Repeated friction was limited to the isolated website build's missing dependencies and Git ignoring `.githooks` fixture content. The first is recorded as unavailable rather than passed; the second was corrected to tracked hook-manager metadata and the ignored goal-created directories were removed. No reusable skill is warranted because neither procedure has recurred outside this card.
- Next: complete known-good, wrong-cwd, idempotence, and publication-redaction contracts; validate frozen fixture hashes and real candidate binary behavior. A01 remains active and unreviewed.

## Iteration 21 — 2026-09-06 — A01 provenance and context health

- The evaluator now records contract and candidate-binary SHA-256 hashes, rejects unsupported contract schemas, proves a known-good full contract, and reports a missing declared cwd as unavailable evidence. The focused suite has 12 passing tests.
- Context level: not exposed. The broader `cargo xtask fast` cold build was allowed to finish but its terminal output was not captured; it is explicitly inconclusive. Its 2.3 GiB ignored worktree `target/` cache was then removed with a Git-scoped cleanup after verification, restoring disk headroom without touching source or other worktrees.
- Repeated lessons remain card-local: fixture provenance and generated-cache recovery are already covered by evidence discipline and safe exact-target cleanup. No new reusable skill is justified. Next: implement remaining A01 idempotence/publication-redaction contracts and real candidate-binary fixture evaluation before independent review.

## Iteration 24 — 2026-09-06 — A01 three-stack controls and context health

- Frozen Rust, TypeScript/Bun, and Python contracts each now complete a real full evaluator control using the identified installed Assura 0.4.0 binary. Each preserves declared source hashes, passes a positive policy check, runs the stack-native test command, and rejects a seeded naming violation in a disposable copy.
- Context level: not exposed. The three controls exposed duplicate YAML keys in newly authored fixture policies; each failure was retained as RED evidence and repaired without changing evaluator acceptance rules. The previous fast-tier run remains inconclusive and is not reclassified.
- No reusable skill is justified: the repeated issue was a one-card fixture-authoring pattern now covered by the evaluator tests and durable contracts. A01 remains active pending lifecycle/idempotence, publication-redaction, broader repository gates, and independent review.

## Iteration 25 — 2026-09-06 — A01 publication-safe evidence

- A focused red test proved the evaluator lacked a publication artifact. The new optional `--public-output` writes a separate aggregate result that omits fixture and command identifiers, hashes, paths, cwd values, stdout, and stderr; the private `--output` record retains diagnostics.
- The redaction test passes with a synthetic command-output secret present only in the private artifact. The full focused evaluator suite passed 13 tests, and `git diff --check` passed. A01 remains active: idempotence and real lifecycle proof are still required, and no broader or hosted gate is claimed from this iteration.

## Iteration 26 — 2026-09-06 — A01 dimension-completeness correction and context health

- A real Rust control exposed a false completion signal: empty guidance, hook, and native assertions were reported as pass. A focused red test now requires every requested but uncontracted dimension to be `unavailable`; the evaluator records per-dimension pass/fail/unavailable states and unavailable evidence is a critical failure. The focused suite passes 15 tests.
- The exact current Rust control now exits 1 with policy/structure/preservation/idempotence pass and guidance/hooks/native unavailable. Earlier TypeScript/Python acceptance-pass observations are retained as historical pre-correction controls and must be rerun; they are not current acceptance evidence.
- Context health: not exposed. The key repeated failure was empty-contract evidence being mistaken for a passing dimension; this is now encoded in the evaluator, not a new skill. A01 stays active; next is completing honest contract evidence for missing dimensions and coordinating actual hook lifecycle proof with A04.

## Iteration 27 — 2026-09-06 — A01 timeout evidence and context health

- A real focused subprocess test first showed that a timed-out policy probe raised instead of emitting evidence. The evaluator now applies a 30-second timeout to policy and native commands, records timeout as failure with captured private partial output, and refuses to treat a timed-out negative probe as policy rejection. The focused suite passes 16 tests.
- Context health: not exposed. Two related evaluator gaps (empty dimension evidence and unhandled timeout) were resolved as compact, reusable contract behavior inside the card; no cross-card operational workflow was rediscovered, so no skill is warranted. A01 remains active pending a review of its remaining contract boundary, broader validation, and independent review.

## Iteration 28 — 2026-09-06 — A01 independent review repairs

- Independent review of `2d950a1` found three high-severity false-green risks: sequential negative mutations contaminated one another, accepted negative probe IDs could leave policy state as pass, and every full run lacks contractable guidance evidence. It also found malformed-contract/path escape and incomplete zero-test detection gaps.
- Focused RED/green tests now prove fresh disposable copies for each negative probe, policy failure mapping, named missing-field errors, cwd escape rejection, and pytest zero-item detection. The focused suite passes 20 tests. The guidance issue remains an intentional unavailable boundary because Contract v1 provides no guidance assertion; it must not be hidden or treated as A01 end-to-end acceptance.
- Next: commit the repairs, obtain re-review of the new SHA, and then decide whether A01's evaluator-only acceptance is sufficiently evidenced or needs an explicit contract-version decision before broader validation.

## Iteration 29 — 2026-09-06 — A01 re-review crash repair

- Re-review confirmed the prior concrete defects were fixed but found one remaining medium error path: a nonexistent relative policy cwd crashed before writing evidence. The focused RED test now passes with an `unavailable` command record and output artifact; the evaluator suite passes 21 tests.
- Guidance remains structurally unavailable in Contract v1 because the packet names the dimension but defines no executable guidance assertion. This is retained as an explicit non-acceptance boundary, not patched around by inventing an unreviewed schema extension. Next: commit this repair and use the final exact SHA for any broader-gate/review decision.

## Iteration 30 — 2026-09-06 — A01 broader local evidence and context health

- Final independent re-review found no new defects in `ba2d865`; it confirmed unavailable policy cwd/executable evidence is non-crashing and the earlier mutation-isolation, policy-state, contract-validation, and zero-test repairs remain intact. `cargo xtask evidence`, Python compilation, the 21-test focused suite, and a final Assura structure check with zero blocking violations passed.
- `cargo xtask docs` exceeded its observation window, then had no child process while available disk fell from 2.3 GiB to 1.1 GiB. It exited before its terminal result was observable, so it is recorded as inconclusive and was not retried. Context health: not exposed. Repeated low-disk build observation is covered by existing evidence discipline; no new skill is warranted. Next: resolve the explicit Contract v1 guidance-evidence decision before calling A01 verified or preparing a PR.

## Iteration 31 — 2026-09-06 — A01 scope reconciliation

- The planning review resolves the Contract v1 concern: A01 supports partial evaluator dimensions; A03–A05 own guidance/hook/native closure and A07 owns full acceptance. Unavailable dimensions therefore remain visible and non-accepting, while A01's card-level acceptance is its independently reviewed ability to catch false-green policy, preservation, hook, and native failures.
- A01 is now `verified` in the backlog with two independent review passes, 21 focused tests, evidence gate, compilation, and zero-blocking structure proof. The docs gate remains inconclusive under low disk and must be decided by hosted PR evidence; A01 is not done until its reviewed PR is merged and reachable from master.

## Iteration 4 — 2026-09-06 — R01/R02 integrated proof and post-merge closure

- PR [#144](https://github.com/rothnic/assura/pull/144) merged as `dcb1fb57ba100f77a7cb7e48c4f14507d3106231`; it was confirmed reachable from `origin/master` after fetch.
- Final reviewed head `4741e560c28620a0a8813f9a8635ae52c67ca3cc` passed all hosted checks in Rust CI run [34004956631](https://github.com/rothnic/assura/actions/runs/34004956631), including macOS/Linux/Windows suites, four-platform adoption, installer smoke, release bundle, and Performance Report. R01 and R02 are done.
- Context health review: three Windows reruns exposed fixture-only `CRCRLF` construction errors; each was preserved as RED evidence, independently reviewed, and narrowed until the actual Windows suite passed. The repeated discovery is specific to this temporary contract fixture; no new reusable skill is warranted. R07 remains active because current self-check still reports 18 advisory line-length findings that need individual dispositions.
- Next ready card: R03 (many-scope performance repair). P01 is also locally ready but remains a separate documentation integration.

## Iteration 5 — 2026-09-06 — P01/R03 integration and R04 MSRV closure

- PR [#146](https://github.com/rothnic/assura/pull/146) merged P01 scope evidence as `380b9bc889b8c653b57da375bd7bf06f174f8f2f`; PR [#147](https://github.com/rothnic/assura/pull/147) merged R03's truthful performance evidence as `3ff8c889e18381e83cc960803ccf9ddfba35f1d3`. R03 remains blocked, not failed-closed: the exact GitHub Linux comparison regressed and a comparable Linux rerun cannot yet be run because `vps-dev` is unresolved.
- PR [#148](https://github.com/rothnic/assura/pull/148) merged R04 as `e6f3a8e70068ea44a51a6d2626eaece1256e28b2`, then `git merge-base --is-ancestor` confirmed it reachable from `origin/master`. The merged Rust `1.86.0` floor is backed by a hosted all-features MSRV Clippy lane, focused `git-signals` proof, all required platform/adoption/installer/release/performance/docs/security gates, and two independent-review passes on the final PR tip.
- The initial hosted R04 run exposed 16 current-Clippy findings; the first correction exposed that the public optional `git-signals` feature did not meet the stated MSRV. Both findings were repaired instead of suppressed or scoped away, and the evidence records the exact failed and corrected SHA lineage. Next ready work remains R05 inspection and R07's individually dispositioned advisory backlog; R03 requires an external comparable Linux runner.

## Iteration 6 — 2026-09-06 — post-merge queue reconciliation and context health

- P01 is now recorded done only because PR #146's `380b9bc889b8c653b57da375bd7bf06f174f8f2f` is reachable from current master and its source-scoped hosted gates passed. Skipped Rust behavior jobs remain explicitly skipped, not counted as tests.
- R05 selection inspected the existing user-owned PR #142 without modifying it: it is stale against current master and retains a failed macOS test, failed Alpine adoption job, and cancelled Windows test. A new isolated current-master port owns any further R05 work.
- Context level: not exposed. Current working facts: (1) R01/R02/P01/R04 have merged evidence; (2) R03 is a documented external comparable-Linux-runner block; (3) R07 has 18 advisory findings requiring individual dispositions; (4) R05 is being ported from #142 without taking over its stale branch; (5) no tag/release/deploy authority has been used. Repeated lessons are already covered by the existing goal-execution and structure-fit skills, so no new skill is warranted.

## Iteration 12 — 2026-09-06 — Q03 queue reconciliation

- Q03 evidence records PR #154 merged as `7a06b345d47521ede6b5e6c7cdc06e1128883774` and reachable from master, but backlog.json still said `implemented`. A dedicated documentation-only handoff corrects that stale queue state to `done`; no product behavior, performance threshold, or evaluator result was changed.
- This makes Q04 dependency readiness accurately inspectable. The handoff remains separate from A01 while PR #162 is held by the already-blocked R03 performance gate and a macOS watch failure.

## Iteration 16 — 2026-09-06 — Q05 maturity containment discovery

- Q05 began in isolated current-master worktree `assura-q05-maturity-containment` at `25a1415`. Consumer enumeration found the score detector, report renderer, and CLI maturity configuration have no active CLI caller; only `MaturityLevel` remains used by internal experimental constraint severity and trigger helpers.
- A focused observation contract was added test-first: an empty workflow directory does not establish CI configuration, local CI configuration remains `unverified` for execution, a bare `pyproject.toml` does not establish Black configuration, and additional package manifests are reported only as observations. The test first failed because the observation API did not exist.
- The local green attempt is currently blocked by `ld` error 28 after goal-owned ignored build directories were safely reclaimed; this is not recorded as a passing test. Q04 PR #164 remains unmerged while installer, adoption, and performance jobs are still in progress.

## Iteration 17 — 2026-09-06 — R03/Q06 performance hold and queue health

- Q06 PR #166's repeat Rust CI run `34023443201` again failed only the unchanged Performance Report gate: `many_configured_scopes_regression` was `assura-cli 19.492 ms > ls-lint-cli 19.115 ms`; all other hosted checks succeeded. The release-readiness extraction is not treated as the cause or merged around the gate.
- The reachable `vps` benchmark host has the canonical fixture but was running unrelated Cargo work. Because paired cold-run attribution requires a quiet comparable runner, no benchmark or speculative optimization was started. R03 remains blocked on that named environment condition and the prior HashSet hypothesis remains rejected.
- Queue health: Q02's existing candidate has correct local governance evidence but remains blocked on independent reviewer/branch-protection authority and the same required performance gate. Existing performance and worktree procedures cover this repeated decision; no new skill is warranted. Next independent ready card: A01.

## Iteration 21 — 2026-09-06 — Q07 cache recovery attribution repair

- Independent review found that the original corrupt-cache test added a naming
  violation before its recovery assertion, allowing snapshot invalidation to
  explain the fresh result. The revised test repeats against an unchanged
  project, asserts a successful fresh validation and a valid rewritten cache
  record, then separately checks the later naming violation.
- Context-health review: the known Trellis `cargo xtask pr` active-task routing
  discrepancy remains documented rather than waived; Q07 will retain its
  nonzero result if it recurs. The nested launcher package test is bounded and
  useful; no reusable-skill gap is evident.

## Iteration 20 — 2026-09-06 — Q07 launcher error contract

- Q07 started in isolated current-master worktree `goal/q07-error-contract` at `c9ad106`. A focused red test proved the primary launcher silently treated a present, non-executable companion as absent because spawn errors were discarded with `.ok()?`.
- The smallest repair now propagates a path-bearing OS error and runtime exit `1`; the exact focused test passed. The first `--exact` invocation selected zero tests because its nested test path was incomplete and is explicitly excluded from proof. Remaining Q07 work is the report-output failure contract, selected cache fallback audit, packaged-launcher coverage, and required integration/release gates.

## Iteration 35 — 2026-09-06 — R01 hosted watch-regression refinement

- PR #173's first hosted matrix disproved the initial external-config rescan filter: macOS coalesced unrelated external-config activity with Assura runtime-output paths, emitting a false full rescan in both explicit-config and requested-directory watch contracts. The failure is preserved in R01 evidence; no gate was waived and no merge occurred.
- A new focused mixed-event unit test was RED before the correction and is green after it. The refinement ignores only rescan events made entirely of external/config-sibling and Assura-runtime-output paths; excluded in-scope paths and pathless rescans remain observable safety fallbacks. Independent review added a direct pathless-rescan regression; all 16 `cli::watch` units and 14 watch integration tests pass locally.
- Context health: the only repeat was platform-specific watcher event coalescing, now captured by a compact unit contract beside the classifier. Existing watch evidence and test patterns are sufficient; no new skill is warranted. Next: independent review of the new SHA, then a fresh hosted matrix.

## Iteration 36 — 2026-09-06 — R01 requested-scope sequence correction

- The corrected R01 hosted matrix passed Linux but macOS exposed a remaining test-sequencing defect: a helper intentionally accepts a successful, pathless full-rescan event for an out-of-scope FSEvent, then the test incorrectly expected the subsequent in-scope mutation to retain sequence 2. The helper now returns the next expected sequence (2 when quiet, 3 after the accepted safety fallback); it does not weaken either event’s content contract.
- The focused requested-directory test, complete 14-test watch integration suite, 16 `cli::watch` units, formatting, diff, and structure gates pass locally. Next: independent review of this exact test-contract repair, then a fresh hosted matrix.

## Iteration 37 — 2026-09-06 — R01 external-config rescan integration

- PR #173 merged the independently reviewed external-config rescan repair as `b3002b1`; the exact commit was fetched and verified reachable from `origin/master`. Its full hosted matrix passed, including macOS/Windows/Linux suites, MSRV, performance, release/installer/adoption, documentation, security, coverage, and evidence gates.
- The correction keeps watcher safety explicit: unrelated external config and Assura runtime-output noise no longer produces false rescans, while pathless and excluded in-scope rescans remain observable fallbacks. No tag, release, deployment, or public action was taken. Next independent ready card: A02.

## Iteration 38 — 2026-09-06 — A02 explicit local-policy preflight

- A02 is active in isolated current-master worktree `goal/a02-local-patterns`. Focused VPS integration proof covers `init --recipe-file` with a spaced path, SHA-256 provenance in `.assura/onboarding/profile-selection.json`, successful `agent onboard --recipe-file`, and a conflicting local rule with path/existing/incoming diagnostics.
- A new red contract showed onboarding was materializing its baseline before detecting a local-policy conflict, changing project config despite the conflict. The implementation now preflights a local recipe against an existing config before baseline materialization; the four-test focused suite is green and the local Assura structure gate has zero violations. This is not card completion: invalid-result, idempotence, bundled fixture, documentation, broader-gate, and review evidence remain.
- Context health: local disk recovered to 1.7 GiB; VPS remains the compile/test authority and has ample capacity. The only repeated operational friction is explicit source synchronization between the isolated local worktree and the VPS clone; existing evidence commands remain sufficient and no new reusable skill is warranted. Next: add the remaining A02 merge and idempotence contracts.

## Iteration 39 — 2026-09-06 — A03 closure bookkeeping correction

- Post-merge verification of PR #179 (`929fbff`) found its evidence text correctly closed A03, but its backlog edit had matched Q02's repeated status fields instead of A03. Q02 also has no referenced evidence file, so its accidental `done` status was not supportable.
- Dedicated documentation-only PR #180 changes exactly those two records: A03 is `done` with its merged evidence, and Q02 is returned to unclaimed `pending`. JSON validation, the installed `assura check --format agent .`, and whitespace validation pass. No product behavior or CI scope changed.
- Context health: not exposed. This is the first exact-ID bookkeeping mismatch in the train; the immediate correction and card-ID-anchored patch are sufficient, so no reusable skill is justified. Next ready cards after PR #180 merges: A04 and A05, each dependent on A03.

## Iteration 40 — 2026-09-06 — A04 hook ownership and real-event proof

- A04 is active in isolated current-master worktree `goal/a04-verified-hook-lifecycle`. Focused RED/green tests establish exact generated-content ownership for Git hook install, force refresh, status, direct removal, and bulk removal; custom hooks, legacy-marker collisions, and custom sidecars are preserved.
- A disposable real Git repository at a path with spaces proved actual candidate
  hook events against a seeded violation: advisory `git push` exited 0 and opt-in
  `ASSURA_BLOCKING_PUSH=1 git push` exited 1. The feature-branch pre-commit
  warning was also observed without claiming merge protection. Ten public hook
  lifecycle tests, four private resolver tests, and thirteen host lifecycle
  tests passed; independent final review is clean.
- Context health: not exposed. Multiple reviewer findings all reduced to one reusable ownership rule and are now captured in the harness hook spec. The local `cargo xtask pr` runner did not yield a terminal result within the observation window, so it remains inconclusive; hosted performance/PR gates are mandatory before merge.

## Iteration 43 — 2026-09-06 — A03 target-state contract repair

- Post-merge verification exposed one verifier inconsistency rather than a
  product regression: the accepted A03 public guide used the evidence-first
  procedure, while `cargo xtask target-state` still required its three retired
  questionnaire markers. The isolated baseline failed only for those markers;
  no guide content was restored and no gate was disabled.
- Implementation `932306e` extracts the production guide contract for focused
  tests, requires the current procedure/action/path, and rejects the retired
  heading/action/path. Focused RED failed three tests for the stale behavior;
  GREEN and post-commit runs passed all three. The exact implementation passed
  `cargo xtask target-state`, `cargo xtask fast`, and the complete local
  `cargo xtask pr` gate, including Clippy, deterministic docs evidence, and the
  48-page website build.
- Independent review of `6d613f8..932306e` found no issues and marked spec and
  quality PASS. Context health: the omitted target-state update is now captured
  by positive, missing-current, retired-guidance, and contextual-exception
  controls; no new reusable skill is warranted. Next: commit this evidence,
  run the final exact-HEAD PR gate, then let the controller open the PR.
