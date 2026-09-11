# Recovery process verification

Date: 2026-09-11. Scope: process artifacts, agent instructions and validation
routing only. Product acceptance is unchanged; no card is promoted by this
file. The historical recovery record is preserved in
[recovery-history-2026-09-11.md](recovery-history-2026-09-11.md).

## Current post-merge reconciliation — PR #304 — 2026-09-11 UTC (`origin/master=284e781`)

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
