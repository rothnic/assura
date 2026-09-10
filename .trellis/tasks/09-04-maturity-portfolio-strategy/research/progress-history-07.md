# Preserved progress history — iterations 104 and earlier

This file preserves older entries moved out of the compact progress index to
keep the active log within the repository's line budget.

## Iteration 104 — 2026-09-10 — continuation-control route on current master

- Context level: not exposed. Current source is freshly fetched
  `origin/master=2eda17e`; the PR #249 candidate `5f17f7f` was based on
  `755c28d` and its merged tree matches. This process slice adds a concise
  continuation-control reference and reconciles active source labels; it does
  not change product, evaluator, threshold, allocation or authority state.
- The revision-pinned ledger has 32 items, `ready_pending=0`, five unfinished
  and three held: A07 active, W03 verified, R01/W02/F01 held. No pending row is
  executable, so the coordinator retains an owned next action rather than
  stopping or creating a duplicate goal.
- Independent impasse/process review recorded `A07-MANIFEST-04` as evidence
  missing: the private exactly-two-condition manifest, supplied-input proof,
  blinded mapping, six frozen holdouts, complete 30-cell matrix and isolated
  protocol-review `PASS` are absent. A07 remains active with zero screening,
  holdout or final-acceptance credit; private values and raw runs stay private.
- The next action is owned by the A07 acceptance coordinator: create or locate
  and validate that private manifest in an isolated protocol review. After a
  redacted `PASS`, fetch again, bind the candidate identity and run a fresh
  no-credit canary before allocating any cells. R01/W02/F01 and W03 remain
  separate held/authority routes.
- A serialized `vps` probe is capacity evidence only (16 CPUs, low load,
  roughly 42.7 GiB memory and 23 GiB disk free at 94% use; nightly Rust 1.95,
  pnpm 10.29.3, no Bun, busy host). No heavy job ran; local cheap gates and
  hosted final proof remain authoritative.
- Parallel local Cargo probes briefly contended on shared package/artifact
  locks; they were serialized afterward. The validation-routing skill now
  treats lock wait as resource timing, never as test progress or proof.
- Topology while this owned candidate is present is `worktrees=35 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9`; report is 0 and strict is 1 only for
  preserved unknown/user dirt, external work and historical registrations; the
  count includes this clean process worktree and returns to 34 after merged
  closure. The owned continuation worktree is clean; final handoff must rerun
  report and strict, then remove only its merged worktree/ref.

## Iteration 100 — 2026-09-10 — post-merge closure for PR #245

- The reviewed reconciliation candidate `391178676931ba935ad0058fce0bc55e101b3641` merged as PR #245 at `27ef54d489847e41e5907f7c74f870a2391a7dae`; exact ancestry from `origin/master` was verified after fetch. Its applicable Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed; product/Rust/performance/release/installer/website checks were explicit skips and remain non-applicable.
- The owned `docs/train-continuation-reconcile` worktree was clean and removed, and its local/remote refs were deleted only after merged reachability. No owned goal worktree or uncommitted process change remains. The preserved root unknown file, external dirty worktree, three prunable/one unreadable registrations and historical unmerged goal refs remain outside ownership.
- The final post-cleanup topology report exited `0` and strict exited `1` with `worktrees=34 dirty=2 prunable=3 unreadable=1 goal_branches=13 unmerged_goal=9`; the strict nonzero state is the preserved external/history set, not abandoned work from this slice.
- A fresh immutable checkout at `27ef54d` reports 32 ledger items, `ready_pending=0`, `unfinished=5`, `held=3`: A07 active, W03 verified, R01/W02/F01 held. No card, evaluator, threshold, release, deployment, publication or invitation state changed. The supported runtime goal remains active; next action is A07's private exactly-two-condition/six-holdout manifest, isolated protocol-review `PASS`, and fresh candidate-bound canary after refreshing current master.

## Iteration 99 — 2026-09-10 — post-merge continuation reconciliation for PR #244

- The read-only reset fetched `origin/master=2902a073f39f8e8a47a9658a6358791c3e7f4655`, the merge of reviewed final head `03a9b01eb1a5742b0cc84373e53f3f9aed8d0f1e` for PR #244. `git merge-base --is-ancestor` exited `0`; the exact final hosted checks passed Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian. Product/Rust, performance, release, installer and website jobs were scope-skipped and remain non-applicable rather than passing proof.
- The owned `docs/train-continuation-control` worktree was clean, removed, and its local/remote branch was deleted only after merged reachability. The root strategy checkout still owns one unknown untracked A04 research file; the external dirty worktree, three prunable registrations, one unreadable registration and historical unmerged goal refs remain outside this slice and were not changed.
- The revision-pinned ledger at `2902a07` still reports 32 items, `ready_pending=0`, `unfinished=5`, `held=3`: A07 is active, W03 verified, and R01/W02/F01 retain exact holds. No card state, acceptance result, release, deployment, publication or invitation was changed.
- Process coordinator / `reconcile-handoff` owns this checkpoint in `/private/tmp/assura-train-continuation-reconcile` on branch `docs/train-continuation-reconcile`. The next real action is A07's private exactly-two-condition manifest and six-holdout validation, isolated protocol-review `PASS`, then a fresh candidate-bound canary against `2902a07`; held R01/W02/F01 and W03 publication decisions remain separate. The goal stays active; an empty ready queue is not a stopping condition.
