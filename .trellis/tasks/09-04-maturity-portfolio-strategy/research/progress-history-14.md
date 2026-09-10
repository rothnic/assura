# Preserved progress history: iterations 44–47

These older entries are kept outside the compact current progress log. They
are historical evidence and must not be used as a live source or queue.

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

## Historical index

- Initial B00/R01 iterations 1–4 remain in
  [`progress-early-history.md`](progress-early-history.md); iterations 5–11
  are preserved in [`progress-history-09.md`](progress-history-09.md).
- Iteration 114 is preserved in
  [`progress-history-10.md`](progress-history-10.md).
- Iteration 111 is preserved in
  [`progress-history-11.md`](progress-history-11.md).
- Iterations 32–33 are preserved in
  [`progress-history-13.md`](progress-history-13.md).
