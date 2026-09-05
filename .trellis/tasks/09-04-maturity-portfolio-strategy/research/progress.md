# Maturity execution train progress

## Iteration 1 — 2026-09-05 — B00 baseline capture

- Refreshed `origin/master` at `ed093668918bc271fc98b9112acaf7c1bf3eb314` and inventoried GitHub state, existing worktrees, release version, overlapping PR #142, and case-study PR #60.
- Preserved the previously uncommitted plan in dedicated PR #143; rebased it directly on current master after independent review identified that evidence must remain in execution ancestry.
- Captured B00 evidence in `research/evidence/B00.md`. Current hosted evidence exposes a real macOS watch assertion failure; it remains failing and blocks the planning handoff from merging alone.

## Iteration 2 — 2026-09-05 — R01 ownership and investigation

- Created isolated worktree `/Users/nroth/.codex/worktrees/assura-r01-watch-scope` on `goal/r01-watch-scope`, parented on the dedicated planning handoff.
- Marked R01 active and assigned its narrowly scoped test-first repair. Investigation shows macOS `need_rescan` truthfully requires `full_rescan_event`; R01 must retain overflow safety while making the contract portable and instrumented.
- Next: review the local R01 implementation, obtain its independent review, then run required hosted platform proof before considering it done.
