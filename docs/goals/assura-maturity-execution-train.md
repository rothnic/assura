---
id: goal-assura-maturity-execution-train
type: goal
title: Assura maturity execution train
status: active
created: 2026-09-11
owners:
  - /root
related:
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/task.json
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/backlog.json
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/recovery-plan.md
  - ../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/orchestration-plan.md
  - ../../.agents/skills/assura-goal-execution/SKILL.md
  - ../../.agents/skills/assura-goal-execution/references/execution-control-plane.md
  - ../../.agents/skills/assura-goal-execution/references/packet-coherence.md

---

# Assura maturity execution train

## Objective

Advance the canonical Trellis maturity task through one explicitly owned,
evidence-bound slice at a time. Continue through investigation, implementation,
validation, independent review, current-base integration, post-merge
reconciliation, and terminal cleanup; never end a continuation at a status
report while an authorized repair, review, integration, or cleanup action can
continue.

This is an execution-control goal, not a claim that process, evaluator,
canary, or documentation work satisfies a product card.

When the refreshed ledger has no ready-pending card, route the active,
implemented, verified and held records before considering any stop. Record the
exact action, owner, evidence, smallest resolution and independent work that
continues; a card-level hold is not a whole-goal block. Every continuation
must end with a live handle or one explicit next action, and the runtime goal
stays active until authorized scope is accepted and owned topology is terminal.

## Latest observed integration — PR #319 — `afa1637` (2026-09-12; refresh required)

PR #319 merged the reviewed R01 owner-approved archive reconciliation from
head `8d09ebd64bc764ce00337a4a957164ffc0a622d2` on base
`3c01e65ab609e40b4cdb9d53d4337577899d5a29` as
`afa1637ccceb93feff1bed5e652cf50516f68a9a` with tree
`d286f78b299f5c080d9e4f2f452bf8615f01ceb1`. Exact-head Documentation Scope,
CI Scope, Evidence Gates, Security Scope and GitGuardian passed; merge-SHA
Documentation (`34680275467`), Security Audit (`34680275427`) and Rust CI
(`34680275538`, including Evidence Gates `103517573693`) passed. Product,
Rust, performance and release rows were scope-skipped and remain non-proof.

