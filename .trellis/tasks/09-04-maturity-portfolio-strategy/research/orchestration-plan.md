# Maturity train orchestration plan

The durable goal contract is
[`docs/goals/assura-maturity-execution-train.md`](../../../../docs/goals/assura-maturity-execution-train.md).
This compact plan is the task-specific operator route; it must not become a
second goal or a substitute for the ledger and selected card packet.

Status: active process contract, 2026-09-12. This document routes the
supported runtime goal; it does not promote a product card, grant authority,
or create screening, holdout, publication, release, deployment, or invitation
credit. The task ledger and each card packet remain the acceptance source of
truth.

The current evidence-backed corrections and the live next action are recorded
in [process-corrections-2026-09-11.md](process-corrections-2026-09-11.md).
Refresh its source pointer before use; it is a compact checkpoint, not a
replacement for the ledger or private A07 packet.

The detailed state machine, checkpoint schema, layered-disclosure boundary,
validation budget, VPS eligibility test and merge fence live in
[`execution-control-plane.md`](../../../../.agents/skills/assura-goal-execution/references/execution-control-plane.md).
Load that reference only for an execution, review, integration or handoff
decision; keep this plan as the task-level route.

## Latest observed integration route — PR #319 — `afa1637` (2026-09-12; refresh required)

- PR #319 merged the reviewed R01 owner-approved archive reconciliation from
  head `8d09ebd64bc764ce00337a4a957164ffc0a622d2` on base
  `3c01e65ab609e40b4cdb9d53d4337577899d5a29` as
  `afa1637ccceb93feff1bed5e652cf50516f68a9a` with tree
  `d286f78b299f5c080d9e4f2f452bf8615f01ceb1`. Exact-head Documentation
  Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian passed;
  merge-SHA Documentation `34680275467`, Security Audit `34680275427` and
  Rust CI `34680275538` passed. Scope-skipped product, Rust, performance and
  release rows remain non-proof.
- Nick's owner-level [PR #185 archive decision](https://github.com/rothnic/assura/pull/185#issuecomment-5591702709)
  accepts H01. R01 is now `not_needed` as an explicit scoped exclusion, not a
  product-fix claim; the failed run and missing causal callback fields remain
  preserved. The revision-pinned route must not infer that the historical
  regression was repaired.
- The active goal remains active. A07 remains ahead of empty pending rows with
  no live candidate; W02/F01 retain their external authority holds and W03
  retains publication authority. Refresh source, release/tag, PR/CI, topology
  and ledger before the next phase.

## Historical integration route — PR #318 — `3c01e65` (2026-09-12; superseded by PR #319)

- PR #318 merged the reviewed current-source continuation-route correction
  from head `78fdfb0ca3f61873b50078e077d5744345a85101` on base
  `a1d387f736d52e19ee0037bcd2e62146a21147d8` as
  `3c01e65ab609e40b4cdb9d53d4337577899d5a29` with tree
  `4a43a34a90f82a3b504b934c8ce90fa3e1c40e47`. Exact-head Documentation
  Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian passed;
  merge-SHA Documentation `34679613211`, Security Audit `34679613203` and
  Rust CI `34679613179` passed. Scope-skipped product, Rust, performance and
  release rows remain non-proof.
- At that observation this was the latest as-of record, not a live pointer.
  The reset found no `origin/release` head and no release tag at `3c01e65`; fetch
  `origin/master` and rerun release/tag, PR/CI, topology and the revision-
  pinned ledger before routing work. The ledger is
  `items=32; ready_pending=0; unfinished=5; held=3` (A07 active, W03
  verified, R01/W02/F01 held); A07 candidate/packet evidence remains
  candidate-base/no-credit and authority flags remain false.
- No live A07 candidate exists. Keep A07 ahead of empty pending rows without
  declaring the goal blocked: confirm explicit no-credit preparation authority
  before creating one clean candidate and running identity, sibling-free
  canary, holdout/manifest rebind and isolated protocol review in order. If
  authority is absent, continue the independently audited held-action routes
  in `progress-current.md`; preserve unknown/foreign/stale topology and
  unfavorable evidence.

## Historical route — `origin/master=20256c1` (2026-09-12 post-merge reconciliation; superseded by 2d512c1)

- PR #314 merged the reviewed e1c9 reconciliation from head
  `c458ef3f9843848d10fabb7ba2ce32b5409d0e73` on base
  `e1c9b78216b5736abd24fbd165ed7e1e383b1744` as
  `20256c132b208bdbce5637d693c4ff8c4b03b5e6` with tree
  `a90744735e7a0b157f41a48a0e78ca7665403efb`. Exact-head Documentation
  Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian passed;
  merge-SHA Documentation `34675964341`, Security Audit `34675964350` and
  Rust CI `34675964340` passed, including CI Scope `103505761574` and
  Evidence Gates `103505781689`. Scope-skipped product, Rust, performance and
  release rows remain non-proof.
- Ledger remains `items=32; ready_pending=0; unfinished=5; held=3` (A07
  active, W03 verified, R01/W02/F01 held). The clean owned
  `docs/a07-postmerge-e1c9` branch/worktree was verified with equal reviewed
  and merge trees, then removed; the a6ed candidate/packet remains
  candidate-base/no-credit and the e1c9 process slice is now historical.
- No screening, allocation, credit, acceptance, release, deployment,
  publication, invitation or protection authority changed. Preserve
  unknown/foreign/stale topology and unfavorable evidence; the optional VPS
  lane remains held.
- Next action: refresh release/tag, PR/CI, topology and ledger again. Keep A07
  ahead of empty pending rows and, if no-credit preparation remains
  authorized, build a fresh exact-toolchain candidate from `20256c1` in one
  clean owned worktree, then run identity, sibling-free canary, current
  holdout/manifest rebind and isolated protocol review in order. Public
  process updates still require independent review, applicable gates,
  current-master integration and merge-SHA observation.

