# Earlier maturity execution progress history

These entries were moved verbatim from `progress.md` on 2026-09-10 UTC to
keep the live progress index under its configured 1,000-line limit. They are
historical evidence only; use the newest `progress.md` entry and current
source-pinned card evidence for routing.

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