The refreshed ledger records R01 as `not_needed` under Nick's explicit H01
archive decision in [PR #185](https://github.com/rothnic/assura/pull/185#issuecomment-5591702709).
This is an approved scoped exclusion, not a product-fix claim: the historical
failed run and missing causal callback fields remain preserved. The active
runtime goal remains active; A07 remains the next owner-controlled route, and
W02/F01 retain their external authority holds while W03 retains publication
authority.

This is the latest observed integration, not a live source claim. On every
continuation fetch `origin/master` again, rerun the revision-pinned ledger and
topology audit, and use the fetched SHA as the only current route.

## Historical checkpoint — PR #318 — 3c01e65 (2026-09-12; superseded by PR #319)

PR #318 merged the reviewed current-source continuation-route correction from
head `78fdfb0ca3f61873b50078e077d5744345a85101` on base
`a1d387f736d52e19ee0037bcd2e62146a21147d8` as
`3c01e65ab609e40b4cdb9d53d4337577899d5a29` with tree
`4a43a34a90f82a3b504b934c8ce90fa3e1c40e47`. Exact-head Documentation Scope,
CI Scope, Evidence Gates, Security Scope and GitGuardian passed. The
merge-SHA Documentation (`34679613211`), Security Audit (`34679613203`) and
Rust CI (`34679613179`) workflows passed; product, Rust, performance and
release rows were scope-skipped and remain non-proof.

At that observation this was the latest integration, not a live source claim.
On every continuation fetch `origin/master` again, rerun the revision-pinned
ledger and topology audit, and use the fetched SHA as the only current route. The ledger
refresh at this source is `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held. The reset found no
`origin/release` head; remote tags `v0.1.0`, `v0.2.0` and `v0.3.0` do not point
at this head. The report/strict topology summary before the owned closure
worktree was
`base=origin/master worktrees=55 dirty=2 prunable=3 unreadable=1
goal_branches=13 unmerged_goal=9`; unknown root dirt, the foreign dirty
worktree and stale registrations remain outside ownership.

Next action after refresh: keep A07 ahead of empty ready-pending rows and keep
the goal active. No live A07 candidate exists at `3c01e65`; confirm explicit
no-credit preparation authority before creating one clean exact-toolchain
candidate and running identity, sibling-free canary, current holdout/manifest
rebind and isolated protocol review in order. Without that authority, continue
the independently audited R01/W02/W03/F01 held-action routes recorded in the
task checkpoint. Any public update still needs independent review, applicable
gates, current-master integration and merge-SHA observation. Never reuse
candidate-base metadata or infer product success from process metadata or
skipped checks.

## Historical checkpoint — 20256c1 (post-merge reconciliation; superseded by 2d512c1)

PR #314 merged the reviewed e1c9 post-merge reconciliation from head
`c458ef3f9843848d10fabb7ba2ce32b5409d0e73` on base
`e1c9b78216b5736abd24fbd165ed7e1e383b1744` as
`20256c132b208bdbce5637d693c4ff8c4b03b5e6` with tree
`a90744735e7a0b157f41a48a0e78ca7665403efb`. Exact-head Documentation Scope,
CI Scope, Evidence Gates, Security Scope and GitGuardian passed. The
merge-SHA Documentation (`34675964341`), Security Audit (`34675964350`) and
Rust CI (`34675964340`) workflows passed; Rust CI CI Scope
(`103505761574`) and Evidence Gates (`103505781689`) passed. Product, Rust,
performance and release rows were scope-skipped and remain non-proof. The
revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3`: A07 active, W03 verified, R01/W02/F01 held.

The clean owned `docs/a07-postmerge-e1c9` branch/worktree was verified clean
and its reviewed tree matched the squash merge tree before removal. The a6ed
exact-toolchain candidate, identity controls, two sibling-free canaries,
six-holdout/two-condition packet, persisted validator and isolated
`PASS_NO_CREDIT` protocol review remain candidate-base/no-credit metadata;
the e1c9 process artifacts are now current-source history. The reviewer
artifact remains immutable; the coordinator's pre/post hash finalization and
all three corrected findings remain retained as evidence.

No screening, allocation, credit, product acceptance, release, deployment,
publication, invitation or protection authority exists. Raw evaluator,
fixture and transcript material remains private; ambient skill metadata and
the generated-hook verifier discrepancy remain explicit limitations. The VPS
lane remains held by the unresolved `vps-dev` alias, configured-host disk and
toolchain/job conditions; remote output cannot replace hosted or platform
proof. Unknown root dirt, foreign dirty work, stale registrations, unrelated
branches and unfavorable evidence remain outside this goal's ownership.

Next action: refresh release/tag, PR/CI, topology and the revision-pinned
ledger at `20256c1`; keep the active A07 route ahead of empty ready-pending
rows, then build a fresh exact-toolchain candidate in one clean owned
worktree only if the no-credit preparation route remains authorized. Re-run
identity, sibling-free canary, current holdout/manifest rebind and isolated
protocol review in order. Any public process update needs independent review,
applicable gates, current-master integration and merge-SHA observation before
the next refresh. Never reuse the a6ed packet for screening or acceptance,
and never infer product success from process metadata or skipped checks.

## Historical checkpoint — e1c9b78 (post-merge reconciliation; superseded by 20256c1)

PR #313 merged the reviewed packet-coherence process slice from head
`a134fc6815fc76730926d9f8ded57f16a8740e74` as
`e1c9b78216b5736abd24fbd165ed7e1e383b1744` with tree
`9d57888eec65d892dcc2a65b829369dd90278637`. Its exact-head and merge-SHA
Documentation, CI Scope, Evidence Gates, Security Scope and GitGuardian
checks passed for applicable scope. The a6ed candidate/packet remained
candidate-base/no-credit; the clean owned e1c9 reconciliation branch/worktree
was later verified clean with an equal merge tree and removed. No product,
screening, allocation, credit, acceptance or authority state changed.

## Historical checkpoint — a6ed20f (candidate-bound protocol disposition; no-credit; superseded by e1c9b78)

PR #312 merged the reviewed dcf2 process reconciliation from head
`f43d1a0cc6cd868a78d4f68dbc97a52a90476abb` as
`a6ed20f532d93f830099e9095798a09515fead0b` with tree
`bd399cd480dc7a31630740718dd8248a0c77bef5`. Exact-head Documentation Scope,
CI Scope, Evidence Gates, Security Scope and GitGuardian passed; merge-SHA
Documentation (`34671630183`), Security Audit (`34671630154`) and Rust CI
(`34671630166`) passed, including Rust CI CI Scope and Evidence Gates. Product,
performance and release rows were scope-skipped and remain non-proof. The
revision-pinned ledger is `items=32; ready_pending=0; unfinished=5; held=3`:
A07 active, W03 verified, R01/W02/F01 held.

The coordinator owns the clean candidate checkout
`/private/tmp/assura-a07-current-a6ed` on `goal/a07-current-a6ed`. The exact
Rust/Cargo `1.94.1` release build exited `0`; Assura `0.4.0` candidate and
login-shell shim both hash to
`1129d4498651fce2679264c5f2464931edd77e5e1bc0bcb3b02914ba1f42913c`.
Candidate identity, command help, login-shell resolution, and deliberate
wrong-target/wrong-root controls pass or reject as intended.

Two fresh sibling-free source-only conditions completed with child exit `0`;
post-exit evaluators exited `0`, covered all seven declared dimensions, and
reported zero critical failures. The private packet is bound to six unique
holdouts (two per stack), exactly two stable conditions and 30 reserved cells.
Its persisted metadata validator is `valid=true`, `validator_exit=0`,
`protocol_status=PASS_NO_CREDIT`, with six handles, 30 cells and two
conditions. The isolated protocol reviewer returned `PASS_NO_CREDIT` after
correcting `A07-A6ED-VALIDATOR-001` (missing persisted validator reference),
`A07-A6ED-REVIEW-TIME-002` (stale review timestamp), and
`A07-A6ED-CANARY-STATE-003` (freeze/canary status mismatch). The coordinator's
post-verdict finalization records the pre/post hash maps; its review SHA is
`b0e4949d6e6ac742d776aac4770d5b32150bec17e70bdcdd4df59ebeee7fca9f` and its
finalization SHA is
`f039b72ace76af33b252d59407de2ad2e44843abcf65d55537353a80c797609a`.

This remains no-credit preparation: screening, allocation, credit, product
acceptance, release, deployment, publication, invitation and protection
authority are all false. Raw evaluator output, fixtures and child transcripts
remain private; ambient skill metadata and the generated-hook verifier
discrepancy remain explicit limitations. The VPS lane remains held by the
unresolved `vps-dev` alias, configured-host disk pressure/toolchain mismatch,
and unrelated jobs; remote output cannot replace hosted or platform proof.

Next action: reconcile this current packet and layered-process update in one
reviewed current-base process slice. Before any later phase, fetch and verify
`origin/master`, release/tag, PR/CI, topology and the revision-pinned ledger;
if source advances, classify this packet candidate-base/no-credit and rebuild.
Do not start screening or allocation from canary/protocol metadata without a
separate explicit authority decision and the complete A07 acceptance contract.

## Historical checkpoint — 3e67d5f (candidate-bound canary; no-credit; superseded by `9c1b68a`)

The reset fetched `origin/master=3e67d5fe123c6ebcf3ac05617966919151e44868`
with tree `1b3b2c130a294b1020f0c63d84df120434bebf42`. The revision-pinned
ledger is `items=32; ready_pending=0; unfinished=5; held=3`: A07 is active,
W03 is verified, and R01/W02/F01 remain held. Root unknown dirt, a foreign
dirty worktree, unreadable/prunable registrations, historical branches and
unfavorable evidence remain outside this goal's ownership.

The owned candidate checkout is `/private/tmp/assura-train-current-3e67`,
built with exact Rust/Cargo `1.94.1`. Candidate reports version `0.4.0`;
binary and login-shell shim SHA-256 are both
`bbb244635770c44862db1becfab5ae46c107c904821baec303bcc52ae6bd05fa`.
Login-shell identity passes, while deliberate wrong-target and wrong-root
controls reject. The first condition attempt is retained as unfavorable
identity-mismatch evidence after a stale `ZDOTDIR` was found and corrected;
the corrected current runner is the only one used for the following results.

Two fresh sibling-free source-only fixtures then completed the fixed one-shot
initializer with exit `0`; post-exit evaluators pass all seven declared
dimensions with zero critical failures, expected negative policy rejection,
and a collected native test. The private current packet at
`/private/tmp/assura-a07-private-3e67` has six unique holdouts, exactly two
conditions and 30 reserved cells; the adapted metadata validator reports
`valid=true` with zero errors. Its canary summary is
`screening/canary-2026-09-12-current-3e67.json` and all allocation, screening,
credit and acceptance flags remain false. Independent metadata-only protocol
review is `PASS_NO_CREDIT` for the corrected packet; this makes the matrix
protocol-ready only, not screened or accepted. A subsequent process merge will
make this candidate candidate-base/no-credit again.

The optional VPS lane remains held: `vps-dev` is unresolved, while the
configured host has a nearly full root volume, unrelated long-running jobs,
and no exact pinned toolchain. Remote output cannot replace platform or
hosted proof. No product, threshold, screening, allocation, release,
deployment, publication, invitation or protection authority changed.

Next action: freeze a concise public evidence update, obtain independent review
and all applicable gates, merge only at current master, observe merge-SHA
workflows, then refresh source/ledger/topology and rebuild a post-merge
candidate before any screening. Never promote a canary or protocol pass to A07
acceptance; any new finding still requires a scoped correction and rereview.

## Historical checkpoint — eef7e1a (post-merge reconciliation; no-credit; superseded by `3e67d5f`)

PR #307 merged the reviewed post-merge pointer slice from head
`9723a6730ac4cff2c8567035a8cd4f1fe21e25f4` on base `66e0b7e` as
`eef7e1a7400c84ec35be33a58914ad07da7fa376`. Exact-head Documentation Scope,
CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed. The
merge-SHA Documentation (`34660366526`), Security Audit (`34660366986`) and
Rust CI (`34660366735`) workflows passed; Rust CI's CI Scope
(`103461414969`) and Evidence Gates (`103461446382`) succeeded, while
product/Rust/performance/release jobs were scope-skipped and remain non-proof.

The revision-pinned ledger remains 32 items, zero ready-pending rows, five
unfinished rows and three held rows: A07 is active, W03 is verified, and R01,
W02 and F01 retain their separate evidence or authority holds. The reviewed
`66e0b7e` process checkpoint and all `dc031527` candidate, canary and
protocol-pass artifacts remain historical candidate-base/no-credit; none may
be reused for screening or acceptance. No product, threshold, screening,
allocation, acceptance, release, deployment, publication or invitation
authority changed.

Current source is `eef7e1a` with tree
`1bfbcc27e5504ff23af489c1d076c4f6aba448a0`. The next continuation must refresh
release/tag, PR/CI, topology and the ledger, inspect active/verified/held work
before pending rows, and keep one explicitly owned recovery or fresh
current-master A07 preparation slice live. A fresh candidate, identity/no-
credit canary, holdout rebind and protocol review are required before any
separately authorized screening. Never stop at this post-merge status, an
empty pending queue, a skipped check or process evidence.

## Historical checkpoint — 66e0b7e (post-merge reconciliation; no-credit; superseded by eef7e1a)

PR #306 merged the reviewed process slice from head
`8576f64d9519ad4eaa529878ef9c8267d60fd1b2` on base `dc031527` as
`66e0b7e75f644dc4d047853488daee4a1cd716a3`. Exact-head Documentation Scope,
CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed. The
configured merge-SHA Documentation (`34658963096`), Security Audit
(`34658963070`) and Rust CI (`34658963059`) workflows also passed; Rust CI's
CI Scope (`103457282547`) and Evidence Gates (`103457314567`) succeeded, while
product/Rust/performance/release jobs were scope-skipped and remain non-proof.

The revision-pinned ledger remains 32 items, zero ready-pending rows, five
unfinished rows and three held rows: A07 is active, W03 is verified, and R01,
W02 and F01 retain their separate evidence or authority holds. The reviewed
`dc031527` candidate, canary and protocol-pass packet are now
candidate-base/no-credit after this source advance and must not be reused for
screening or acceptance. The clean owned PR branch/worktree was removed;
unknown root dirt, foreign dirty work, stale registrations and unfavorable
evidence remain preserved as ownership boundaries. No product, threshold,
screening, allocation, acceptance, release, deployment, publication or
invitation authority changed.

Current source is `66e0b7e` with tree `f08120b991b0fd51c311212e45ffeeff3c6a972c`.
The next action is to refresh release/tag, PR/CI, topology and the ledger,
inspect active/verified/held work before pending rows, and keep one explicitly
owned recovery or fresh current-master A07 preparation slice live. A fresh
candidate, identity/no-credit canary, holdout rebind and protocol review are
required before any separately authorized screening. Never stop at this
post-merge status, an empty pending queue, a skipped check or process evidence.

## Historical checkpoint — dc031527 (candidate-bound protocol-pass, no-credit preparation; superseded by 66e0b7e)

The latest reset fetched `origin/master=dc031527afd400be52dfd8fe9cfdabc7a6caa685`
after PR #305 merged the reviewed process reconciliation from `284e781`. This
is the current checkpoint for this continuation; every later phase must fetch
again before using it. PR #305's reviewed head was
`5363b4a5d2e39beed6087bea6a5f6fa2d2aaf17c`; applicable exact-head
Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks
passed, and merge-SHA Documentation (`34653158838`), Rust CI (`34653159004`)
and Security Audit (`34653158857`) completed successfully for applicable
scope. Rust CI's CI Scope (`103439684733`) and Evidence Gates (`103439731188`)
passed; product/Rust/performance/release and other scope-skipped rows are not
product proof.

The revision-pinned ledger remains 32 items, zero ready-pending rows, five
unfinished rows and three held rows: A07 is active, W03 is verified, and R01,
W02 and F01 retain their separate evidence or authority holds. The `284e781`
checkpoint and d228472 candidate are historical or candidate-base/no-credit
after this source advance and must not be reused. The root unknown path,
foreign dirty worktree, stale registrations and unfavorable evidence remain
preserved as ownership boundaries. No product, threshold, screening,
allocation, acceptance, release, deployment, publication or invitation
authority changed.

The fresh candidate in `/private/tmp/assura-a07-current-reconcile-dc031` uses
Rust/Cargo `1.94.1`, matching source/tree/binary/shim identity
(`dc031527`, `b68b66cc47f7e4081afd2dc21c1387de34b30c32`,
`3cdaf976d7da61043a5867cb17ffd789b92b484e26f1e7d3a7989bd25ce816cf`).
Login-shell identity, wrong-target and wrong-root controls pass/reject as
intended. Two sibling-free source-only canaries and full seven-dimension
post-exit evaluators pass with zero critical failures; ambient skill metadata
and the generated-hook verifier discrepancy remain explicit limitations and
no credit is assigned.

The private packet `/private/tmp/assura-a07-private-dc031` now contains six
current-candidate holdouts, exactly two conditions, 30 reserved cells and
corrected current receipt/construction/second-readonly aliases. Its
metadata-only validator is `valid=true` with zero errors. Independent
protocol review returned `PASS_NO_CREDIT` after correcting the stale contract
digest and construction hash; the private review artifact records both
findings and their scoped rereview. Screening, allocation, credit and product
acceptance remain false. Reconcile this evidence in a reviewed current-base
process slice, then refresh source, release/tag, PR/CI, topology and the
ledger after every merge; never let a canary, protocol metadata, skipped check
or empty pending queue end the goal.

## Historical checkpoint — 284e781 (d228472 candidate-base reconciliation; superseded by dc031527)

The latest reset fetched `origin/master=284e78156370d49d8315f04391fcc74eaba3acb2`
after PR #304 merged the reviewed post-merge process reconciliation from base
`d228472`. This is the current checkpoint for this continuation; every later
phase must fetch again before using it. PR #304's reviewed head was
`17fa9197ed79083be4b8ba0714851aeaae9bdbf3`; its applicable Documentation, CI
Scope, Security Scope, Evidence Gates and GitGuardian checks passed after
independent review `PASS`, and the configured push-triggered Documentation
(`34651068660`), Rust CI (`34651068675`) and Security Audit (`34651068699`)
workflows completed at the merge SHA for their applicable scope. Rust CI's CI
Scope (`103433100783`) and Evidence Gates (`103433136825`) passed; product/
Rust/performance/release and other scope-skipped rows are not product proof.
The revision-pinned ledger remains 32 items, zero ready-pending rows, five
unfinished rows, and three held rows: A07 is active, W03 is verified, and R01,
W02, and F01 retain their separate evidence or authority holds. The d228472
process checkpoint and candidate are now historical or candidate-base/no-credit
after this source advance and must not be reused. Their ambient-context
limitation and initial root-level freeze placement failure remain retained as
unfavorable evidence. The root unknown path, foreign dirty worktree, stale
registrations, historical branches and unfavorable evidence remain preserved as
ownership boundaries. No holdout/manifest rebind or protocol/screening/
allocation/product acceptance state carries forward.

The reusable execution-control-plane reference defines the state machine,
checkpoint fields, layered disclosure, validation budget, measured VPS test,
review contract, post-merge push-workflow fence and terminal topology fence.
It is a phase reference, not a second goal or a substitute for the task ledger.
The coordinator's next action is to refresh source/release/tag/PR/CI/topology
and the ledger, inspect active and held work, then keep one explicitly owned
action live. If A07 preparation is separately authorized, build a fresh
exact-toolchain candidate from `284e781`, run identity and sibling-free
no-credit canary gates, rebind the six immutable holdouts and exactly-two-
condition manifest, and obtain isolated protocol `PASS` before screening.
Otherwise continue an independently authorized held recovery slice. Never stop
at an empty pending queue.

PR #292's exact-head applicable checks passed, but its separate push-triggered
Rust CI run reported a macOS `watch_stops_cleanly_without_runtime_artifacts`
SIGINT failure and cancelled the Ubuntu/Windows matrix siblings. A focused
local rerun passed once; the hosted failure remains an unresolved diagnostic,
not a green signal and not a reason to weaken or blindly retry the gate.
PR #294's reviewed process slice then passed both its exact-head and
post-merge push-triggered checks at `c9ac2a84`; PR #295 subsequently passed its
exact-head and merge-SHA push-triggered Documentation, Security and Rust CI
workflows at `9047a3d`; PR #296 reconciled those results at `83c382a`; PR #297
reconciled that checkpoint at `6d57b86`; PR #298 reconciled that checkpoint at
`329bce0`; PR #299 reconciled that checkpoint at `651ef31`; PR #300
reconciled that checkpoint at `0dff804`; PR #301 reconciled that checkpoint at
`24a1966`; PR #302 reconciled that checkpoint at `453a32a`; PR #303
reconciled the current checkpoint at `d228472`. This
confirms the reconciliation fence is executable, but it does not repair the
older R01 failure. A configured post-merge failure, cancellation or missing
result keeps the specific recovery route open.

The read-only remote-capacity probe also keeps the VPS efficiency lane held:
`vps-dev` is not a configured alias, while `vps` has 16 CPUs, 61 GiB RAM and
20 GiB free on a 339 GiB root volume (95% used), runs unrelated long-lived
jobs, and exposes nightly Rust/Cargo rather than the candidate's pinned
toolchain. No remote heavy job is eligible until a named host passes an
exact-toolchain, free-disk and owned-job probe; remote Linux output never
replaces platform or hosted proof.

## User certainty bar

Every continuation must make these answers auditable:

1. What is the fetched source SHA, ledger state, owner, checkout, phase, and
   exact next action?
2. Which observable contract and negative controls prove the current slice?
3. Which review, local, hosted, performance, authority, or cleanup gate is
   still open, and what independent work continues meanwhile?
4. Why is a card still active, verified, held, or done without relying on a
   passing process check or a skipped/zero-test job?

## Scope

- Refresh source, release/tag availability, PR/CI state, ledger, ownership and
  topology at every reset, card boundary, merge, and handoff.
- Route active, implemented, verified, and merge-ready candidates before
  pending rows; choose one named owner and one concrete next action.
- Use clean explicit worktrees, focused red/green checks, exact evidence,
  independent review, current-base integration, and verified cleanup.
- Use layered context: keep `AGENTS.md` a universal router and load the task,
  card packet, and phase reference only when that decision requires them.
- Measure validation placement and use the VPS only after fresh capacity,
  disk, toolchain, and owned-job checks; retain hosted and platform-specific
  gates as final proof.

## Correction-derived controls

The latest A07 rereview is encoded as a reusable packet-coherence fence in
[`packet-coherence.md`](../../.agents/skills/assura-goal-execution/references/packet-coherence.md):

- before review, freeze an explicit role-to-path/hash map while packet status is
  `PENDING`, the review timestamp is null, and all authority flags are false;
- require the reviewer to verify that map without mutating packet state;
- after a verdict, update dependent statuses, timestamps and references in one
  coordinator-owned transition, rerun the validator with the exact disposition,
  and emit a post-verdict hash map linked to the immutable review SHA;
- treat a missing validator, stale timestamp, contradictory pending/pass prose,
  stale alias, or hash-map mismatch as a concrete finding that blocks the next
  transition until corrected and scoped-rereviewed.

This goal also keeps a single active runtime goal and one owned next action.
Compaction reloads only the universal, goal/phase, selected-card and one
special-lane layers; it never creates a replacement goal or copies private
packets into prompts. A process or protocol pass is evidence for its slice,
not a product-card completion signal.

## Non-goals and authority boundaries

- Do not weaken thresholds, hide failures, retry an unchanged failing method,
  or treat skipped, unavailable, zero-test, or scope-uncertain checks as pass.
- Do not mutate, stash, reset, commit, delete, or absorb unknown/user-owned
  dirt. Do not create duplicate goals for this active task.
- A07 preparation is separate from A07 implementation and acceptance. Its
  identity freeze, source-only no-credit canary, private two-condition packet,
  six-handle binding, 30-cell matrix, isolated protocol review, screening,
  holdout, follow-up feature, and final acceptance are separate gates; none
  grants screening or acceptance authority by itself.
- R01, W02, W03 publication, F01 outreach, release, deployment, invitation,
  protection changes, and CI-infrastructure changes retain their named holds
  or require their own explicit authority.

## Operating contract

1. **Reset and route.** Run the workflow gate, fetch `origin/master`, record
   release/tag and PR/CI availability, run the revision-pinned ledger and
   topology report, and inspect actual owner/session liveness. Preserve dirty,
   unreadable, prunable, and historical state rather than normalizing it away.
2. **Own and prove.** Work from a clean checkout based on the recorded source;
   keep one owner per candidate and commit coherent work before switching,
   yielding, reviewing, or handing off. Start with cheap structure/scope and
   focused red tests, then run only the smallest applicable validation tier.
3. **Review and correct.** Freeze the diff and public evidence, dispatch the
   Assura review brief, and continue independent authorized preparation while
   the reviewer or CI runs. Turn vague feedback into a contract, location,
   failure scenario, and smallest verification; fix accepted findings as a
   delta, rerun affected gates, and obtain scoped rereview.
4. **Integrate and reconcile.** Merge only reviewed current-base work with all
   applicable local/hosted/performance gates passing and no unresolved
   authority or zero-test issue. Fetch again, prove reachability and tree
   equality, rerun the ledger, run strict topology, and remove only the exact
   clean owned branch/worktree or record a verified archive.
5. **Continue.** If no pending row is executable, advance the active recovery
   or preparation slice. If an action is externally held, record its contract,
   evidence, owner, smallest resolution, and independent work that continues;
   do not convert that hold into a whole-goal stop.

## Definition of done

The goal is complete only when every authorized card outcome is proven by its
observable packet, review findings are resolved, required local and hosted
proof covers the final source, merged reachability is verified, and every
goal-owned branch/worktree is merged or verified-archived and clean. A process
merge, passing canary/evaluator, or protocol `PASS` alone never marks a card
done.

## Validation matrix

Run from the current clean owned checkout, substituting the task path as
needed:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex --task \
  .trellis/tasks/09-04-maturity-portfolio-strategy
python3 ./.agents/skills/assura-goal-execution/scripts/audit-context-routing.py \
  "$repo_root" .trellis/tasks/09-04-maturity-portfolio-strategy
/Users/nroth/.codex/skills/assura-orchestration/scripts/audit-topology.sh \
  "$repo_root" --report
git fetch origin master --prune
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

For Rust, Cargo, CI, release, behavior, or performance changes, add the
focused red test and the applicable `cargo fmt`, `cargo test`, `cargo clippy`,
benchmark, `cargo xtask pr`, release, and hosted checks from the selected card.
The strict topology audit is a required handoff gate; preserved foreign or
unknown state is reported, not silently repaired.

## Review tasks and blocking criteria

- R1: Verify current source/ledger pointers and that every older pointer is
  labeled as-of or historical.
- R2: Verify one owner, clean worktree discipline, exact candidate identity,
  and terminal closure evidence.
- R3: Verify focused tests, negative controls, expected test counts, and honest
  treatment of skipped/unavailable/performance checks.
- R4: Verify A07 privacy, candidate/fixture provenance, condition receipts,
  protocol scope, and separate screening authority.
- R5: Verify VPS/CI efficiency proposals are measurement-backed and do not
  replace hosted, platform, or protection gates.
- R6: Verify compaction/continuation always records a live handle or concrete
  next action and does not create a replacement goal.

Block integration if source is stale, ownership is ambiguous, review findings
are unresolved, required evidence is missing, performance thresholds regress,
checks are skipped or zero-test, authority is inferred, or the owned topology
cannot be closed or archived with an explicit recovery record.

## Progress log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-09-12 | Iteration 170: PR #313 merged the independently reviewed packet-coherence process slice from head `a134fc6` on base `a6ed20f` as `e1c9b78216b5736abd24fbd165ed7e1e383b1744`; exact-head Documentation Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian plus merge-SHA Documentation (`34675049056`), Security Audit (`34675049038`) and Rust CI (`34675049055`) passed for applicable scope, with Rust CI CI Scope (`103503298569`) and Evidence Gates (`103503318127`) successful. The clean owned a6ed candidate branch/worktree was verified ancestor-reachable and removed; its candidate/packet is now candidate-base/no-credit. Screening, allocation, credit, acceptance and all authority flags remain false. | PR #313; merge-SHA workflow records; topology refresh. |
| 2026-09-12 | Iteration 169: refreshed current master at `a6ed20f`, completed the exact-toolchain candidate, identity controls, two sibling-free no-credit canaries, six-holdout/two-condition packet validation and isolated protocol rereview. Corrected findings `A07-A6ED-VALIDATOR-001`, `A07-A6ED-REVIEW-TIME-002` and `A07-A6ED-CANARY-STATE-003`; coordinator finalization records immutable pre/post hashes. Added the layered packet-coherence reference and reconciled the goal/task/E2E routes. Context level: not exposed; current source, owner, packet disposition and next integration action are summarized in the checkpoint above. | Current candidate worktree and private protocol review/finalization records; context-routing audit 58/0; scoped structure, evidence and docs gates. |
| 2026-09-12 | PR #311 merged the reviewed 36e5 post-merge process reconciliation as `dcf2a9e` from head `6602ac8`; exact-head Documentation Scope, CI Scope, Evidence Gates, Security Scope and GitGuardian plus merge-SHA Documentation (`34670911182`), Security Audit (`34670911121`) and Rust CI (`34670911124`) passed. The 36e5 candidate/packet are now candidate-base/no-credit; their protocol `PASS_NO_CREDIT` remains historical evidence and a fresh current-master candidate is required before screening. No product, screening, allocation, credit or authority state changed. | PR #311; current process records; merge-SHA workflow records. |
| 2026-09-12 | PR #310 merged the reviewed post-merge reconciliation at `36e5a840` from head `c1b489c4`; applicable exact-head Documentation, CI Scope, Evidence Gates, Security Scope and GitGuardian plus merge-SHA Documentation (`34668192073`), Security Audit (`34668192076`) and Rust CI (`34668192074`) passed. The 9c1 candidate/packet are now candidate-base/no-credit. A fresh 36e5 candidate passed exact identity controls, seeded source-only A/B canaries and full seven-dimension evaluators; the prior empty-fixture failure remains unfavorable evidence, and the new six-holdout packet passed isolated protocol rereview `PASS_NO_CREDIT` after its final construction digest binding was corrected. No product, screening, allocation, credit or authority state changed. | PR #310; `research/evidence/A07.md`; `research/process-corrections-2026-09-11.md`; fresh private identity/canary/packet/review records. |
| 2026-09-11 | PR #304 reconciled the reviewed process route at `284e781`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as did merge-SHA Documentation (`34651068660`), Rust CI (`34651068675`) and Security Audit (`34651068699`) for applicable scope. The d228472 candidate canary remains explicitly candidate-base/no-credit after this source advance, with its ambient-context limitation and initial structure-placement failure retained. No packet rebind, isolated protocol review, screening, allocation or product acceptance state carried forward. | PR #304; `research/evidence/A07.md`; `research/progress-current.md` Iteration 164; merge-SHA workflow records. |
| 2026-09-11 | PR #303 reconciled the reviewed process route at `d228472`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as did merge-SHA Documentation (`34646077514`), Rust CI (`34646077520`) and Security Audit (`34646077494`) for applicable scope. A fresh exact-toolchain d228472 candidate then passed identity controls, two sibling-free source-only initializer canaries, and full seven-dimension evaluators with zero critical failures. Generic ambient skill metadata remained visible and is recorded as an isolation limitation; an initial root-level freeze placement failed the structure gate and was corrected under ignored `target/a07-evidence/`. This is no-credit preparation only; packet rebind, isolated protocol review, screening, allocation and product acceptance remain false. | PR #303; `research/evidence/A07.md`; `research/progress-current.md` Iteration 163; private identity/freeze and redacted canary/evaluator receipts. |
| 2026-09-11 | PR #301 merged the reviewed current-source routing/skill reconciliation as `24a1966`; exact-head applicable checks and merge-SHA Documentation (`34642991331`), Rust CI (`34642991392`) and Security Audit (`34642991344`) workflows passed for their applicable scope. The 0dff804 A07 candidate is now candidate-base/no-credit; no product, threshold, screening, allocation, acceptance, authority or release state changed. | PR #301; `research/progress-current.md` Iteration 161; post-merge workflow records. |
| 2026-09-11 | The current-source recovery, A07 binding, E2E and executor routes were reconciled to `0dff804`; older pointers are historical, the layered execution skill now requires byte-identical freeze records and concrete review rereview, and scoped gates pass. This process slice remains separate from A07 product/screening success. | `research/progress-current.md` Iteration 160; `research/recovery-plan.md`; `research/a07-candidate-binding-plan.md`; context-routing audit (`55/0`); `cargo xtask evidence`; `cargo xtask docs`. |
| 2026-09-11 | `/root` refreshed `origin/master=0dff804` and froze a fresh A07 candidate in a clean owned checkout with pinned Rust/Cargo `1.94.1`. The release binary and login-shell shim identities match the source/tree; deliberate wrong-target and wrong-root controls reject, `assura check --format json .` exits `0` with zero blocking violations, and independent review passes after the freeze records are made byte-identical. This is no-credit preparation only; canary, holdout/manifest rebind, protocol, screening and allocation remain false. | `research/progress-current.md` Iteration 159; `research/process-corrections-2026-09-11.md`; private freeze and identity records under `/private/tmp/assura-a07-private-0dff`; scoped review PASS. |
| 2026-09-11 | PR #299 merged the reviewed current-checkpoint reconciliation as `651ef31`; its exact-head applicable checks passed, the reviewed tree equals the merge tree, and merge-SHA push-triggered Rust CI (`34635475363`), Documentation (`34635475400`) and Security Audit (`34635475378`) completed successfully. Product/Rust/performance/release rows were explicitly skipped by scope and remain non-applicable. This process result updates current pointers only; no product, threshold, allocation, screening or authority state changed. | PR #299; `research/progress-current.md` Iteration 158; post-merge run records. |
| 2026-09-11 | PR #298 merged the reviewed current-checkpoint reconciliation as `329bce0`; its exact-head applicable checks passed, the reviewed tree equals the merge tree, and merge-SHA push-triggered Rust CI (`34633883919`), Documentation (`34633883756`) and Security Audit (`34633883854`) completed successfully. Product/Rust/performance/release rows were explicitly skipped by scope and remain non-applicable. This process result updates current pointers only; no product, threshold, allocation, screening or authority state changed. | PR #298; `research/progress-current.md` Iteration 157; post-merge run records. |
| 2026-09-11 | PR #297 merged the reviewed current-checkpoint reconciliation as `6d57b86`; its exact-head applicable checks passed, the reviewed tree equals the merge tree, and merge-SHA push-triggered Rust CI (`34632545493`), Documentation (`34632545635`) and Security Audit (`34632545507`) completed successfully. Product/Rust/performance/release rows were explicitly skipped by scope and remain non-applicable. This process result updates current pointers only; no product, threshold, allocation, screening or authority state changed. | PR #297; `research/progress-current.md` Iteration 156; post-merge run records. |
| 2026-09-11 | PR #296 merged the reviewed post-merge reconciliation as `83c382a`; its exact-head applicable checks passed, the reviewed tree equals the merge tree, and merge-SHA push-triggered Rust CI (`34630104586`), Documentation (`34630104663`) and Security Audit (`34630104624`) completed successfully. Product/Rust/performance/release rows were explicitly skipped by scope and remain non-applicable. This process result updates current pointers only; no product, threshold, allocation, screening or authority state changed. | PR #296; `research/progress-current.md` Iteration 155; post-merge run records. |
| 2026-09-11 | PR #295 merged the reviewed workflow-fence and measured-capacity correction as `9047a3d`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as did the merge-SHA push-triggered Documentation (`34628874282`), Security Audit (`34628874295`) and Rust CI (`34628874311`) workflows. Product/Rust/performance/release jobs were explicitly skipped by scope and remain non-applicable. The merge tree equals the reviewed head tree; no product, threshold, allocation, screening or authority state changed. | PR #295; `research/progress-current.md` Iteration 154; post-merge run records. |
| 2026-09-11 | A read-only capacity probe kept the optional VPS lane held: `vps-dev` is not resolvable; configured `vps` has 16 CPUs/61 GiB RAM but only 20 GiB free of 339 GiB root (95% used), unrelated long-running jobs, and Rust/Cargo `1.95.0-nightly` instead of the pinned candidate toolchain. No remote state changed and no remote result is counted as proof. | `research/progress-current.md` Iteration 153; `research/process-corrections-2026-09-11.md`; bounded SSH probe. |
| 2026-09-11 | PR #294 merged the reviewed execution-control-plane correction as `c9ac2a8`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as did the configured post-merge Rust CI, Documentation and Security workflows. The correction makes merge-SHA push workflows a separate reconciliation gate. A retained read-only diagnosis bound R01's macOS failure to job `103316578631`, `tests/watch_cli.rs:196`, where `watch_stops_cleanly_without_runtime_artifacts` did not stop after SIGINT; the focused local Darwin rerun passed once and remains non-substitutive. No product, threshold, allocation, screening or authority state changed. | PR #294; push run `34625737610`; retained failure `34615572565`; Iteration 152 in `research/progress-current.md`. |
| 2026-09-11 | PR #292 merged the reviewed agent-content-preservation recovery slice as `4560c710`; exact-head Documentation, CI Scope, Security Scope, Evidence Gates, Rust/platform tests, performance, release/adoption smoke and GitGuardian checks passed (Security Audit was scope-skipped). The subsequent push-triggered Rust CI run failed in the macOS watch SIGINT test and cancelled Ubuntu/Windows siblings; this remains retained unfavorable hosted evidence. | PR #292; hosted check rollup and push run `34615572565`; focused local `watch_stops_cleanly_without_runtime_artifacts` rerun passed once. |
| 2026-09-11 | Rebuilt the A07 candidate and no-credit protocol packet at current `origin/master=4560c710`: exact login-shell identity with wrong-target/wrong-root rejects, two fresh sibling-free source-only canaries with all seven evaluator dimensions passing, six immutable holdouts, second-readonly pass, current fixture/invariant/evaluation bindings, exactly two conditions and 30 reserved cells. The packet was initially pending protocol review; the subsequent scoped rereview PASS is recorded below. Screening and credit remain false. | Private packet under `/private/tmp/assura-a07-private-4560c71`; candidate freeze, identity, canary receipts/evaluations, holdout binding, fixture freshness, invariants, evaluation binding, mapping and manifest. |
| 2026-09-11 | Recorded the scoped independent protocol rereview as `PASS` after repairing the explicit receipt-condition aliases and current construction digest. The private packet is now protocol-pass/no-credit; screening, allocation, credit and product acceptance remain false. | `/private/tmp/assura-a07-private-4560c71/screening/protocol-review-current-4560c71.json`; reviewed hashes are preserved separately from post-verdict hashes. |
| 2026-09-11 | Independent process review returned `PASS` for the committed goal/task reconciliation at `831dab5`: current-source routing, historical supersession, hosted-failure honesty, layered context, one-owner continuation and the A07 line limit were verified; no product or authority expansion was found. | `/root/context_routing_review_fast`; review covered `da432ad` plus the committed reconciliation. |
| 2026-09-11 | PR #289 merged the reviewed durable goal, layered-routing skill metadata and current-route corrections as `40f1155c`; the 851a6 candidate packet is now candidate-base/no-credit and must be rebuilt after the next source refresh. | PR #289; `research/progress-current.md` Iteration 147; post-merge ledger/context/structure/evidence/docs gates. |
| 2026-09-11 | Created the durable active execution goal from the corrected Trellis recovery contract. The current source is `851a6b8`; A07 is the only active lane while the ledger has no ready-pending card. | `.trellis/tasks/09-04-maturity-portfolio-strategy/task.json`; `research/orchestration-plan.md`; `research/process-corrections-2026-09-11.md`; `research/recovery-plan.md`; workflow gate, ledger and topology reset evidence. |