## Historical route — `origin/master=e1c9b78` (2026-09-12 post-merge reconciliation; superseded by 20256c1)

- PR #313 merged the reviewed packet-coherence process slice from head
  `a134fc6815fc76730926d9f8ded57f16a8740e74` on base
  `a6ed20f532d93f830099e9095798a09515fead0b` as
  `e1c9b78216b5736abd24fbd165ed7e1e383b1744` with tree
  `9d57888eec65d892dcc2a65b829369dd90278637`. Exact-head Documentation
  Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian passed;
  merge-SHA Documentation `34675049056`, Security Audit `34675049038` and
  Rust CI `34675049055` passed, including CI Scope `103503298569` and
  Evidence Gates `103503318127`. Scope-skipped product, Rust, performance and
  release rows remain non-proof.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3` (A07 active, W03 verified, R01/W02/F01 held). The clean owned
  `goal/a07-current-a6ed` branch/worktree was verified ancestor-reachable and
  removed. Its candidate, canaries, six-holdout/two-condition packet,
  validator and isolated protocol `PASS_NO_CREDIT` are candidate-base/no-credit
  after the source advance; the reviewer artifact remains immutable.
- No screening, allocation, credit, acceptance, release, deployment,
  publication, invitation or protection authority changed. Preserve unknown
  root dirt, foreign dirty work, stale registrations, unrelated branches and
  unfavorable evidence. The optional VPS lane remains held by the unresolved
  alias, disk pressure, toolchain mismatch and unrelated jobs.
- Next action: refresh release/tag, PR/CI, topology and ledger again before
  selecting work. Keep A07 ahead of empty pending rows and, if no-credit
  preparation remains authorized, build a fresh exact-toolchain candidate from
  `e1c9b78` in one owned clean worktree, then run identity, sibling-free
  canary, current holdout/manifest rebind and isolated protocol review in
  order. Any public update still needs independent review, applicable gates,
  current-master integration and merge-SHA observation.

## Historical route — `origin/master=a6ed20f` (2026-09-12 candidate-bound disposition; superseded by e1c9b78)

- PR #312 merged the reviewed dcf2 process slice from head
  `f43d1a0cc6cd868a78d4f68dbc97a52a90476abb` as
  `a6ed20f532d93f830099e9095798a09515fead0b` with tree
  `bd399cd480dc7a31630740718dd8248a0c77bef5`. Applicable exact-head and
  merge-SHA Documentation, CI Scope, Evidence Gates, Security and GitGuardian
  checks passed; scope-skipped product/performance/release rows remain
  non-proof. Ledger: `items=32; ready_pending=0; unfinished=5; held=3`
  (A07 active, W03 verified, R01/W02/F01 held).
- `/root` owns a clean exact-toolchain candidate in
  `/private/tmp/assura-a07-current-a6ed` on `goal/a07-current-a6ed`. Build,
  command-help, login-shell identity, wrong-target and wrong-root controls
  pass/reject as intended; candidate/shim SHA is
  `1129d4498651fce2679264c5f2464931edd77e5e1bc0bcb3b02914ba1f42913c`.
- Fresh sibling-free A/B canaries and post-exit full evaluators both exit `0`
  with all seven dimensions passing and zero critical failures. The packet has
  six unique holdouts, two stable conditions and 30 reserved cells; persisted
  validation is valid/exit-0 and protocol `PASS_NO_CREDIT`. The isolated
  rereview resolved `A07-A6ED-VALIDATOR-001`, `A07-A6ED-REVIEW-TIME-002` and
  `A07-A6ED-CANARY-STATE-003`; coordinator pre/post hash finalization is
  recorded. This remains metadata preparation only.
- Next action: commit and independently review this goal/skill/evidence
  reconciliation, run applicable gates, integrate only reviewed current-master
  work, observe configured merge-SHA workflows, and refresh source/ledger/
  topology. If source advances, classify the packet candidate-base/no-credit
  and rebuild. Keep screening/allocation/credit/acceptance false; the VPS lane
  remains held by unresolved alias, disk pressure, toolchain mismatch and
  unrelated jobs.

## Historical route — `origin/master=3e67d5f` (2026-09-12 candidate-bound no-credit; superseded by `9c1b68a`)

- Reset evidence is current at source
  `3e67d5fe123c6ebcf3ac05617966919151e44868`, tree
  `1b3b2c130a294b1020f0c63d84df120434bebf42`; the ledger remains
  `items=32; ready_pending=0; unfinished=5; held=3` (A07 active, W03
  verified, R01/W02/F01 held). Preserve root/foreign/stale topology findings
  and unknown dirt outside ownership.
- `/root` owns `/private/tmp/assura-train-current-3e67`, an exact
  Rust/Cargo `1.94.1` release candidate with binary/shim
  `bbb244635770c44862db1becfab5ae46c107c904821baec303bcc52ae6bd05fa`.
  Login-shell identity and both negative controls pass/reject as intended.
  The corrected canary runner retains the earlier stale-login-shell mismatch
  as unfavorable evidence rather than hiding or overwriting it.
- Conditions A and B are fresh sibling-free source-only one-shot runs with
  initializer exit `0`; full seven-dimension post-exit evaluators pass with
  zero critical failures. The private packet has six unique holdouts, two
  conditions and 30 reserved cells; adapted metadata validation is `valid=true`.
  All cells remain reserved/no-credit and screening is unauthorized.
- Independent protocol metadata review is `PASS_NO_CREDIT` for the corrected
  packet. Update a concise public evidence slice and send it through
  independent review plus applicable gates. Any merge advances source, so
  refresh and rebuild the candidate before screening. A protocol pass is not
  A07 acceptance; any new finding still needs a scoped correction/rereview.
- The VPS efficiency lane is held by the measured unresolved alias, disk
  pressure, unrelated jobs and toolchain mismatch. Do not launch remote heavy
  work or replace hosted/platform proof.

## Historical route — `origin/master=eef7e1a` (2026-09-11 post-merge reconciliation; superseded by `3e67d5f`)

- PR #307 merged reviewed process corrections from head
  `9723a6730ac4cff2c8567035a8cd4f1fe21e25f4` on base `66e0b7e` as
  `eef7e1a7400c84ec35be33a58914ad07da7fa376`. Exact-head Documentation,
  CI Scope, Security Scope, Evidence Gates and GitGuardian passed; applicable
  merge-SHA Documentation `34660366526`, Security Audit `34660366986` and
  Rust CI `34660366735` passed. Scope-skipped product/Rust/performance/release
  rows remain non-proof.
- Ledger remains `items=32; ready_pending=0; unfinished=5; held=3`: A07
  active, W03 verified, R01/W02/F01 held. The `66e0b7e` process checkpoint and
  all earlier candidate/canary/protocol packets are candidate-base/no-credit
  after this source advance. The owned branch/worktree is being reconciled;
  preserved external/unknown dirt remains outside ownership. No product or
  authority state changed.
- Resume route: refresh source/release/tag/PR/CI/topology and the ledger;
  inspect active/implemented/verified/held before pending; choose one named
  owner and one actionable recovery/preparation phase; and keep it live
  through validation, review, integration, post-merge workflow observation and
  cleanup. For A07, rebuild at refreshed master, then identity/no-credit
  canary, holdout/manifest rebind, isolated protocol `PASS`, and only then a
  separately authorized screening decision. Empty pending is not completion.

## Historical route — `origin/master=66e0b7e` (2026-09-11 checkpoint; superseded by eef7e1a)

PR #306 merged the reviewed process reconciliation at `66e0b7e`; applicable
hosted gates passed and the `dc031527` candidate/packet was classified
candidate-base/no-credit. No product or authority state changed.

## Historical route — `origin/master=dc031527` (2026-09-11 checkpoint; superseded by 66e0b7e)

PR #305 merged the reviewed process reconciliation from `284e781` as
`dc031527afd400be52dfd8fe9cfdabc7a6caa685`. Exact-head applicable
Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks
passed. Merge-SHA Documentation `34653158838`, Rust CI `34653159004` (CI Scope
`103439684733`, Evidence Gates `103439731188`) and Security Audit
`34653158857` completed successfully for applicable scope; product, Rust,
performance and release rows were scope-skipped and remain non-proof.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3`: A07 active, W03 verified and R01/W02/F01 held. The `284e781` process
checkpoint and d228472 candidate are historical or candidate-base/no-credit;
no product, threshold, screening, allocation, acceptance, release, deployment,
publication or invitation authority changed.

