# Recovery process verification

Date: 2026-09-11. Scope: process artifacts, agent instructions and validation
routing only. Product acceptance is unchanged; no card is promoted by this
file. The historical recovery record is preserved in
[recovery-history-2026-09-11.md](recovery-history-2026-09-11.md).

## Current post-merge reconciliation — PR #306 — 2026-09-11 UTC (`origin/master=66e0b7e`)

- PR #306 merged the reviewed process route from head
  `8576f64d9519ad4eaa529878ef9c8267d60fd1b2` on base `dc031527` as
  `66e0b7e75f644dc4d047853488daee4a1cd716a3`. Exact-head Documentation,
  CI Scope, Security Scope, Evidence Gates and GitGuardian passed. Merge-SHA
  Documentation `34658963096`, Security Audit `34658963070` and Rust CI
  `34658963059` passed for applicable scope; CI Scope `103457282547` and
  Evidence Gates `103457314567` succeeded. Scope-skipped product, Rust,
  performance and release rows remain non-proof.
- The ledger remains `items=32; ready_pending=0; unfinished=5; held=3` with
  A07 active, W03 verified and R01/W02/F01 held. The `dc031527` candidate and
  packet are candidate-base/no-credit after the source advance; the protocol
  `PASS_NO_CREDIT` is historical metadata only. The owned branch/worktree was
  clean and removed; unknown or foreign dirt, stale registrations and
  unfavorable evidence remain outside ownership. No product or authority
  state changed.

Next action: refresh source/release/tag/PR/CI/topology and the ledger before a
new phase. If A07 preparation is authorized, rebuild the candidate at
`66e0b7e`, rerun identity and no-credit canary gates, rebind the six holdouts
and exactly-two-condition manifest, and obtain a scoped isolated protocol
`PASS`; otherwise continue an independently authorized held recovery slice.
Keep the goal active and never infer product success from process, canary,
protocol, skipped or zero-test evidence.

## Historical candidate and packet correction — PR #305 — 2026-09-11 UTC (`origin/master=dc031527`; superseded by 66e0b7e)

- PR #305 merged the reviewed process route from `284e781` as
  `dc031527afd400be52dfd8fe9cfdabc7a6caa685`; exact-head applicable checks and
  merge-SHA Documentation `34653158838`, Rust CI `34653159004` (CI Scope
  `103439684733`, Evidence Gates `103439731188`) and Security Audit
  `34653158857` passed. Scope-skipped product/performance/release jobs remain
  non-proof.
- The ledger remains `items=32; ready_pending=0; unfinished=5; held=3` with
  A07 active, W03 verified and R01/W02/F01 held. The clean candidate checkout
  `/private/tmp/assura-a07-current-reconcile-dc031` uses exact Rust/Cargo
  `1.94.1` and matching source/tree/binary/shim identity; two source-only
  canaries and full seven-dimension post-exit evaluators pass with zero
  critical failures. The canary is no-credit and retains ambient metadata and
  hook-verifier limitations.
- The six-holdout/two-condition private packet was rebound at
  `/private/tmp/assura-a07-private-dc031`. A metadata-only validator resolves
  current receipt, construction and second-readonly aliases, proves six
  unique handles and 30 reserved cells, and returns `valid=true` with zero
  errors. Historical construction records remain preserved; no screening,
  allocation or credit flag is true. Independent protocol review returned
  `PASS_NO_CREDIT` after correcting the stale contract digest and construction
  hash; the private artifact records the scoped rereview.

Next action: reconcile this protocol-pass/no-credit evidence only in a
reviewed current-base process slice; after any merge refresh source and the
ledger before further work. If source advances, classify the packet
candidate-base/no-credit and rebuild. Keep the goal active and do not treat an
empty pending set or process evidence as product success.

## Historical post-merge reconciliation — PR #304 — 2026-09-11 UTC (`origin/master=284e781`; superseded by dc031527)

- PR #304 merged reviewed process artifacts from head `17fa919` on base
  `d228472` as `284e78156370d49d8315f04391fcc74eaba3acb2`. Independent review
  returned `PASS`; exact-head Documentation, CI Scope, Security Scope,
  Evidence Gates and GitGuardian passed. Merge-SHA Documentation
  `34651068660`, Rust CI `34651068675` (CI Scope `103433100783`, Evidence
  Gates `103433136825`) and Security Audit `34651068699` completed for
  applicable scope. Scope-skipped product/performance/release jobs remain
  non-proof.
- The revision-pinned ledger is `items=32; ready_pending=0; unfinished=5;
  held=3` with A07 active, W03 verified and R01/W02/F01 held. The d228472
  candidate and canary/evaluator results are now candidate-base/no-credit;
  their ambient-context limitation and initial structure-placement failure are
  retained, but no holdout, manifest, protocol, screening or acceptance state
  carries forward.
- The owned process branch/worktree was clean, merged and removed. Unknown
  root dirt, foreign dirty work, stale/prunable registrations and unfavorable
  R01 evidence remain preserved. This reconciliation changes routing evidence
  only.

Next action: refresh all source/ownership/ledger facts before a new phase. If
A07 preparation is separately authorized, build a fresh `284e781` candidate
and repeat identity, no-credit canary, packet rebind and isolated protocol
review in that order. Otherwise continue an independently authorized held
recovery slice. Keep the goal active; do not reuse d228472 evidence or infer
product success from process or skipped checks.

## Historical candidate-bound canary — 2026-09-11 UTC (`origin/master=d228472`, superseded by `284e781`)

- PR #303 merged the reviewed process reconciliation at
  `d2284724192dbca848bbd135afacc33d2533e06f`; exact-head and applicable
  merge-SHA Documentation (`34646077514`), Rust CI (`34646077520`) and
  Security Audit (`34646077494`) checks passed. The reviewed tree equals the
  merge tree. The ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3` with A07 active, W03 verified and R01/W02/F01 held.
- The clean owned candidate `/private/tmp/assura-a07-current-d228` was built
  with Rust/Cargo `1.94.1`; source/tree/binary/shim and login-shell identity,
  wrong-target and wrong-root controls agree. Two fresh sibling-free
  source-only children and full seven-dimension evaluators passed with zero
  critical failures. Event scans found no private evaluator, foreign checkout,
  repository checkout or sibling-condition references; generic ambient
  user-level skill metadata appeared and remains an explicit isolation
  limitation. The first root-level freeze placement failed the structure gate
  and was corrected under ignored `target/a07-evidence/`.
- This is no-credit preparation only. No current holdout/manifest rebind,
  isolated protocol review, screening, allocation or product acceptance exists.
  Next action is a current six-handle/two-condition rebind and isolated
  protocol `PASS`; if a process merge advances source, classify this candidate
  candidate-base/no-credit and repeat the fresh-source sequence.

Context level: not exposed. At compaction or handoff reload AGENTS, the
workflow-gate result, current source/ledger, this index and only the next
phase reference; do not load the historical archive unless a specific finding
requires it.
