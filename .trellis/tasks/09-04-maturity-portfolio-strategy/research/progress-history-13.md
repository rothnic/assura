# Preserved progress history: iterations 32–33

These older entries are retained here after the active progress log reached
its configured 1,000-line limit. They are historical evidence, not current
routing; always refresh the ledger and source pointer before acting.

## Iteration 42 — 2026-09-06 — Acceptance correction and queue reconciliation

- The preceding review was progress: it found concrete A04 contract violations
  on master `6d613f8`, changing the next action from A05 expansion to A04 repair.
  A04 is reopened; its prior merge evidence is retained with explicit supersession.
- Q02/#158, Q04/#164 and Q06/#166 are implemented but unmerged, not unowned pending.
  Current failed/canceled checks are recorded in their evidence. A05's six dirty
  paths remain untouched in its owned worktree; partial implementation is not proof.
- Closed obsolete goal-owned PR #169 after comparing its test-only relaxation
  with merged #173 (`b3002b1`, ancestry command exit 0). Current master retains
  the stricter no-event assertion and fixes external rescan filtering itself.
  The closed PR and its unmerged branch retain historical evidence.
- Inventory: 52 worktrees before this repair, plus its one new isolated worktree;
  32 prior goal branches. Clean/dirty state and master ancestry were checked.
  Prune dry-run found the same two pre-existing missing-path records; neither
  was pruned. Build-cache ownership is checked before any goal-owned cleanup.
- Context health: context budget is not exposed. Known state is now routed by
  the correction plan, evidence and exact-ID queue records. Repeated observation
  timeout confusion requires retaining tool session IDs and terminal exits;
  no duplicate Cargo run should be started against a live session.
- Reusable workflow review: use the existing local-build guidance for platform
  recovery and the correction plan for this acceptance repair. Session handling
  is a general orchestration rule, not another Assura-specific skill.
- Next: independently reviewed ownership repair, then effective hook manager
  integration and host evidence; R03 remains an independent attribution task.

### Iteration 42 execution updates

- Closed #169 without merging its obsolete test relaxation; its unmerged branch
  remains. Removed six clean, merged goal-owned documentation worktrees and
  local branches: a01-postmerge-record (`0cf2eb0`), a03-postmerge-closure
  (`3c1b92c`), p01-postmerge-closure (`812aa35`), q07-postmerge-record
  (`d0454a3`), r04-postmerge-closure (`9a16ba5`), w01-postmerge-closure
  (`35d7628`). Each cleanup rechecked ancestry and exited 0; committed files
  remain recoverable from Git. Inventory is now 47 worktrees. Unknown paths,
  the two old prunable entries, A05, and the worker's A03 build cache are untouched.
- The generic SDD scratch path failed Assura's root policy (7 violations).
  Moved only this goal's scratch to `.trellis/.runtime/a04-orchestration`.
  No config policy changed; the subsequent structure check exited 0 with zero
  violations. Ruling: reuse the repository's runtime area, not a new root
  exclusion; cost if wrong is a scratch-pointer update, not relaxed policy.
- R03 runner availability was rechecked using current compiler processes and
  sampled CPU use, not process names in stale evidence. Its isolated immutable
  master diagnostic build is now live; the default nightly compiler and shared
  services prevent claiming hosted stable-toolchain parity. See R03 evidence.
- The three still-present remote documentation branches (A01/A03/Q07 above)
  were deleted with exact expected-SHA leases after ancestry verification;
  pre-push validation and deletion exited 0. The other three were already absent.
  A separate current-master A03 target-state repair worktree brings the inventory
  to 48. The verifier still required retired questionnaire text, independently
  reproduced on unchanged master; correcting its owning contract is required,
  not a waiver of `cargo xtask pr`.
- Ownership review found arbitrary-byte custom hooks and Unix project paths
  were not safely preserved/resolved. The file-content repair is committed;
  raw path invocation and deterministic rollback checks remain under correction.
  A04 stays active after this slice: effective hook paths and host evidence are
  still required. One worker bypassed the commit hook during cache contention;
  this was explicitly rejected as a procedure, recorded, and does not waive any
  final gate or approval requirement.
- A03 fast checks completed with exit 0 before cloning the idle Cargo cache
  into A04's own target. The clone exposed stale shared fingerprints, so the
  first A04 run is invalid evidence; only its local Assura package artifacts
  are being cleaned before the actual RED run. Cache separation avoids lock
  contention but does not prove source/binary identity.
- The immutable master VPS baseline also passed with the exact hosted Rust
  1.98.1 compiler: eight accepted no-slower rows, independent native gate, and
  all five warm p95 rows. All command exits and hashes are in R03 evidence.
  This is not a source optimization or proof that historical failures were noise.

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
