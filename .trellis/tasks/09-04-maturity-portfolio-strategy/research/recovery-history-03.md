# Preserved recovery history

This file retains the historical post-merge reconciliation and iteration-label
correction moved from the active recovery index to keep its configured line
limit intact. Do not route live work from this file.

## Post-merge reconciliation — 2026-09-10

- PR #238 (`docs/current-master-pointer-refresh`) merged as
  `edec906f6fa568878a1200f10ecbf3313c129156` from reviewed final head
  `805b324c99829d5fae97f1f975f2c47cc681161a`, with base
  `8cabc53658530a239c00a0c55cbae9b050ad74ca`. The independent process review
  and scoped rereview both returned `ready within reviewed scope — PASS` with
  no findings. The exact reviewed head is now reachable from `origin/master`.
- Exact-head hosted proof passed Documentation Scope, CI Scope, Security Scope,
  Evidence Gates, and GitGuardian. Product/Rust/performance/release jobs were
  explicitly scope-skipped and were not counted as passes. The PR changed only
  process documentation and task metadata; no product, evaluator, private
  fixture, threshold, deployment, release, publication, or invitation action
  occurred.
- The clean owned worktree `/private/tmp/assura-current-master-pointer-refresh`
  and local/remote branch `docs/current-master-pointer-refresh` were removed
  only after clean status and ancestry checks. No goal-owned uncommitted or
  abandoned branch remains from this slice. The task branch entry is retained
  as historical provenance.
- A post-merge fetch observed `origin/master=edec906`. The `8cabc536` pointer
  in the reconciled plan is therefore a timestamped pre-merge snapshot, not a
  permanent baseline; the next continuation must fetch and resolve the ledger
  again before any A07 candidate build or screening decision. A07 remains
  `active` with zero screening, holdout, or final-acceptance credit. Its next
  owner/action is still: define and privately evidence exactly two product-input
  conditions and six holdouts, obtain isolated protocol-review `PASS`, then run
  a fresh candidate-bound canary against the newly refreshed master. No private
  values or canary result are present in this public evidence.
- Before handoff, topology report and strict both exited `128` only because the
  preserved root unknown file and pre-existing missing-gitdir registration
  remain. `git worktree prune --dry-run -v` reported the three existing stale
  registrations; no prune or unrelated cleanup was performed. This is an
  explicit coverage limitation, not a passing strict audit.

### Iteration-label correction verification

- The independent review of the first post-merge candidate identified one
  concrete P2 bookkeeping finding: the post-merge reconciliation was labeled
  iteration 91 even though the proof-record delta already used that number.
  The finding was accepted and corrected to iteration 92 in commit
  `45a961ba0628a082c990c664ac9698d8f36a8bd8` on the current
  `origin/master=edec906f6fa568878a1200f10ecbf3313c129156` base.
- After the correction, the workflow gate reported `Ready: yes`; the source
  check returned `success: true` with the same six unchanged low-severity
  max-line advisories; `cargo xtask evidence`, `cargo xtask target-state`,
  `cargo fmt --all -- --check`, `git diff --check`, JSON parsing,
  `assura check --format agent --agent codex`, and the evidence-only CI scope
  classifier all exited `0`. The classifier reported
  `evidence=true`, Rust/release/performance/rustdoc/website/security=false,
  `changed_count=2`; no product or performance gate was silently skipped.
- The identical `cargo xtask docs` gate passed after the already-recorded
  frozen website dependency bootstrap and built 48 pages. These results cover
  the accepted correction tree; the final proof-record candidate must rerun
  the same affected gates before hosted submission and scoped rereview.
