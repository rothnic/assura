# Recovery process verification

Date: 2026-09-10. Scope: process artifacts, agent instructions and validation
routing only. Product acceptance is unchanged; no card is promoted by this file.
The historical proof below is retained; the current reconciliation and continuation
route are recorded before the historical next-phase note.
Process iteration: 92 (post-merge reconciliation; after the highest recorded
historical iteration, 88; PR #236 closure was iteration 89, the pointer
candidate review was iteration 90, and the proof-record delta was iteration
91).
Context level: not exposed. The before/after phase record is this evidence
file, linked from the full historical progress log.

## Candidate and proof

- Base: `da773bce8315cbb9e69941f0fe89d0a91e42829d`.
- Process implementation: `ceb3029009cc573f9cd6a37c84a8270193c7111b`.
- Checkout: `/private/tmp/assura-execution-recovery-plan`.
- Branch/owner: `goal/execution-recovery-plan`, this thread's process coordinator.
- `git diff --check`: exit 0.
- `cargo xtask evidence`: exit 0; CI scope controls and evidence policy passed.
- `cargo run --quiet -- check --format json .`: exit 0, success true; four
  existing low-severity line-count advisories in unchanged Rust/test files.
  No new advisory in changed artifacts. Source output retained locally in
  `/private/tmp/assura-recovery-structure.json`.
- `target/debug/xtask target-state`: exit 0 using the source-built xtask.
- Workflow gate with canonical task: exit 0, Ready yes, clean owned checkout.
- `scripts/ci-scope.sh --base origin/master --head HEAD`: exit 0;
  evidence=true, Rust/release/performance/rustdoc/website/security=false.
  Classifier and workflow configuration are unchanged.
- `git diff --quiet origin/master HEAD -- src tests Cargo.toml Cargo.lock
  xtask .github/workflows scripts .assura/config.yml`: exit 0; those product,
  policy and CI surfaces are unchanged. Full Rust/platform/performance reruns
  are not claimed for this process-only diff.

AGENTS shrank from 296 to 93 lines; its managed Trellis block is retained.
The stale hardcoded minimum Rust version was replaced by the manifest/CI
source route. Goal execution now routes Trellis tasks and layered references.

## Independent audit

Read-only agent `recovery_audit` identified five concrete process issues at the
base: broad `fast`/`pr` duplication, catch-all environment recovery, cwd drift in
the old recipe, unsuitable dirty-diff VPS transport, and evidence-policy checks
mistaken for outcome proof. All five were accepted and corrected in the skills
and recovery plan. The audit found existing hosted proof-aware scope reuse;
it remains unchanged. It did not establish external provider deployment rules.

Independent review of `ceb3029` accepted the recovery sequence and exercised
seven decisions: current-base state supersedes stale checkout; failed required
performance holds merge; live timeout resumes its handle; process scope does
not authorize product edits; VPS capacity/toolchain must be checked; unknown
dirt is preserved during owned branch closure; passing slice does not close
failing A07. These are instruction simulations, not operational outcome proof.

Three medium findings were accepted: P1 restored unconditional `Ready: no`
routing; P2 qualified remote compiler/Cargo identity with the selected
toolchain; P3 retained `cargo xtask docs` for any docs/website path instead of
narrowing it to build inputs. Scoped rereview remains required after these
corrections. No hosted-green claim is made before that run completes.

Scoped rereview at `e60c7b120226de6e05e2940e1f24b6bb3f821bfb` returned
ready within process scope, with P1/P2/P3 resolved and no remaining findings.
The final source-built evidence, target-state and structure sequence exited 0;
structure retained the same four advisories. This paragraph records those
results as an evidence-only delta. Hosted proof and merge/closure are recorded
on the resulting PR and must be verified before final handoff.

## Topology and remote observations

Global `audit-topology.sh --strict`: exit 128 on pre-existing missing worktree
registration. This is not a passing strict gate. Read-only fallback inspected
all 14 registered goal worktrees: clean, with reachability classified against
origin/master. Eight pre-existing branches plus this process branch are
unmerged; five point to commits reachable from master. These are reachability
facts, not permission to remove others' work. The root's untracked research
file is untouched. `git worktree prune --dry-run -v` reported three missing
registrations; no prune/removal was performed.

VPS probes succeeded through configured alias `vps` after `vps-dev` resolution
failed. CPU/RAM, disk and toolchain observations are in the recovery plan.
No remote builds, cleanup, runner installation or benchmark speedup is claimed.
The bundle validation procedure is documented, not yet operationally timed.

The historical heartbeat was updated through the app to follow the then-current
scope and revision-aware task evidence; its response reported ACTIVE. A
historical goal-tool read returned no active goal, which is not current runtime
state and must not be used to stop the supported goal. The current continuation
route is recorded below and in `recovery-plan.md`.

## Prior closure — 2026-09-10 (PR #236; superseded)