`/root` owns a clean exact-toolchain candidate checkout at
`/private/tmp/assura-a07-current-reconcile-dc031`. Its source/tree/binary/shim
identity is bound to `dc031527`/`b68b66cc47f7e4081afd2dc21c1387de34b30c32`/
`3cdaf976d7da61043a5867cb17ffd789b92b484e26f1e7d3a7989bd25ce816cf` with
Rust/Cargo `1.94.1`; identity controls pass, and wrong-target/wrong-root
controls reject. Two sibling-free source-only canaries and full seven-
dimension evaluators pass with zero critical failures, but remain no-credit.

The private A07 packet at `/private/tmp/assura-a07-private-dc031` now has
corrected current receipt, construction and second-readonly aliases, six
unique holdouts and 30 reserved cells. The metadata-only validator reports
`valid=true` with zero errors. Independent protocol review returned
`PASS_NO_CREDIT` after correcting the stale contract digest and construction
hash; all screening, allocation and credit flags remain false. The observed
stale-alias correction is retained as process evidence: do not rebind a
current packet by blind string replacement.

Next action: reconcile the protocol-pass/no-credit evidence in a reviewed
current-base process slice. Refresh source, release/tag, PR/CI, topology and
the ledger after every merge; if source advances, classify this packet
candidate-base/no-credit and rebuild. Continue R01/W02/F01 only within their
named holds and never stop at an empty pending queue.

## Historical route — `origin/master=284e781` (superseded by dc031527)

The latest reset fetched
`origin/master=284e78156370d49d8315f04391fcc74eaba3acb2` with the merge tree;
fetch again before the next phase. PR #304 merged reviewed head
`17fa9197ed79083be4b8ba0714851aeaae9bdbf3` from base `d228472` after
independent `PASS`. Its applicable exact-head checks passed, and merge-SHA
Documentation `34651068660`, Rust CI `34651068675` and Security Audit
`34651068699` completed successfully for applicable scope. Rust CI's CI Scope
`103433100783` and Evidence Gates `103433136825` passed; product/Rust/
performance/release and other scope-skipped rows remain non-proof.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3`: A07 active, W03 verified and R01/W02/F01 held. The d228472
candidate and canary/evaluator results are now candidate-base/no-credit after
this process merge; no product, screening, allocation, acceptance, release,
deployment, publication or invitation authority changed. The merged d228
process checkout is clean and removed; root/foreign/stale topology exceptions
remain preserved. The pre-cleanup report was
`base=origin/master, worktrees=53, dirty=3, prunable=3, unreadable=1,
goal_branches=13, unmerged_goal=9`.

The measured VPS lane remains held: `vps-dev` is not a configured alias;
configured `vps` has 16 CPUs, 61 GiB RAM, 20 GiB free on a 339 GiB root (95%
used), and Rust/Cargo `1.95.0-nightly`. Do not launch a heavy remote job until
an exact-toolchain, free-disk and owned-job probe passes; remote Linux output
never replaces platform or hosted proof.

Next action: refresh source/release/tag/PR/CI/topology and the ledger before
the next phase. If A07 preparation is separately authorized, build a fresh
exact-toolchain candidate from `284e781`, run identity and sibling-free
no-credit canary gates, rebind the six holdouts and exactly-two-condition
manifest, and obtain isolated protocol `PASS` before screening. Otherwise
continue an independently authorized held R01/W02/F01 recovery slice. Observe
the post-merge workflow fence for every future merge; an empty pending queue
is not a stop condition.

## Historical route — `origin/master=d228472` (superseded by `284e781`)

PR #303 merged the reviewed process reconciliation at `d228472`. Its
applicable exact-head and merge-SHA checks passed. The fresh d228472 candidate
passed identity, two sibling-free canaries and full evaluator checks with no
credit, while ambient skill metadata and the initial structure-placement
failure were retained. The source advance to `284e781` makes that candidate
candidate-base/no-credit; no packet, screening, allocation or authority state
carried forward.

## Historical route — `origin/master=24a1966` (superseded by `453a32a`)

The latest reset fetched `origin/master=24a1966a55eb3ec184190c565f2715ee5daa7801`
with tree `57cfe8e7f437fc15f4fc8fdeff807c0eb9214c00`; fetch again before the
next phase. PR #301 merged reviewed head `428c26aee89b449e5c57ed1589d6a9b8d083c3bc`
after independent `PASS`. Its applicable exact-head checks passed, and
merge-SHA Documentation `34642991331`, Rust CI `34642991392` and Security Audit
`34642991344` completed successfully for their applicable scope. Product/Rust/
performance/release rows were explicitly skipped and remain non-proof.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3`: A07 active, W03 verified and R01/W02/F01 held. The 0dff804 candidate
identity freeze is candidate-base/no-credit after this source advance; do not
reuse its binary, packet or canary. The merged process branch was clean and
removed; root/foreign/stale topology exceptions remain preserved.

