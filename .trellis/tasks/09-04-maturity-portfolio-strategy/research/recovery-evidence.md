# Recovery process verification

Date: 2026-09-10. Scope: process artifacts, agent instructions and validation
routing only. Product acceptance is unchanged; no card is promoted by this file.
The historical proof below is retained; the current reconciliation and continuation
route are recorded before the historical next-phase note.
Process iteration: 99 (post-merge continuation reconciliation; iteration 98
was the current-master continuation-control refresh; iteration 97
was the ledger-routing helper pin/path correction; iteration 95 was the
deterministic ledger-routing helper; iteration 94 was the A07
private-manifest readiness audit; iteration 93 was the topology-audit
helper correction; after the highest recorded historical iteration, 88; PR
#236 closure was iteration 89, the pointer candidate review was iteration 90,
the proof-record delta was iteration 91, and post-merge reconciliation was
iteration 92).
Context level: not exposed. The before/after phase record is this evidence
file, linked from the full historical progress log.

## Post-merge continuation reconciliation — 2026-09-10 UTC (PR #244 / 2902a07)

- Owner/phase: process coordinator / `reconcile-handoff`. A fresh fetch from
  the preserved root succeeded and resolved `origin/master` to
  `2902a073f39f8e8a47a9658a6358791c3e7f4655`, the merge of PR #244. The
  reviewed final head `03a9b01eb1a5742b0cc84373e53f3f9aed8d0f1e` is an
  ancestor (`git merge-base --is-ancestor` exited `0`) with base
  `80d2d9a7fea55c1413f7e50f0873306049a65a48`.
- PR #244's exact final hosted result is terminal: Documentation Scope, CI
  Scope, Security Scope, Evidence Gates and GitGuardian passed. Build
  Documentation, Check, Clippy, Code Coverage, MSRV, Performance Report,
  Release Bundle Smoke, Rustfmt, Test Suite, installer/adoption and website
  jobs were scope-skipped by the evidence-only classifier; they are retained
  as non-applicable, never treated as passing product or performance proof.
  The initial clean-checkout documentation precondition and frozen-install
  retry are retained in iteration 98; the identical retry passed and built 48
  pages.
- The owned `/private/tmp/assura-train-continuation-control` worktree was clean
  and removed after merged reachability; its local and remote
  `docs/train-continuation-control` refs were deleted by exact identity. The
  root's unknown A04 research file, the external dirty worktree, three
  prunable registrations, one unreadable registration and historical
  unmerged goal refs remain outside this slice's ownership. No stash, reset,
  prune or broad cleanup was performed.
- The refreshed topology report at this handoff is
  `base=origin/master worktrees=35 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9`; report exited `0` and strict exited `1`.
  `git worktree prune --dry-run -v` listed only the three stale registrations.
  The strict nonzero result is the preserved root/external dirt and historical
  topology outside this process slice, not a reason to erase or claim ownership
  of them.
- The revision-pinned ledger at `2902a07` reports `items=32`,
  `ready_pending=0`, `unfinished=5`, `held=3`: A07 remains active, W03
  remains verified, and R01/W02/F01 retain their named holds. No product card,
  evaluator result, acceptance threshold, release, deployment, publication or
  invitation changed. This is process reconciliation evidence only.
- The next authorized action is A07's private exactly-two-condition manifest
  and six-holdout validation, isolated protocol-review `PASS`, and a fresh
  candidate-bound canary against `2902a07`; the R01 native-readiness/trace,
  W02 Cloudflare approval, F01 participant authorization and W03 publication
  decisions stay with their named owners. The supported runtime goal remains
  active; an empty ready queue is an intermediate route, not completion.

## Reconciliation candidate review — 2026-09-10 UTC (PR #245)

- Independent review returned `PASS` with no findings for exact candidate
  `cb11a22dcdf5a01fdb5cbf8f9f2ce168e76d3a8f` against
  `2902a073f39f8e8a47a9658a6358791c3e7f4655`. The reviewer verified exactly
  seven intended documentation/Trellis/task-metadata paths, clean status and
  ancestry, linked history, JSON/structure/workflow/evidence/target-state/
  docs/format/diff gates, and no product/evaluator/threshold/privacy/
  authority/CI-policy changes.
- PR #245's applicable hosted checks were terminal and passing: Documentation
  Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian. Product,
  Rust, performance, release, installer and website jobs remained explicit
  scope skips and are not product or performance proof. The final review delta
  below must receive a scoped rereview and the same exact-head checks before
  merge.

## Current-master continuation-control refresh — 2026-09-10 UTC

