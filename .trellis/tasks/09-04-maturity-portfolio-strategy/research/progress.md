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

## Iteration 5 — 2026-09-06 — P01/R03 integration and R04 MSRV closure

- PR [#146](https://github.com/rothnic/assura/pull/146) merged P01 scope evidence as `380b9bc889b8c653b57da375bd7bf06f174f8f2f`; PR [#147](https://github.com/rothnic/assura/pull/147) merged R03's truthful performance evidence as `3ff8c889e18381e83cc960803ccf9ddfba35f1d3`. R03 remains blocked, not failed-closed: the exact GitHub Linux comparison regressed and a comparable Linux rerun cannot yet be run because `vps-dev` is unresolved.
- PR [#148](https://github.com/rothnic/assura/pull/148) merged R04 as `e6f3a8e70068ea44a51a6d2626eaece1256e28b2`, then `git merge-base --is-ancestor` confirmed it reachable from `origin/master`. The merged Rust `1.86.0` floor is backed by a hosted all-features MSRV Clippy lane, focused `git-signals` proof, all required platform/adoption/installer/release/performance/docs/security gates, and two independent-review passes on the final PR tip.
- The initial hosted R04 run exposed 16 current-Clippy findings; the first correction exposed that the public optional `git-signals` feature did not meet the stated MSRV. Both findings were repaired instead of suppressed or scoped away, and the evidence records the exact failed and corrected SHA lineage. Next ready work remains R05 inspection and R07's individually dispositioned advisory backlog; R03 requires an external comparable Linux runner.

## Iteration 6 — 2026-09-06 — post-merge queue reconciliation and context health

- P01 is now recorded done only because PR #146's `380b9bc889b8c653b57da375bd7bf06f174f8f2f` is reachable from current master and its source-scoped hosted gates passed. Skipped Rust behavior jobs remain explicitly skipped, not counted as tests.
- R05 selection inspected the existing user-owned PR #142 without modifying it: it is stale against current master and retains a failed macOS test, failed Alpine adoption job, and cancelled Windows test. A new isolated current-master port owns any further R05 work.
- Context level: not exposed. Current working facts: (1) R01/R02/P01/R04 have merged evidence; (2) R03 is a documented external comparable-Linux-runner block; (3) R07 has 18 advisory findings requiring individual dispositions; (4) R05 is being ported from #142 without taking over its stale branch; (5) no tag/release/deploy authority has been used. Repeated lessons are already covered by the existing goal-execution and structure-fit skills, so no new skill is warranted.
