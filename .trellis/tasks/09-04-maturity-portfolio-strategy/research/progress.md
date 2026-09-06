# Maturity execution train progress

## Iteration 33 — 2026-09-06 — A01 current-master integration

- PR [#162](https://github.com/rothnic/assura/pull/162) merged as `fdd0e76426c9ca6916fa72cdb3948378ad3a92e3`; a fresh fetch proved that merge is reachable from `origin/master`. The exact independently reviewed head was `1522352cb8b817620c4ea773780877332e122919`.
- Rust CI, Documentation, Security Scope, and GitGuardian hosted checks passed. The scope-directed Security Audit job was skipped and is not represented as a passing test. A01 is now done as the partial evaluator contract, not as end-to-end initialization acceptance.
- Context health: the long release/build commands in isolated worktrees can outlive an output window, so completed exits are being re-captured individually rather than inferred. This is an execution-observation issue, not a new reusable project skill. Next: finish current-master Q04 verification/review while A02 is dependency-ready.

## Iteration 32 — 2026-09-06 — A01 evaluator trust-boundary closure

- Rebased A01 onto current master `6f72bf3`, then independent review identified and the candidate repaired absolute candidate-binary enforcement, named-rule matching for negative probes, required negative policy evidence, stdout/stderr zero-test detection, and required SHA-256 fixed-prompt provenance. The final reviewed SHA is `83687478a7b399e318f917edfa5641cce4ad95d1`; final independent review found no remaining findings.
- Focused RED tests were observed for every review condition. The final local evaluator suite passed 27 tests; Python compilation, repository structure check, evidence policy, and the 48-page documentation build passed. Rust, TypeScript, and Python policy-only partial controls all recorded matched named-rule negative probes and remain ineligible by scope; those runs do not assert unrequested native, guidance, or hook dimensions. A separate full Rust control records native, guidance, and hooks as unavailable; TypeScript/Python full-run evidence remains separately incomplete.
- Context health: earlier Q02 and unrelated-card evidence show R03's no-slower gate can fluctuate on the same fixture, so hosted proof remains mandatory. Existing evaluator and performance workflows already cover the new findings; no new skill is warranted. Next: await the exact-SHA hosted matrix, then merge only if every required job passes and the candidate remains current-master based.

## Iteration 33 — 2026-09-06 — A03 guidance-evaluator evidence repair

- A03 implementation merged in `2e882ae` after independent review and a fully green hosted retry. Its required partial evaluator run exposed the remaining honest gap: Contract v1 named `guidance` but had no assertion type, so it reported `unavailable` rather than a false pass.
- A focused evaluator repair now adds optional fixture-owned textual guidance assertions, with passing, missing-fragment, and unsafe-path tests. The evaluator suite passed 30 tests and Python compilation passed. The exact A03 binary-backed disposable proof is pending: local storage had only 118 MiB free and Cargo failed with `No space left on device`; the local `vps-dev` SSH alias was unavailable. The failed temporary build and fixture were removed, restoring 128 MiB, still inadequate. This is an environmental evidence gap, not a passing result.
- Context health: the repeated issue is constrained local disk, already visible in prior A01 docs observations. No new reusable skill is warranted; next is an adequately provisioned runner for the exact current-master binary proof, then independent review and hosted gating of this repair.

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

## Iteration 4 — 2026-09-06 — R01 hosted regression repair

- The first current-master R05 matrix exposed a real macOS watch-test regression: pathless filesystem rescan events produced the intentionally safe `full_rescan_event` report instead of the formerly expected silence.
- R01 narrowed only its test contract to accept that exact safe fallback while rejecting other output; production overflow behavior remains unchanged. Independent review found the repair clean.

## Iteration 5 — 2026-09-06 — R01 current-master integration

- PR #155 (`ca3d4efb1ca3366bb157074d81601693347eb723`) passed the full hosted matrix, including macOS, Ubuntu, Windows, installer/adoption, release, evidence, and performance checks.
- Merged as `fc5dd2214483b463be6c8a0b6823adf810b36388`, then verified reachable from `origin/master`. The card remains awaiting a separate post-merge evidence/state closure.

## Iteration 6 — 2026-09-06 — Active repair context health

- Q03's hosted Windows failure is a test-only extended-path formatting mismatch; its correction is under independent review and must honestly dispose of the Q03-introduced loader-size advisory before it can proceed.
- R03 is independently reproducing the many-scope release comparison. It has preserved the failing hosted rows and is investigating launcher/scope-plan attribution; the evaluator and threshold remain unchanged. `vps-dev` is currently unavailable, so it is not counted as verification.
- Context health: current-master integration and independent-review gates are functioning. No new reusable skill is warranted; the existing performance and structure-fit skills cover the active decisions. The next action is Q03 repair review/hosted proof, then R03's evidence-backed outcome; R05 waits for both its current-master refresh and the performance gate.

## Iteration 7 — 2026-09-06 — Q03 integration and R03 hypothesis rejection

- Q03's Windows path-contract correction and loader-size remediation passed independent review and the complete hosted matrix, then merged as `7a06b345d47521ede6b5e6c7cdc06e1128883774`.
- R03 rejected, rather than merged, a locally favorable HashSet experiment because stronger same-fixture VPS public-command evidence records a regression. The experiment was reverted with no policy/evaluator change.
- Next: merge this evidence closure after review; then refresh R05 onto current master and select only the next dependency-ready card. The exact R03 protocol remains required for a genuinely distinct hypothesis.

## Iteration 8 — 2026-09-06 — R05 installer integration

- R05's first current-master matrix exposed a real Alpine fixture collision; the archive installer itself had completed. The repair retained dedicated Alpine musl coverage under `/tmp` and removed only the duplicate generic collision path.
- Independent review also found and repaired a pair-preservation hole during the second backup move. Unix and hosted Windows controls now prove both old companion binaries survive that injected failure.
- PR #153 merged as `2ee15e42c5b3bfdfcaf8c2ba8a2aa8f789c78356` after its full hosted artifact matrix passed. Next ready cards are Q02, A01, W01, and F01; choose one behavior surface at a time after this evidence closure is reviewed.

## Iteration 9 — 2026-09-06 — W01 CTA contract and context health

- W01 started in isolated current-master worktree `assura-w01-marketing-ctas` at `09310aa`. A route-wide Playwright fragment contract first failed on the real `/compare/ls-lint/#onboard` dead destination, then passed after the shared focused-page destination was corrected to `/#onboard`.
- The complete marketing suite passed 105 tests. `cargo xtask docs`, repository structure check, and evidence gates passed. Manual keyboard checks confirmed both desktop and 375px About-page Start actions invoke the setup dialog while retaining the real no-JavaScript homepage target.
- Context health: the only repeated execution friction was an overlapping cold Cargo docs invocation; the active first invocation was allowed to finish and the duplicate waiter then completed, with no changed-command retry or output claim. Existing isolation, test-first, browser, and evidence workflows are sufficient; no new skill is justified.
- Next: independent review of the exact W01 candidate, then hosted PR proof. Q02 remains separately in its final hosted matrix and must not be merged until every required job is terminal green.

## Iteration 10 — 2026-09-06 — W01 current-master closure

- PR #159 merged as `8eaa31434528ba2a877284929723fba708d944b3` after independent review and all targeted hosted website/evidence/scope/security/external-build checks passed. The skipped Rust, release, and performance jobs were outside the website-only scope and are not recorded as passing.
- Fetch confirmed the merge SHA is reachable from `origin/master`; W01 is now done. No deployment, release, tag, or public communication was initiated.
- Next: rebase the independently prepared F01 kit onto current master before review, while Q02 remains held by the honest R03 performance gate failure.

## Iteration 11 — 2026-09-06 — F01 pilot-kit preparation

- F01's interview kit was prepared in isolated worktree `assura-f01-pilot-kit`, independently reviewed, and corrected to exclude re-identification data from repository evidence. The consented, Nick-controlled follow-up boundary is explicit; no external outreach or participant selection occurred.
- The rebased candidate `877acebb2e8ef53cfb638acd14c290f46e2e43fc` passed docs, structural, and evidence gates locally against `9b032d3`. F01 remains blocked on Nick's explicit authorization to select and invite participants; that external authority was not inferred.
- Next: independent final review of the rebased exact SHA, then documentation-only PR evidence. Q02 remains blocked by R03's honest hosted performance regression.

## Iteration 15 — 2026-09-06 — A01 evaluator contract and context health

- A01 is active in isolated worktree `assura-a01-init-evaluator` from current master `80f42e9`. Test-first evaluator contracts now cover permissive false-green negative probes, partial-scope ineligibility, preservation hashes, unavailable native commands, structural paths, and required positive probes.
- Context level: not exposed. Relevant working facts are: (1) A01 owns a Python stdlib evaluator and fixtures, not agent orchestration; (2) each covered contract was observed RED before its minimal implementation; (3) Q02 remains held by the honest R03 performance failure; (4) F01 is merged preparation but blocked on Nick's outreach authority; (5) no release, deployment, tag, or public communication authority has been used.
- The repeated evaluator contract pattern belongs in A01's tests and documentation rather than a reusable skill: it has not yet been rediscovered outside this one card. Continue with fixture-backed hook/zero-test/wrong-cwd contracts before broader verification and review.

## Iteration 18 — 2026-09-06 — A01 fixture freeze and context health

- A01 now has a Python-stdlib evaluator contract slice, frozen Rust/TypeScript/Python baselines, tracked existing-config and hook-manager variations, and a boundary document. Ten focused tests cover false-green, partial, structure, preservation, positive/negative policy, native unavailable/zero-test, hook-path, and schema rejection paths.
- Context level: not exposed. Repeated friction was limited to the isolated website build's missing dependencies and Git ignoring `.githooks` fixture content. The first is recorded as unavailable rather than passed; the second was corrected to tracked hook-manager metadata and the ignored goal-created directories were removed. No reusable skill is warranted because neither procedure has recurred outside this card.
- Next: complete known-good, wrong-cwd, idempotence, and publication-redaction contracts; validate frozen fixture hashes and real candidate binary behavior. A01 remains active and unreviewed.

## Iteration 21 — 2026-09-06 — A01 provenance and context health

- The evaluator now records contract and candidate-binary SHA-256 hashes, rejects unsupported contract schemas, proves a known-good full contract, and reports a missing declared cwd as unavailable evidence. The focused suite has 12 passing tests.
- Context level: not exposed. The broader `cargo xtask fast` cold build was allowed to finish but its terminal output was not captured; it is explicitly inconclusive. Its 2.3 GiB ignored worktree `target/` cache was then removed with a Git-scoped cleanup after verification, restoring disk headroom without touching source or other worktrees.
- Repeated lessons remain card-local: fixture provenance and generated-cache recovery are already covered by evidence discipline and safe exact-target cleanup. No new reusable skill is justified. Next: implement remaining A01 idempotence/publication-redaction contracts and real candidate-binary fixture evaluation before independent review.

## Iteration 24 — 2026-09-06 — A01 three-stack controls and context health

- Frozen Rust, TypeScript/Bun, and Python contracts each now complete a real full evaluator control using the identified installed Assura 0.4.0 binary. Each preserves declared source hashes, passes a positive policy check, runs the stack-native test command, and rejects a seeded naming violation in a disposable copy.
- Context level: not exposed. The three controls exposed duplicate YAML keys in newly authored fixture policies; each failure was retained as RED evidence and repaired without changing evaluator acceptance rules. The previous fast-tier run remains inconclusive and is not reclassified.
- No reusable skill is justified: the repeated issue was a one-card fixture-authoring pattern now covered by the evaluator tests and durable contracts. A01 remains active pending lifecycle/idempotence, publication-redaction, broader repository gates, and independent review.

## Iteration 25 — 2026-09-06 — A01 publication-safe evidence

- A focused red test proved the evaluator lacked a publication artifact. The new optional `--public-output` writes a separate aggregate result that omits fixture and command identifiers, hashes, paths, cwd values, stdout, and stderr; the private `--output` record retains diagnostics.
- The redaction test passes with a synthetic command-output secret present only in the private artifact. The full focused evaluator suite passed 13 tests, and `git diff --check` passed. A01 remains active: idempotence and real lifecycle proof are still required, and no broader or hosted gate is claimed from this iteration.

## Iteration 26 — 2026-09-06 — A01 dimension-completeness correction and context health

- A real Rust control exposed a false completion signal: empty guidance, hook, and native assertions were reported as pass. A focused red test now requires every requested but uncontracted dimension to be `unavailable`; the evaluator records per-dimension pass/fail/unavailable states and unavailable evidence is a critical failure. The focused suite passes 15 tests.
- The exact current Rust control now exits 1 with policy/structure/preservation/idempotence pass and guidance/hooks/native unavailable. Earlier TypeScript/Python acceptance-pass observations are retained as historical pre-correction controls and must be rerun; they are not current acceptance evidence.
- Context health: not exposed. The key repeated failure was empty-contract evidence being mistaken for a passing dimension; this is now encoded in the evaluator, not a new skill. A01 stays active; next is completing honest contract evidence for missing dimensions and coordinating actual hook lifecycle proof with A04.

## Iteration 27 — 2026-09-06 — A01 timeout evidence and context health

- A real focused subprocess test first showed that a timed-out policy probe raised instead of emitting evidence. The evaluator now applies a 30-second timeout to policy and native commands, records timeout as failure with captured private partial output, and refuses to treat a timed-out negative probe as policy rejection. The focused suite passes 16 tests.
- Context health: not exposed. Two related evaluator gaps (empty dimension evidence and unhandled timeout) were resolved as compact, reusable contract behavior inside the card; no cross-card operational workflow was rediscovered, so no skill is warranted. A01 remains active pending a review of its remaining contract boundary, broader validation, and independent review.

## Iteration 28 — 2026-09-06 — A01 independent review repairs

- Independent review of `2d950a1` found three high-severity false-green risks: sequential negative mutations contaminated one another, accepted negative probe IDs could leave policy state as pass, and every full run lacks contractable guidance evidence. It also found malformed-contract/path escape and incomplete zero-test detection gaps.
- Focused RED/green tests now prove fresh disposable copies for each negative probe, policy failure mapping, named missing-field errors, cwd escape rejection, and pytest zero-item detection. The focused suite passes 20 tests. The guidance issue remains an intentional unavailable boundary because Contract v1 provides no guidance assertion; it must not be hidden or treated as A01 end-to-end acceptance.
- Next: commit the repairs, obtain re-review of the new SHA, and then decide whether A01's evaluator-only acceptance is sufficiently evidenced or needs an explicit contract-version decision before broader validation.

## Iteration 29 — 2026-09-06 — A01 re-review crash repair

- Re-review confirmed the prior concrete defects were fixed but found one remaining medium error path: a nonexistent relative policy cwd crashed before writing evidence. The focused RED test now passes with an `unavailable` command record and output artifact; the evaluator suite passes 21 tests.
- Guidance remains structurally unavailable in Contract v1 because the packet names the dimension but defines no executable guidance assertion. This is retained as an explicit non-acceptance boundary, not patched around by inventing an unreviewed schema extension. Next: commit this repair and use the final exact SHA for any broader-gate/review decision.

## Iteration 30 — 2026-09-06 — A01 broader local evidence and context health

- Final independent re-review found no new defects in `ba2d865`; it confirmed unavailable policy cwd/executable evidence is non-crashing and the earlier mutation-isolation, policy-state, contract-validation, and zero-test repairs remain intact. `cargo xtask evidence`, Python compilation, the 21-test focused suite, and a final Assura structure check with zero blocking violations passed.
- `cargo xtask docs` exceeded its observation window, then had no child process while available disk fell from 2.3 GiB to 1.1 GiB. It exited before its terminal result was observable, so it is recorded as inconclusive and was not retried. Context health: not exposed. Repeated low-disk build observation is covered by existing evidence discipline; no new skill is warranted. Next: resolve the explicit Contract v1 guidance-evidence decision before calling A01 verified or preparing a PR.

## Iteration 31 — 2026-09-06 — A01 scope reconciliation

- The planning review resolves the Contract v1 concern: A01 supports partial evaluator dimensions; A03–A05 own guidance/hook/native closure and A07 owns full acceptance. Unavailable dimensions therefore remain visible and non-accepting, while A01's card-level acceptance is its independently reviewed ability to catch false-green policy, preservation, hook, and native failures.
- A01 is now `verified` in the backlog with two independent review passes, 21 focused tests, evidence gate, compilation, and zero-blocking structure proof. The docs gate remains inconclusive under low disk and must be decided by hosted PR evidence; A01 is not done until its reviewed PR is merged and reachable from master.

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

## Iteration 21 — 2026-09-06 — Q07 cache recovery attribution repair

- Independent review found that the original corrupt-cache test added a naming
  violation before its recovery assertion, allowing snapshot invalidation to
  explain the fresh result. The revised test repeats against an unchanged
  project, asserts a successful fresh validation and a valid rewritten cache
  record, then separately checks the later naming violation.
- Context-health review: the known Trellis `cargo xtask pr` active-task routing
  discrepancy remains documented rather than waived; Q07 will retain its
  nonzero result if it recurs. The nested launcher package test is bounded and
  useful; no reusable-skill gap is evident.

## Iteration 20 — 2026-09-06 — Q07 launcher error contract

- Q07 started in isolated current-master worktree `goal/q07-error-contract` at `c9ad106`. A focused red test proved the primary launcher silently treated a present, non-executable companion as absent because spawn errors were discarded with `.ok()?`.
- The smallest repair now propagates a path-bearing OS error and runtime exit `1`; the exact focused test passed. The first `--exact` invocation selected zero tests because its nested test path was incomplete and is explicitly excluded from proof. Remaining Q07 work is the report-output failure contract, selected cache fallback audit, packaged-launcher coverage, and required integration/release gates.

## Iteration 35 — 2026-09-06 — R01 hosted watch-regression refinement

- PR #173's first hosted matrix disproved the initial external-config rescan filter: macOS coalesced unrelated external-config activity with Assura runtime-output paths, emitting a false full rescan in both explicit-config and requested-directory watch contracts. The failure is preserved in R01 evidence; no gate was waived and no merge occurred.
- A new focused mixed-event unit test was RED before the correction and is green after it. The refinement ignores only rescan events made entirely of external/config-sibling and Assura-runtime-output paths; excluded in-scope paths and pathless rescans remain observable safety fallbacks. Independent review added a direct pathless-rescan regression; all 16 `cli::watch` units and 14 watch integration tests pass locally.
- Context health: the only repeat was platform-specific watcher event coalescing, now captured by a compact unit contract beside the classifier. Existing watch evidence and test patterns are sufficient; no new skill is warranted. Next: independent review of the new SHA, then a fresh hosted matrix.

## Iteration 36 — 2026-09-06 — R01 requested-scope sequence correction

- The corrected R01 hosted matrix passed Linux but macOS exposed a remaining test-sequencing defect: a helper intentionally accepts a successful, pathless full-rescan event for an out-of-scope FSEvent, then the test incorrectly expected the subsequent in-scope mutation to retain sequence 2. The helper now returns the next expected sequence (2 when quiet, 3 after the accepted safety fallback); it does not weaken either event’s content contract.
- The focused requested-directory test, complete 14-test watch integration suite, 16 `cli::watch` units, formatting, diff, and structure gates pass locally. Next: independent review of this exact test-contract repair, then a fresh hosted matrix.

## Iteration 37 — 2026-09-06 — R01 external-config rescan integration

- PR #173 merged the independently reviewed external-config rescan repair as `b3002b1`; the exact commit was fetched and verified reachable from `origin/master`. Its full hosted matrix passed, including macOS/Windows/Linux suites, MSRV, performance, release/installer/adoption, documentation, security, coverage, and evidence gates.
- The correction keeps watcher safety explicit: unrelated external config and Assura runtime-output noise no longer produces false rescans, while pathless and excluded in-scope rescans remain observable fallbacks. No tag, release, deployment, or public action was taken. Next independent ready card: A02.

## Iteration 38 — 2026-09-06 — A02 explicit local-policy preflight

- A02 is active in isolated current-master worktree `goal/a02-local-patterns`. Focused VPS integration proof covers `init --recipe-file` with a spaced path, SHA-256 provenance in `.assura/onboarding/profile-selection.json`, successful `agent onboard --recipe-file`, and a conflicting local rule with path/existing/incoming diagnostics.
- A new red contract showed onboarding was materializing its baseline before detecting a local-policy conflict, changing project config despite the conflict. The implementation now preflights a local recipe against an existing config before baseline materialization; the four-test focused suite is green and the local Assura structure gate has zero violations. This is not card completion: invalid-result, idempotence, bundled fixture, documentation, broader-gate, and review evidence remain.
- Context health: local disk recovered to 1.7 GiB; VPS remains the compile/test authority and has ample capacity. The only repeated operational friction is explicit source synchronization between the isolated local worktree and the VPS clone; existing evidence commands remain sufficient and no new reusable skill is warranted. Next: add the remaining A02 merge and idempotence contracts.

## Iteration 39 — 2026-09-06 — A03 closure bookkeeping correction

- Post-merge verification of PR #179 (`929fbff`) found its evidence text correctly closed A03, but its backlog edit had matched Q02's repeated status fields instead of A03. Q02 also has no referenced evidence file, so its accidental `done` status was not supportable.
- Dedicated documentation-only PR #180 changes exactly those two records: A03 is `done` with its merged evidence, and Q02 is returned to unclaimed `pending`. JSON validation, the installed `assura check --format agent .`, and whitespace validation pass. No product behavior or CI scope changed.
- Context health: not exposed. This is the first exact-ID bookkeeping mismatch in the train; the immediate correction and card-ID-anchored patch are sufficient, so no reusable skill is justified. Next ready cards after PR #180 merges: A04 and A05, each dependent on A03.
