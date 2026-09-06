# Maturity execution train progress

## Iteration 1 — 2026-09-05 — B00 baseline capture

- Refreshed `origin/master` at `ed093668918bc271fc98b9112acaf7c1bf3eb314` and inventoried GitHub state, existing worktrees, release version, overlapping PR #142, and case-study PR #60.
- Preserved the previously uncommitted plan in dedicated PR #143; rebased it directly on current master after independent review identified that evidence must remain in execution ancestry.
- Captured B00 evidence in `research/evidence/B00.md`. Current hosted evidence exposes a real macOS watch assertion failure; it remains failing and blocks the planning handoff from merging alone.

## Iteration 2 — 2026-09-05 — R01 ownership and investigation

- Created isolated worktree `/Users/nroth/.codex/worktrees/assura-r01-watch-scope` on `goal/r01-watch-scope`, parented on the dedicated planning handoff.
- Marked R01 active and assigned its narrowly scoped test-first repair. Investigation shows macOS `need_rescan` truthfully requires `full_rescan_event`; R01 must retain overflow safety while making the contract portable and instrumented.
- Next: review the local R01 implementation, obtain its independent review, then run required hosted platform proof before considering it done.

## Iteration 3 — 2026-09-05 — R01 local proof and context health

- R01 is locally verified at `b52dc4db7c986ca305b0f594d5d23b99543da29a`: the targeted suite passed 13/13, library tests passed 514 Assura plus 15 watch-state tests, and the scoped integration passed 20 consecutive runs after the full tier was green.
- Context health review: the repeated local root Cargo rebuild stalled without CPU and was interrupted rather than retried unchanged; the isolated R01 worktree completed its own verification normally. The durable policy mismatch is card evidence naming (`<ID>.md`) versus the current kebab-case self-policy, explicitly assigned to independent ready card R07. No new skill is warranted: the existing `assura-structure-fit` skill already routes that decision.
- Next: independent review of the exact R01 SHA, then hosted macOS/Linux/Windows proof. Do not call R01 done until that proof exists.

## Iteration 4 — 2026-09-06 — R01/R02 integrated proof and post-merge closure

- PR [#144](https://github.com/rothnic/assura/pull/144) merged as `dcb1fb57ba100f77a7cb7e48c4f14507d3106231`; it was confirmed reachable from `origin/master` after fetch.
- Final reviewed head `4741e560c28620a0a8813f9a8635ae52c67ca3cc` passed all hosted checks in Rust CI run [34004956631](https://github.com/rothnic/assura/actions/runs/34004956631), including macOS/Linux/Windows suites, four-platform adoption, installer smoke, release bundle, and Performance Report. R01 and R02 are done.
- Context health review: three Windows reruns exposed fixture-only `CRCRLF` construction errors; each was preserved as RED evidence, independently reviewed, and narrowed until the actual Windows suite passed. The repeated discovery is specific to this temporary contract fixture; no new reusable skill is warranted. R07 remains active because current self-check still reports 18 advisory line-length findings that need individual dispositions.
- Next ready card: R03 (many-scope performance repair). P01 is also locally ready but remains a separate documentation integration.