Next action: `/root` owns a fresh exact-toolchain candidate freeze at 24a1966
in a clean checkout. Then run the sibling-free source-only no-credit canary,
rebind the six holdouts and two-condition manifest, and obtain isolated
protocol `PASS` before any separately authorized screening. Continue R01/W02/
F01 held routes independently and observe the post-merge workflow fence for
every future merge; an empty pending queue is not a stop condition.

## Historical route — `origin/master=0dff804` (superseded by `24a1966`)

The latest reset fetched `origin/master=0dff804421c7563b08773eb75d9327fd0194db56`
with tree `4dcb61add201e72e6b61f3bd8392f508d008160c`; fetch again before the
next phase. PR #300's reviewed head `f37c20e21ec94116c18dd591638fabf86163f947`
passed its applicable Documentation, CI Scope, Security Scope, Evidence Gates
and GitGuardian checks, and merge-SHA Rust CI `34638264948`, Documentation
`34638264944` and Security Audit `34638264978` completed for their applicable
scope. Product/Rust/performance/release rows were explicitly skipped and are
not product proof.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3`: A07 active, W03 verified and R01/W02/F01 held. `/root` owns the
current A07 no-credit identity preparation in
`/private/tmp/assura-a07-current-0dff`; exact Rust/Cargo `1.94.1`, source/tree,
binary/shim and login-shell identity agree, wrong-target/wrong-root controls
reject, and independent scoped review returned `PASS` after freeze-record
byte identity was corrected. Canary, holdout/manifest rebind and protocol
review are not run; screening, allocation and credit remain false.

Next action: run the fresh sibling-free source-only no-credit canary, then
rebind the six holdouts and two-condition manifest and obtain isolated protocol
`PASS` before any separately authorized screening. Preserve the 4560c710
packet as candidate-base/no-credit and keep R01, W02, W03 publication and F01
authority boundaries separate. Close only the exact clean owned preparation
branch/worktree after current-source review and post-merge proof.

## Historical post-merge route — `origin/master=651ef31` (superseded by `0dff804`)

The latest reset fetched `origin/master=651ef31ea4d609a83c59af2a458f4313db91c9c3`
after PR #299 merged the reviewed current-checkpoint reconciliation. This
pointer is dated evidence, not a permanent baseline: fetch again before the
next phase. PR #299's reviewed head was
`c6a47ae4e486d4d226a747a3f49653c235f251d1` and its applicable Documentation,
CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed. The
reviewed head tree equals the merge tree. The configured push-triggered Rust
CI (`34635475363`), Documentation (`34635475400`) and Security Audit
(`34635475378`) workflows completed at the merge SHA; product/Rust/
performance/release jobs were scope-skipped and are not product proof. The
earlier push-triggered macOS watch-SIGINT failure remains retained unfavorable
evidence and was not converted into a green result.

At this source the revision-pinned ledger is still
`items=32; ready_pending=0; unfinished=5; held=3`: A07 active, W03 verified,
and R01/W02/F01 held. The 4560c710 candidate and protocol packet are
candidate-base/no-credit metadata after this source advance. No screening,
allocation, credit, release, deployment, publication or invitation authority
exists.

The merge fence is now two-stage: exact-head pull-request checks prove the
reviewed candidate before merge, and configured push-triggered workflows at
the merge SHA must be observed during reconciliation. A failed, cancelled,
unavailable, zero-test, scope-uncertain or absent post-merge job is an
unresolved recovery route, not a green result or a reason to close the owned
branch. PR #299's successful merge-SHA runs (`34635475363`, `34635475400`,
`34635475378`) are evidence that this process fence itself passed; they do not
repair R01.

The coordinator owns the process route: refresh source/release/tag/PR/CI,
topology and ledger; inspect active/implemented/verified candidates; then
choose one explicitly owned recovery or preparation action. If a future A07
phase is separately authorized, rebuild the candidate at the refreshed source,
run the no-credit canary, rebind the packet and obtain isolated protocol PASS
before screening. Otherwise continue the hosted watch-SIGINT diagnostic or
another authorized recovery slice. Do not stop at the empty pending queue.

## Historical post-merge route — `origin/master=329bce0` (superseded by `651ef31`)

PR #298 merged the reviewed current-checkpoint reconciliation at `329bce0`.
Its applicable exact-head and merge-SHA checks passed; product/Rust/
performance/release rows were scope-skipped and were not product proof. This
checkpoint is retained as historical process evidence; PR #299 is the current
source reconciliation. The candidate-base/no-credit A07 packet and retained
R01 diagnostic were unchanged.

## Historical post-merge route — `origin/master=6d57b86` (superseded by `329bce0`)

PR #297 merged the reviewed current-checkpoint reconciliation at `6d57b86`.
Its applicable exact-head and merge-SHA checks passed; product/Rust/
performance/release rows were scope-skipped and were not product proof. This
checkpoint is retained as historical process evidence; PR #299 is the current
source reconciliation. The candidate-base/no-credit A07 packet and retained
R01 diagnostic were unchanged.

## Historical post-merge route — `origin/master=83c382a` (superseded by `6d57b86`)

PR #296 merged the reviewed post-merge reconciliation at `83c382a`. Its
applicable exact-head and merge-SHA checks passed; product/Rust/performance/
release rows were scope-skipped and were not product proof. This checkpoint is
retained as historical process evidence; PR #297 is the current source
reconciliation. The candidate-base/no-credit A07 packet and retained R01
diagnostic were unchanged.

## Historical post-merge route — `origin/master=9047a3d` (superseded by `83c382a`)

PR #295 merged the reviewed workflow-fence and measured-capacity correction
at `9047a3d`. Its exact-head applicable checks and configured merge-SHA
Documentation, Security Audit and Rust CI workflows passed; product/Rust/
performance/release rows were scope-skipped and were not product proof. This
checkpoint is retained as historical process evidence; PR #296 is the current
source reconciliation. The retained R01 macOS watch-SIGINT failure and the
candidate-base/no-credit A07 state were unchanged.

## Historical candidate route — `origin/master=4560c710`

PR #292 merged the reviewed recovery slice at
`4560c710967d59993b9ea4f9b86613d446443f79`. Its exact-head applicable checks
passed, while the distinct push-triggered Rust CI run `34615572565` exposed a
macOS watch SIGINT failure and cancelled the Ubuntu/Windows matrix siblings.
The focused local test passed once; the hosted failure remains an explicit
diagnostic and cannot be hidden, downgraded or blindly retried.

The ledger is `items=32; ready_pending=0; unfinished=5; held=3` (A07 active,
W03 verified, R01/W02/F01 held), so A07 remains the active lane despite the
empty pending queue. The current owned candidate is frozen with exact
Rust/Cargo `1.94.1` identity. Its two fresh source-only canaries pass the full
seven-dimension evaluator with zero critical failures, and the private packet
now has current six-handle/second-readonly provenance, fixture freshness,
shared invariants, evaluation bindings, two conditions, and 30 reserved cells.
The independent reviewer found two metadata defects (receipt-ID aliases and a
stale construction hash); both are repaired, cross-artifact assertions pass,
and the scoped rereview returned `PASS` for the metadata scope. Everything
remains no-credit.

That candidate's next action was a source refresh; it is now candidate-base
metadata. Preserve its protocol hashes, unresolved hosted failure and all
unfavorable evidence, but do not use it as current candidate proof.

## Outcome invariant

The train advances one explicitly owned slice at a time from its current
source through investigate, implement, verify, independent review, current-
base integration, post-merge reconciliation, and clean closure. A passing
slice is not a passing parent card. A failed, skipped, zero-test, unavailable,
or scope-uncertain required gate remains unresolved; thresholds, negative rows,
and unfavorable evidence are preserved.

The coordinator never ends at an empty pending queue or an observation
timeout. The next checkpoint names the owner/session, source and candidate
SHA, checkout/branch, phase, proof and exact next observation. A card-level
hold records its contract, evidence, owner, smallest resolution and unrelated
work that can continue. Whole-goal completion requires accepted cards, final
current-base proof, and merged or verified-archived owned topology.

## Layered context route

Load the smallest layer that answers the next decision. This keeps AGENTS.md a
universal router instead of a second manual and prevents a compacted session
from routing from an old checkout or private evaluator detail.

| Decision | Read | Output |
| --- | --- | --- |
| Reset/ownership | `AGENTS.md`, workflow-gate output, cwd/branch/remote/HEAD/status, topology report | clean owned checkout or preserved dirt classification |
| Source/queue | refreshed `origin/master`, `audit-ledger.sh`, `backlog.json`, recovery plan | revision-pinned active/implemented/verified/merge-ready route |
| Card execution | PRD, selected packet and `evidence/<ID>.md` only | contract, files, dependencies, focused gate and owner |
| Special procedure | phase-linked reference such as validation routing, runner isolation or screening manifest | bounded commands and privacy/authority limits |
| Review/integration | frozen diff, public evidence, review brief and current-base checks | finding disposition, final-head proof and closure action |

At goal start, after compaction, and before handoff run
`scripts/audit-context-routing.py` from the same clean checkout. A failed
routing audit is a process-repair slice; it is not product acceptance and does
not justify copying detail into AGENTS.md. Keep private conditions, mappings,
fixtures, raw evaluator results and child transcripts outside prompts and
public evidence.

## Execution loop

1. **Reset.** Fetch `origin/master`; record its full SHA and time. Run the
   workflow gate, context-routing audit, revision-pinned ledger and topology
   report. Preserve unknown/user-owned dirt and inspect actual worker/session
   liveness; an owner string alone is not liveness.
2. **Route.** Inspect active, implemented, verified and merge-ready cards before
   pending rows. Choose exactly one real action with one owner: resume a live
   handle, diagnose a failed gate, implement/verify an authorized slice, obtain
   a named authority decision while preparing independent work, or integrate a
   reviewed current-base candidate.
3. **Prove.** Run cheap structure/scope/target checks first, then the focused
   red/green contract and the smallest applicable tier. Keep one heavy Cargo
   sequence per checkout; `cargo xtask pr` already includes `fast`. Record cwd,
   source, toolchain/binary identity, host, elapsed/wait time, exit and test/job
   counts. Do not rerun an unchanged failing method.
4. **Review.** Freeze the committed diff and public evidence. Dispatch an
   independent reviewer with the Assura brief. Vague feedback becomes a
   concrete contract, location, failure scenario and smallest verification;
   accepted findings are fixed as a delta, affected gates rerun, and the delta
   independently rereviewed. Continue only independent authorized preparation
   while review or CI handles remain live.
5. **Integrate.** Merge only reviewed current-base source with all applicable
   local/hosted/performance checks passing, formal review resolved and existing
   authority sufficient. Scope-skipped jobs are recorded as non-applicable.
   Review trigger/deployment coupling read-only; a merge permission never
   grants deploy, release, publication or invitation authority.
6. **Reconcile.** Fetch again, prove merged reachability/tree equality, rerun
   the ledger and topology audits, update the compact checkpoint, and remove
   only the exact clean owned branch/worktree after its closure proof. Keep
   historical snapshots labeled as-of rather than creating SHA-only churn.

## Efficient validation placement

- Local: workflow/context/structure/target-state checks before long work;
  install pinned website dependencies once per isolated checkout; use focused
  tests and the applicable `cargo xtask` tier.
- VPS: use the documented `assura-local-build` bundle procedure only after a
  fresh SSH, disk, memory, toolchain and owned-job probe. Start one bounded
  heavy job, retain its session/log/exit, and compare only immutable
  candidate/baseline binaries. Remote Linux supplements, never replaces,
  macOS/Windows/browser/host-permission or hosted proof.
- Hosted: push one prepared candidate, wait on the exact current-head checks,
  preserve failed performance rows and cancelled/unavailable required jobs, and
  never use retries, cache changes, protection changes or a self-hosted runner
  to manufacture a green result. Propose CI efficiency changes only after
  three measured comparable runs show queue/execution, retry and coverage
  effects.

## Historical post-merge route — `origin/master=40f1155c` (superseded by `651ef31`)

The latest reset fetched `origin/master=40f1155c3d26dc40141d2464d7e2f027f7bb9770`
after PR #289 merged the reviewed durable goal, layered-routing skill metadata
and current-route corrections. Its applicable Documentation Scope, CI Scope,
Security Scope, Evidence Gates and GitGuardian checks passed; product/Rust/
performance/release jobs were scope-skipped and are not acceptance proof. The
latest tag must be refreshed separately; no release branch exists.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held. The ac3, ca816 and 851a6 candidate,
canaries, identity controls, six-handle/two-condition packet, 30 reserved cells
and isolated protocol `PASS` records are candidate-base/no-credit metadata after this
source advance. No candidate or screening allocation is current.

Refresh source, release/tag, PR/CI, topology and the ledger, then build and
freeze a fresh explicit-workdir candidate at the fetched source, run the
bounded no-credit canary, rebind the packet and obtain isolated protocol `PASS`
before seeking separately authorized screening. Do not reuse the 851a6 packet;
if source advances or identity mismatches, retain it as candidate-base metadata
and repeat the complete sequence. Preserve the root unknown path, foreign
dirty worktree, stale registrations, historical failures and goal branches.

## Historical candidate route — ac3eb13 (superseded by `cd629d4`, then `ca81689`)

PR #285 reconciled the reviewed c4f checkpoint at `ac3eb13`; its applicable
checks passed and the ac3 candidate/packet protocol rereview returned `PASS`.
Those canaries, identity controls and packet are historical no-credit metadata
after PR #286 advanced master. No screening allocation or acceptance credit
exists; retain the unfavorable wrapper evidence and rerun the complete current
candidate sequence before allocation.

## Historical post-merge route — c4f57d7 (superseded by `ac3eb13`)

The latest reset fetched `origin/master=c4f57d701d2ec37301f4677eae27b7275f89d2fb`
after PR #284 merged the reviewed process/evidence route. Its applicable
Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian
checks passed; product/Rust/performance/release jobs were scope-skipped and are
not acceptance proof. The latest tag is `v0.3.0-447-gc4f57d7`; no release branch
exists.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held. The 30b candidate, canaries and
protocol `PASS` are historical no-credit metadata after this source advance;
no current candidate or screening allocation exists. `/root` owns the next
explicit-workdir c4f candidate freeze, bounded no-credit canary, packet rebind
and isolated protocol review. Preserve the root unknown path and foreign,
stale and historical topology exceptions.

## Historical candidate route — 30b4c663 (superseded by `c4f57d7`)

The latest reset fetched `origin/master=30b4c663b28bf1d15f12c99b6263411d3bfc8d45`
after PR #283 reconciled the reviewed process route. Its applicable
Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian
checks passed; product/Rust/performance/release jobs were scope-skipped and are
not acceptance proof. The latest tag remains `v0.3.0`; no release branch exists.

The revision-pinned ledger is `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held. The explicit-workdir candidate was
built with Rust/Cargo `1.94.1`; two fresh source-only canaries pass the full
seven-dimension evaluator and its negative policy control. These are no-credit
preparation. The current six-handle/two-condition packet is rebound and its
isolated protocol rereview returned `PASS`; no screening cells are allocated
and authority is false. `/root` must refresh source/ledger, prove the packet
still matches, and seek separately authorized screening. Preserve the
root unknown path and foreign, stale and historical topology exceptions.

