# Preserved progress history — A01 iterations 15–31

These early A01 evaluator entries were moved out of the compact progress index
while preserving their exact evidence and historical acceptance boundaries.

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