- PR #236 merged as `35cce811532c793f9446d13b8ef42f6390d370bf` from reviewed
  head `2008f3fe7eecb6806492490511b3a8d6a47c4ab1` on refreshed `origin/master`.
  Its stale-baseline and manifest gate-order findings were resolved by the
  final scoped rereview. Hosted CI Scope, Documentation Scope, Security Scope,
  Evidence Gates and GitGuardian passed; scope-skipped jobs remain explicitly
  skipped and were not counted as passes.
- The merged process artifacts now route A07 through current identity and
  holdout freeze, private manifest validation, isolated protocol-review `PASS`,
  fresh current-master canary, and only then 30-cell screening. The prior
  `77b41fe` canary remains historical/no-credit; A07 has no screening, holdout
  or final-acceptance credit.
- The owned `docs/a07-screening-routing` worktree and branch were clean,
  ancestry-verified and removed. The root unknown file, known missing-gitdir
  registration and other classified user/archive worktrees remain untouched.
- In this fresh recovery worktree, the first `cargo xtask docs` attempt failed
  with exit 1 because `website/node_modules` was absent (`astro: command not
  found`). After the locked `pnpm --dir website install --frozen-lockfile`
  bootstrap, the identical docs gate passed with exit 0 and built 48 pages.
  The initial environment failure is retained as a setup observation, not
  treated as a skipped or passing check.

## Current reconciliation — 2026-09-10

- A fresh read-only reset fetched `origin/master=8cabc53658530a239c00a0c55cbae9b050ad74ca`.
  This is the merge of PR #237 from reviewed head
  `19c5941a8717f05f305032be56080f626fdd06b5`; the prior PR #236 merge
  `35cce811` and its `5329abd` follow-on baseline are now historical.
- The reconciled slice updates only live process pointers and the task's
  execution-branch metadata. It does not alter product source, A07 scoring,
  private conditions, holdout allocation, performance thresholds, or authority
  boundaries. A07 remains active with no screening, holdout, or acceptance
  credit.
- The current owned checkout is `/private/tmp/assura-current-master-pointer-refresh`
  on `docs/current-master-pointer-refresh`, based on `8cabc536`; its owner is
  this process coordinator. The root checkout's untracked
  `research/a04-host-status-doctor-permission-gap.md` remains unknown/user-owned
  and untouched. Existing detached archives and stale registrations remain
  outside this slice's ownership.
- Live routing now points the recovery plan, A07 candidate plan/evidence, and
  continuation prompt at `8cabc536`. Older hashes remain explicitly labeled
  historical. The next authorized A07 action is still private manifest and
  isolated protocol review before a fresh candidate-bound canary; no canary or
  screening allocation is claimed here.
- The committed pointer diff passed the local process tier: workflow gate
  `Ready: yes`; `git diff --check`, JSON parsing, `cargo xtask evidence`,
  `cargo xtask target-state`, `cargo fmt --all -- --check`, and
  `assura check --format agent --agent codex` all exited `0`. The source check
  returned `success: true` with six unchanged low-severity max-line advisories;
  none is in the changed process paths. `scripts/ci-scope.sh --base origin/master
  --head HEAD` reported `evidence=true` and Rust/release/performance/rustdoc/
  website/security false, so no product or performance gate was silently
  skipped.
- The required docs gate first exited `1` because the disposable checkout had
  no `website/node_modules` (`astro: command not found`). The exact frozen
  bootstrap `pnpm --dir website install --frozen-lockfile` exited `0`, and the
  identical `cargo xtask docs` rerun exited `0` after building 48 pages. The
  failed environment precondition is retained; it is not counted as a pass.
- Independent process review of the frozen pointer-reconciliation candidate
  returned `ready within reviewed scope — PASS` with no findings. It verified
  current-base ancestry, consistent `8cabc536` routing, historical/no-credit
  labeling for `35cce81`/`5329abd`/`77b41fe`, unchanged thresholds and authority
  boundaries, and no product/evaluator/private-fixture changes. The review did
  not establish hosted results, post-merge reachability, private manifest
  validity, canary success, or A07 acceptance; scoped rereview is required for
  this proof-record delta before submission.
- Required completion remains: scoped rereview, applicable hosted
  scope/evidence checks on the exact final head, merge, post-merge reachability,
  and clean owned-worktree closure. A strict topology failure caused by
  preserved unknown dirt or stale missing-gitdir registrations is retained as a
  limitation, never represented as green.

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

## Historical next phase (superseded)

Appending a recovery summary initially made `progress.md` exceed its existing
1,000-line limit; the source structure check failed. The correction keeps the
checkpoint here and adds a link in the log's existing heading,
preserving historical entries and the unchanged limit. Rerun the source check
after this correction; the failed attempt is not passing evidence.

Resolve candidate review, rerun affected checks, then submit the reviewed final
candidate for applicable hosted checks. Merge only after those pass. Verify
integration and remove only this process slice's clean owned branch/worktree.
The wider recovery plan remains for the train coordinator to execute.