## Historical post-merge route — 55a38ff

The latest reset fetched `origin/master=55a38ff68e39aae9ede78bb51c01a9c2e1392de9`.
PR #282 merged the reviewed process-correction slice from `af5240d`; its
Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian
checks passed. Product/Rust/performance/release jobs were scope-skipped and are
not acceptance proof. The latest tag is `v0.3.0` (describe:
`v0.3.0-445-g55a38ff`), and no release branch exists.

The revision-pinned ledger is `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held. Open PRs #194, #187, #142 and #195
remain outside this route; #194 has a failed Performance Report, #187 retains
unverified Workers-build/deployment coupling, #142 has macOS/Alpine failures
and a cancelled Windows job, and #195 is an unmerged R03 collector change.

The `f4368883` candidate-bound canary, six-handle/two-condition packet and
isolated protocol `PASS` are historical no-credit metadata after PR #282
advanced master. No candidate packet is current at `55a`; no screening cells
are allocated. `/root` owns the next preparation phase: build and freeze a
fresh explicit-workdir candidate at `55a`, prove identity, run the bounded
no-credit canary, then rebuild/review packet metadata before any separately
authorized screening request. Preserve the root unknown path and foreign,
stale and historical topology exceptions.

## Historical candidate-bound route — f4368883 (superseded by `55a38ff`)

The current source is `origin/master=f4368883ac776cd5a9cb337b68c80165fc228a46`.
The process-only PR #281 merge is current; its applicable documentation,
scope, security, evidence and GitGuardian checks passed, while scope-skipped
product/Rust/performance/release jobs remain non-applicable. The revision-
pinned ledger is unchanged at 32 items, zero ready-pending, five unfinished and
three held: A07 active, W03 verified, R01/W02/F01 held.

The clean candidate build used the explicit current worktree and exact
Rust/Cargo `1.94.1` toolchain. Its release binary and fixed regular shim are
bound privately to the source/tree SHA and version. Two fresh source-only
one-shot Codex children completed the initializer; the full evaluator passed
all seven declared dimensions and its negative policy control rejected as
expected for both private condition values. The child streams also revealed
ambient user-level skill metadata despite `--ignore-user-config`; no evaluator
contract, mapping, hidden oracle or foreign worktree was exposed. This is a
recorded isolation limitation and all results remain no-credit canary evidence.

The current private r2 rebind records the candidate identity, two
supplied-input receipts, six immutable holdout references, an explicit
excluded draft and a materialized reserved 30-cell matrix. Its isolated
protocol metadata review returned `PASS`; that only makes the packet
internally executable and cannot grant screening, acceptance, release,
deployment, publication, invitation or protection authority. Refresh
source/ledger and prove packet identity again before seeking the separately
authorized screening gate. Do not allocate a screening cell from this
no-credit checkpoint.

## Historical post-merge route — 961dced

The latest reset fetched `origin/master=961dced162c8fe67ad87d64e0be750249edd5681`
after PR #280 merged the reviewed process-only route from `6a12ffa`. Its
applicable Documentation, CI Scope, Security Scope, Evidence Gates and
GitGuardian checks passed; product/Rust/performance/release jobs were explicitly
scope-skipped and are not acceptance proof. Rust CI run `34564937464` completed
successfully for its applicable jobs.

The revision-pinned ledger remains 32 items, `ready_pending=0`, five unfinished
and three held: A07 active, W03 verified, R01/W02/F01 held. The 5b03c7f
candidate freeze, two full-contract canaries, six-handle/two-condition rebind
and isolated protocol `PASS` are historical no-credit metadata after this
source advance. No screening, allocation or acceptance credit exists.

The current owner is `/root` in `/private/tmp/assura-a07-postmerge-961dced`;
the corrected candidate build is separately owned in
`/private/tmp/assura-a07-current-961dced`. A first build accidentally used the
dirty root and is retained as failed no-credit provenance; the explicit-workdir
rebuild produced the current candidate, and the fresh identity, receipt and
full-contract canary artifacts agree on source/tree/binary/shim/toolchain. The
six-handle/two-condition packet is rebound; isolated review found two metadata
defects, both were corrected, and scoped rereview returned `PASS`. Refresh
source/ledger again and prove packet identity before separately authorized
screening. Preserve all R01/W02/W03/F01 and external-authority holds.

## Historical candidate-bound route — 5b03c7f (protocol PASS; no-credit)

The latest reset fetched `origin/master=5b03c7f41df1fe8ecf9ae5168eaa11f550b8721f`
and the revision-pinned ledger remains 32 items, `ready_pending=0`, five
unfinished and three held: A07 active, W03 verified, R01/W02/F01 held. The
root's unknown user-owned dirty path and foreign/stale topology exceptions were
preserved; all candidate work is isolated in
`/private/tmp/assura-a07-current-5b03c7f` and the documentation owner branch
`docs/a07-current-5b03c7f`.

The current candidate was built once with the exact Rust/Cargo `1.94.1`
toolchain. Its private freeze and login-shell identity control match the source
SHA, Git tree SHA, absolute binary, version and shim. Two fresh source-only
fixtures under sibling-free parents completed the composed Codex initialization
route for the two private values of one product-input flag. Both full evaluator
runs passed all seven dimensions with zero critical failures, the expected
negative policy probe and a collected native test. A wrong-path control returned
exit `97`; an unsupported evaluator-dimension attempt was retained as failed
no-credit evidence and corrected by checking the evaluator's declared set.

The six-handle holdout construction, current binding and second-readonly
confirmation are rebound to this candidate with immutable creation records,
exact toolchain comparison and explicit raw-hook exclusion. The manifest has
exactly two conditions, one-variable difference, a private mapping and 30
reserved cells; all allocation and credit flags are false. Independent
protocol review returned `PASS` with no mandatory findings; its metadata-only
scope and residual limit (no independent public-contract byte rehash) are
recorded in the private review artifact. The coordinator's exact next action
is to refresh source/release/tag/PR/CI/topology and the revision-pinned ledger
again, then prove the refreshed revision still matches this packet. On
mismatch, record contract/location/failure/smallest verification, freeze a new
candidate, rerun the no-credit canary, rebind the holdout/manifest and obtain an
isolated protocol `PASS` before separately authorized screening. Do not
allocate cells or claim A07 acceptance from this preparation.

## Historical post-merge route — c5b2822

The refreshed source is `origin/master=c5b2822a664e3f04924cd0096fc000a5d132777b`,
the merge of PR #278's reviewed R01 retained-artifact inventory. Its exact-head
Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian
checks passed. Product, Rust, performance and release jobs were scope-skipped
for this documentation/evidence-only slice and are not acceptance proof. The
post-merge current-master check and tree-diff evidence passed; the retained
artifact set still lacks the raw watch/callback trace needed to close R01.

The revision-pinned ledger at this source remains 32 items with
`ready_pending=0`, five unfinished and three held: A07 active, W03 verified,
and R01/W02/F01 held. The 71adc2e candidate freeze, canaries, six-handle
binding, manifest and protocol `PASS` are historical no-credit metadata after
this source advance. Do not allocate screening cells or claim a card from
those records.

The next owned A07 action is, in order: refresh source, release/tag, PR/CI and
topology; rerun the revision-pinned ledger; freeze a current candidate and
identity canary; rebind all six holdouts and both condition receipts; obtain an
isolated protocol `PASS`; then seek the separately authorized screening gate.
R01 remains held for the raw trace or a maintainer native-readiness decision;
W02, W03 publication, F01 outreach, release and deployment retain their named
authority boundaries. Keep this process goal active while that action exists.

## Historical current train route — 71adc2e (superseded by c5b2822)

At the prior refresh, `origin/master=71adc2e695f4696e6a1fef7d5075aa836f4102b1`
was the source pointer. It is now historical after c5b2822. The owned candidate-bound build used the
pinned Rust/Cargo `1.94.1` toolchain; login-shell identity and two corrected
source-only canaries passed the full evaluator with the expected negative
policy probe. The earlier omitted-contract attempts remain explicit
unfavorable no-credit evidence. The ledger remains 32 items, no ready-pending
card, five unfinished cards and three narrow holds. A02 is complete and its
plain-init handoff incident is historical.

A07 is the active lane. The current canary, immutable holdout/manifest rebind
and isolated metadata protocol review are preparation only and grant no
allocation or acceptance credit; the protocol review returned `PASS`. The
exact next action is to seek the separately authorized screening preparation,
after refreshing source and ledger again. Do not reuse an older packet or
allocate or credit cells from this process checkpoint.

## Historical pre-71adc2 train route (superseded by current `71adc2e`) — `af005a7`

The prior post-merge refresh pointed at `af005a7dc5c367bc9388e08203fbf351bc0c88f2`
after PR #275, a process-only documentation slice. Its merged tree, owned
closure and applicable hosted checks were verified. At that time no af005a7
candidate had been frozen; the 129a249 candidate, canaries, bindings and
protocol `PASS` records were historical no-credit metadata. The preserved
next-action record was to refresh and re-freeze the candidate, which is now
superseded by the current canary and rebind route above.

R01 still needs the raw callback trace or a specific maintainer native-readiness
decision. W02 needs explicit Cloudflare approval before any current push that
could trigger builds. W03's technical change is verified but publication remains
separately authorized. F01's pilot kit is prepared, while participant selection
and invitations remain Nick-authorized. These holds do not end independent
process, review, topology, or authorized integration work.

## Checkpoint record

Before yielding, changing cards, dispatching review, or handing off, append a
compact record to the selected evidence:

```text
source/base: origin/master=<full SHA>; fetched_at=<time>
card/slice: <ID and observable outcome>
owner/session: <owner and live handle or none>
phase: investigate | implement | verify | review | integrate | reconcile
checkout: <absolute worktree>; branch=<branch>; candidate=<SHA>
proof: <commands, host/toolchain/binary, exits, elapsed, logs>
review: <review SHA, finding IDs and dispositions>
next: <exact command/observation and trigger>
held: <contract/evidence/owner/smallest resolution/independent work>
closure: active | merge-ready | merged | archived
```

## Process-slice evidence

This plan and its routing audit were validated from a clean detached checkout
at `origin/master=f921cae79d04e98ce729eecc64c79c68e4591fb9` before the candidate
was committed. The positive controls were:

- `python3 .agents/skills/assura-goal-execution/scripts/audit-context-routing.py . .trellis/tasks/09-04-maturity-portfolio-strategy` — exit 0; 42 routing/link/reference checks passed.
- `cargo run --quiet -- check --format json .` — exit 0; `success=true`, 1,827 files, 396 directories, and only existing low-severity line-length advisories.
- `cargo xtask evidence` — exit 0; review-evidence and CI-scope policy checks passed.
- `pnpm --dir website install --frozen-lockfile` followed by `cargo xtask docs` — exits 0; 48 website pages built.
- `git diff --check` and `python3 -m py_compile` for the audit script — exit 0.

The negative control ran the same audit against the pre-correction dirty
checkout. It returned exit 1 with 11 findings: a 295-line AGENTS router, an
overlong skill description, no `.trellis/tasks/` scope, and all seven layered
references absent. This demonstrates the audit catches the exact stale-context
failure without treating the old checkout's task state as current evidence.

The audit's missing-artifact negative control removed
`research/executor-prompt.md` in a disposable detached worktree and returned
exit 1 with `TASK_FILE executor-prompt.md FAIL`; the file was restored and the
worktree removed cleanly. The four task routing artifacts are therefore
required inputs, not optional observations.

The independent review of candidate `e479ac6` raised `CTX-001` because those
four files were initially optional. The finding was accepted and fixed in
`8803376`; the scoped rereview of `5dc8bfa` returned `PASS` with no remaining
concrete findings. The audit remains intentionally structural: it does not
prove task-content freshness, external references, or private evaluator
semantics, so those checks stay in the goal and card contracts.

The independent impasse review also found that the train's `task.json` primary
`branch` still pointed at the historical A05 checkout. The correction clears
that field to `null` and declares the active process branch only in
`meta.execution_branches`; `cargo xtask target-state` then passes without
inventing a single permanent train branch. This keeps session attribution from
silently selecting a completed card while preserving explicit card-branch
provenance.