- Owner/phase: process coordinator / `investigate-prepare`. The read-only reset
  ran from the stale root only for inspection; `git fetch origin master` exited
  `0` and resolved `origin/master` to
  `80d2d9a7fea55c1413f7e50f0873306049a65a48` (the PR #243 merge). The clean
  owned worktree for this process slice is based on that SHA; the root's
  unknown `research/a04-host-status-doctor-permission-gap.md` is preserved.
- `bash .agents/skills/assura-goal-execution/scripts/audit-ledger.sh
  <clean-worktree> .trellis/tasks/09-04-maturity-portfolio-strategy` exited
  `0` and loaded both task blobs through the immutable base object. It reported
  `items=32`, `ready_pending=0`, `unfinished=5`, `held=3`; A07 is active,
  W03 verified, R01/W02/F01 held, and no pending row is executable. This is
  routing evidence only; it is not owner-liveness, acceptance, review or merge
  proof. Active/implemented/verified candidates were inspected before pending
  rows as required by the execution contract.
- Release/installation refresh: `git fetch origin release` exited `128` with
  `couldn't find remote ref release`; `git ls-remote --tags origin` reaches
  `v0.3.0`; `command -v assura` resolved `/usr/local/bin/assura` and
  `assura --version` returned `0.4.0`; `Cargo.toml` at the current base also
  declares `0.4.0`. No release branch, tag or installed binary is treated as
  public-install proof.
- Open-PR/CI refresh: PR #194 is open against `master` with head
  `4c3c4981cf5be19e7dc49ac7d090b01d3070871f`; its required Performance Report
  failed after `6m53s` while other applicable jobs passed. PR #195 is open
  against the unmerged #194 branch and has only GitGuardian evidence. PR #187
  is not current and its successful Workers Builds check is coupled to
  Cloudflare production; PR #142 retains Alpine and macOS/Windows failures.
  Scope-skipped jobs are retained as non-applicable, never as passing proof.
- Serialized remote preflight: `ssh -o BatchMode=yes -o ConnectTimeout=8
  vps-dev ...` failed to resolve the host; the same bounded probe to `vps`
  exited `0` and reported 16 CPUs, 43,750 MiB available memory, load below 1,
  23,217,420 KiB free at 94% disk use, nightly Rust 1.95, Node 22.22.1 and
  pnpm 10.29.3. No build, benchmark or test ran remotely. This is capacity
  evidence only; remote work remains conditional on exact toolchain and
  projected target-size headroom, with one job per host and hosted CI as final
  platform/performance proof.
- Topology: `audit-topology.sh --report` exited `0`; `--strict` exited `1`
  with `worktrees=34 dirty=2 prunable=3 unreadable=1 goal_branches=13
  unmerged_goal=9`. `git worktree prune --dry-run -v` was read-only. The root
  unknown file, external dirty worktree, stale registrations and historical
  unmerged goal branches are outside this slice's ownership; no stash, reset,
  prune, deletion or ownership reassignment occurred.
- Review and hosted proof for the initial committed candidate: independent
  process review of `9a5d7a84b0cd7d93664782ef00f365e225b61d7f` against
  `80d2d9a7fea55c1413f7e50f0873306049a65a48` returned `PASS` with no findings.
  PR #244's exact head was `CLEAN/MERGEABLE`; Documentation Scope, CI Scope,
  Security Scope, Evidence Gates and GitGuardian passed. Rust/product,
  performance, release, install and website jobs were scope-skipped and not
  counted as passes. The evidence update below is a documentation delta and
  requires the same scoped rereview and exact-head gates before merge.
- Decision/next action: keep the existing runtime goal active; do not create a
  duplicate or stop at the empty ready set. The A07 acceptance coordinator
  must privately name exactly two product-input conditions and six holdouts,
  validate the complete 30-cell matrix, obtain an isolated protocol-review
  `PASS`, and then run a fresh candidate-bound canary against `80d2d9a`. No
  screening cell, release, deployment, publication or invitation is authorized
  by this process slice. R01's raw macOS trace/native-readiness decision,
  W02's Cloudflare approval, F01's participant authorization and W03's
  publication remain separate held actions; independent process work may
  continue. This entry does not promote any card.

## Ledger-routing helper integration reconciliation — 2026-09-10

- The reviewed process slice from PR #242 is integrated: candidate
  `9651b9ddcd6393472d7bd70a74128c0c24071619` was reviewed against
  `2607709fd218685bb8882e278428bd798f6a6291`, passed the applicable hosted
  gates, and merged as `e296076558418cd7492c0244138e23470a1dc272`.
  Documentation Scope, CI Scope, Security Scope, Evidence Gates, and
  GitGuardian passed. Product/Rust/release/performance/website jobs were
  scope-skipped by the evidence-only classifier and are retained as
  non-applicable, not passing evidence.
- After merge, `origin/master` was refreshed to
  `e296076558418cd7492c0244138e23470a1dc272`; the merged candidate is an
  ancestor. Its owned worktree and local/remote `docs/ledger-routing-helper`
  branch were cleanly removed, with compare-and-delete verification. The
  helper's historical branch name remains in task provenance; no card state
  was promoted by this process slice.
- Running the merged helper from a fresh clean current-master checkout
  reports `items=32`, `ready_pending=0`, `unfinished=5`, `held=3`: A07 is
  still active, R01/W02/F01 retain their named holds, and W03 remains verified
  with publication authority separate. No pending card is executable from
  this revision, and this routing result is not owner-liveness, acceptance,
  review, or merge authority.
- The required final topology report exited `0` and strict exited `1` with
  `base=origin/master`, `worktrees=34`, `dirty=2`, `prunable=3`,
  `unreadable=1`, `goal_branches=13`, `unmerged_goal=9`. The two dirty
  worktrees, unreadable/prunable registrations and historical unmerged goal
  branches are outside this slice's ownership; `git worktree prune --dry-run
  -v` listed the three stale registrations and no prune was performed.
  The root unknown file remains untouched.
- Owner/phase: process coordinator / reconcile-handoff. The only remaining
  authorized continuation is the A07 coordinator's private exactly-two-
  condition manifest and isolated protocol review, followed by a fresh
  current-master canary if that review passes. R01's diagnostic/native-
  readiness decision, W02's Cloudflare publication approval, F01's participant
  authorization, and W03's publication remain separate authority boundaries;
  no deployment, release, publication or invitation action was taken.
- Reconciliation candidate `708a6c30adb906a25767516d79980500e0b25c4f` passed
  the workflow (`Ready: yes`), source JSON (`success=true`), evidence,
  target-state, format, diff, JSON, Assura and CI-scope gates. CI scope was
  `evidence=true`, `changed_count=2`, with product/Rust/release/performance/
  rustdoc/website/security surfaces `false`. Its first docs attempt failed
  only because the fresh checkout had no `website/node_modules` (`astro` was
  unavailable); frozen installation added 355 cached packages and the
  identical docs retry passed with 48 pages. The failed precondition is
  retained and is not passing evidence.

## Ledger-routing helper — 2026-09-10

- The current base is `origin/master=2607709fd218685bb8882e278428bd798f6a6291`.
  This process-only slice adds a read-only queue-routing helper at
  `.agents/skills/assura-goal-execution/scripts/audit-ledger.sh` and a concise
  route to it from the goal-execution skill. It changes no product behavior,
  evaluator input, acceptance threshold, release, deployment, publication,
  invitation or authority surface.
- The helper reads the backlog and declared execution branches from an
  explicit Git revision and emits `BASE`, `TASK`, `CARD`, `READY_PENDING`,
  `UNFINISHED`, `BRANCH` and `SUMMARY` records. It requires every declared
  dependency to resolve to a `done` or `verified` item before reporting a
  pending card as ready; an unknown dependency cannot become ready through an
  empty-state default. The output is routing evidence only: it does not prove
  owner liveness, card acceptance, review, or merge authorization.
- `bash -n` and the current repository run passed. At the current base the
  helper reports `items=32`, `ready_pending=0`, `unfinished=5`, `held=3` and
  routes A07 as active while retaining R01/W02/F01 held actions and W03's
  verified state. A disposable `HEAD` fixture reported one ready pending card,
  one held card, and excluded a pending card whose dependency ID was absent;
  this verifies both positive and missing-dependency controls without touching
  the Assura checkout.
- Owner/phase: process coordinator / investigate-prepare. The owned checkout
  is `/private/tmp/assura-ledger-routing-helper` on
  `docs/ledger-routing-helper`, based on the refreshed master. The exact
  candidate and final gates will be recorded before review; no private
  evaluator or fixture values are copied into repository evidence.
- Candidate `f882d22613e21c3c987cd028f0f82366ee8a9c02` passed the final local
  process/docs tier: workflow `Ready: yes`; source check success `true` with
  six unchanged low max-line advisories; `cargo xtask evidence`,
  `cargo xtask target-state`, `cargo fmt --all -- --check`, `git diff --check`,
  JSON parsing, and `assura check --format agent --agent codex` all exited `0`.
  The committed CI scope classifier reported `evidence=true`,
  `changed_count=4`, with product/Rust/release/performance/rustdoc/website and
  security surfaces `false`.
- The first identical `cargo xtask docs` attempt exited `1` because the clean
  checkout lacked `website/node_modules` and could not resolve `astro`. The
  locked `pnpm --dir website install --frozen-lockfile` bootstrap exited `0`
  with 355 cached packages; the identical docs gate then exited `0` and built
  48 pages. The failed environment precondition remains recorded and is not a
  pass.
- The next authorized action remains the A07 coordinator's private
  exactly-two-condition manifest and isolated protocol review. The helper must
  be used at the next continuation before any pending-card selection; its
  result cannot override that A07 protocol gate or the preserved topology
  ownership exceptions.

## Ledger-routing helper correction — 2026-09-10

- Independent review found two routing-integrity defects in the first helper
  candidate: it resolved and printed a base SHA but loaded task blobs through
  the mutable symbolic ref, and its `awk` parser truncated declared worktree
  paths at spaces. Both findings were accepted as concrete process correctness
  gaps; no product, evaluator, threshold, release, deployment, publication,
  invitation or authority surface is involved.
- The correction now loads `backlog.json` and `task.json` through the resolved
  immutable object ID, and parses `git worktree list --porcelain -z` records by
  complete `worktree ` suffix. This keeps the advertised revision and routing
  records coherent if a ref advances during execution and preserves paths
  exactly for ownership inspection.
- Verification before final hosted review: `bash -n` passed; the live ledger
  still reports `items=32`, `ready_pending=0`, `unfinished=5`, `held=3`; the
  disposable positive/missing-dependency fixture still reports exactly one
  ready pending card and excludes the unknown dependency; and a disposable
  declared branch fixture with a space-containing worktree path reports the
  complete path. These checks are routing controls only and do not establish
  owner liveness, acceptance, review, or merge authority.
- Owner/phase: process coordinator / correction-validate. The same clean
  current-master checkout and one-card ownership are retained. The affected
  process/docs gates and scoped independent rereview must pass before any PR
  submission or merge.
- Correction commit `49dc5d6e9ef5cfe66bf8c0f0d5f3330752b6f523` is based on
  `origin/master=2607709fd218685bb8882e278428bd798f6a6291`. On that exact
  tree, the workflow gate was `Ready: yes`; source check JSON was
  `success=true`; `cargo xtask evidence`, `cargo xtask target-state`,
  `cargo fmt --all -- --check`, `git diff --check`, task JSON parsing,
  `assura check --format agent --agent codex`, and the CI-scope classifier all
  exited `0`. The classifier reported `evidence=true`, `changed_count=4`,
  with product/Rust/release/performance/rustdoc/website/security surfaces
  `false`.
- The exact correction tree's `cargo xtask docs` gate passed and built 48
  pages. The earlier clean-checkout dependency bootstrap failure remains
  recorded above and is not counted as a pass; the frozen install and
  successful rerun are the valid docs evidence.

## Current A07 readiness audit — 2026-09-10

- A fresh read-only reset resolved the integration base to
  `origin/master=e03278e25e1d81d6c5d84beb1bac992ea6059869` after PR #240. The
  prior topology-helper slice is merged and its owned branch/worktree is
  closed; this audit changes no product, evaluator, threshold, release,
  deployment, publication, invitation or authority surface.
- The A07 acceptance coordinator's private evidence store was inspected only
  for readiness metadata. The available records include holdout-construction
  material and historical/no-credit run records, but no discoverable artifact
  that satisfies the required screening manifest's exactly-two condition rows,
  one-variable supplied-input proof, blinded mapping, complete 30-cell matrix,
  and isolated protocol-review `PASS`. Historical run filenames and explicit
  controls are not condition definitions or screening allocation evidence.
  Private values, fixture/contract identities, child transcripts and raw
  evaluator output remain outside this repository and are not copied here.
- This is a concrete held action, not a whole-goal stop: the A07 coordinator
  must create or locate the versioned private manifest, validate the contract
  and complete matrix in a separate isolated protocol review, and record only
  a redacted `PASS` or findings. Until that observation exists, A07 remains
  `active` with zero screening, holdout or final-acceptance credit. The next
  observation after a protocol `PASS` is a fresh candidate-bound canary against
  `e03278e`; no historical canary or run is reused.
- Owner/phase: A07 acceptance coordinator / investigate-prepare. The process
  audit checkout is `/private/tmp/assura-a07-manifest-readiness-audit` on
  `docs/a07-manifest-readiness`, based on the refreshed master. Its only
  intended delta is this redacted task evidence and branch provenance.
- The live topology audit remains intentionally non-green because preserved
  unknown/user-owned dirt, a dirty external worktree, stale registrations and
  unmerged historical goal branches remain outside this slice's ownership.
  The helper now reaches its summary and strict mode remains nonzero; no prune,
  reassignment or deletion is authorized by this audit.
- Candidate `5af50cf1a649330dce514184d3dc1fb1538e188c` passed the docs/process
  validation tier: workflow `Ready: yes`; source check success `true` with the
  same six unchanged low max-line advisories; `cargo xtask evidence`,
  `cargo xtask target-state`, `cargo fmt --all -- --check`, `git diff --check`,
  JSON parsing, and `assura check --format agent --agent codex` all exited `0`.
  The committed CI scope classifier reported `evidence=true`,
  `changed_count=3`, with product/Rust/release/performance/rustdoc/website and
  security surfaces `false`.
- The first identical `cargo xtask docs` attempt exited `1` because the clean
  checkout had no `website/node_modules` and could not resolve `astro`. The
  locked `pnpm --dir website install --frozen-lockfile` bootstrap exited `0`
  with 355 cached packages, and the identical docs gate then exited `0`,
  building 48 pages. The failed environment precondition is retained and is
  not counted as a pass.

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

## Topology-audit helper correction — 2026-09-10

- During this continuation reset, the previous orchestration helper
  `audit-topology.sh` emitted a raw Git exit `128` before its `SUMMARY` when a
  prunable worktree directory still existed but its `.git` metadata was gone.
  That failure was diagnostic-tool fragility, not evidence that the topology
  was clean. The exact old script hash was
  `654f0f826a632804f77d89977eb90a2c8306010e88b766c1739eb934a6eb1483`.
- The owned process correction is installed in
  `/Users/nroth/.codex/skills/assura-orchestration`: the helper now probes Git
  metadata and status explicitly, emits `UNREADABLE_WORKTREE` on failure,
  counts unreadable records in `SUMMARY`, and preserves a nonzero strict
  result. The final script hash is
  `67635d3f48143c5b3cf9196b192d138f50770f8d947cf2df2eb81722f4cb0ce5`; the
  accompanying skill guidance hash is
  `49544ac6b0d6493c49f962b2bfdd90ab674111a24754c6fd10b25c912f22332d`.
  No repository product, evaluator, threshold, release, deployment,
  publication, or invitation surface changed.
- `bash -n` passed. A disposable repository with an external child worktree
  whose `.git` file was removed produced `CLEAN_WORKTREE`,
  `PRUNABLE_REGISTRATION`, `UNREADABLE_WORKTREE`, and a complete
  `SUMMARY base=HEAD worktrees=2 dirty=0 prunable=1 unreadable=1
  goal_branches=0 unmerged_goal=0`; report mode exited `0` and strict mode
  exited `1`. This confirms the helper no longer aborts or misclassifies the
  stale registration as clean.
- The live Assura report now reaches
  `SUMMARY base=origin/master worktrees=34 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9` with report exit `0`; strict exits `1` for
  the preserved root/user dirt, dirty external worktree, stale registrations,
  and unmerged goal branches. No prune, deletion, ownership reassignment, or
  unknown-work mutation was performed. Independent process review returned
  `PASS` with no findings for the exact script and skill hashes above.
- This recovery slice is owned by the process coordinator in
  `/private/tmp/assura-topology-audit-robustness` on branch
  `docs/topology-audit-robustness`, based on live `origin/master=d099253`.
  The next action remains the A07 acceptance coordinator's private
  exactly-two-condition manifest and isolated protocol review; topology
  inventory may continue independently, while R01/W02/F01 retain their named
  external/diagnostic holds.
- The committed evidence candidate `e834dbbf2780e304ca77560c1f2aa39c3c393041`
  passed the process/docs tier: the workflow gate reported `Ready: yes`, the
  source check returned `success: true` with the same six unchanged low
  max-line advisories, and `cargo xtask evidence`, `cargo xtask target-state`,
  `cargo fmt --all -- --check`, `git diff --check`, JSON parsing,
  `assura check --format agent --agent codex`, and
  `scripts/ci-scope.sh --base origin/master --head HEAD` all exited `0`.
  CI scope classified `evidence=true`, `changed_count=2`, with all
  product/Rust/release/performance/rustdoc/website/security surfaces false.
  The first docs attempt exited `1` because `website/node_modules` was absent;
  the frozen `pnpm --dir website install --frozen-lockfile` bootstrap exited
  `0`, and the identical `cargo xtask docs` rerun exited `0` after building 48
  pages. The failed precondition is retained and is not counted as a pass.

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
