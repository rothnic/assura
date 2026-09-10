# Recovery process verification

Date: 2026-09-10. Scope: process artifacts, agent instructions and validation routing only. Product acceptance is unchanged; no card is promoted by this file.
Older tail note: [recovery history 01](recovery-history-01.md) and
[recovery history 02](recovery-history-02.md).
The historical proof below is retained; the current continuation route and
post-merge source reconciliation are maintained in [recovery-plan.md](recovery-plan.md)
and the newest progress entry before these historical notes.
Process iteration: 129 (origin/master=69246d3 current candidate freeze, identity controls and two full-contract no-credit canaries pass; corrected 9df6ae6 binding/manifest/protocol PASS retained as historical metadata-only/no-credit after source/contract advance; iteration 128 preserved in progress-history-16.md; iteration 127 recorded post-merge source reconciliation;
iteration 123 was the corrected continuation goal and current-source rebind;
iteration 122 was the merged A07 process route and current-candidate canary;
iteration 121 was the accepted stale-route finding delta; iteration 120 was the
A07 protocol review PASS at current master; iteration 119
was the A07 private manifest review at current master; iteration 118
was the A07 private-readiness audit at current master; iteration 117
was the post-merge reconciliation for PR #264; iteration 116
was the current-master pointer reconciliation after PR #263, iteration 115
was the PR #262 post-merge reconciliation, iteration 114 was the preserved
A07 route-history correction, and iteration 113 was the PR #259
post-merge reconciliation; iteration 112
was the PR #258 post-merge reconciliation and iteration 111 was the candidate identity freeze after the goal-contract
correction; iteration 110 was the goal-contract correction after the R01
raw-log recovery; iteration 109 is the current-master R01 recovery evidence;
iteration 106 was the post-merge checkpoint for PR #251; the R01
diagnostic preparation is candidate-base evidence;
iteration 104 was the continuation-control route on current master;
iteration 103 was the current source-pointer reconciliation;
iteration 102 was the post-merge source-pointer lifecycle checkpoint;
iteration 101 was the current-master/private-manifest readiness re-audit;
iteration 100 was the post-merge closure for PR #245; iteration 99 was the
post-merge continuation reconciliation; iteration 98 was the current-master
continuation-control refresh; iteration 97 was the post-merge ledger-routing
reconciliation; iteration 96 was the ledger-routing helper pin/path correction;
iteration 95 was the
deterministic ledger-routing helper; iteration 94 was the A07
private-manifest readiness audit; iteration 93 was the topology-audit
helper correction; after the highest recorded historical iteration, 88; PR
#236 closure was iteration 89, the pointer candidate review was iteration 90,
the proof-record delta was iteration 91, and post-merge reconciliation was
iteration 92).
Context level: not exposed. The before/after phase record is this evidence file, linked from the full historical progress log.

## Historical 8be candidate rebind and protocol correction — 2026-09-10 UTC (`origin/master=8be6103`, superseded by current `69246d3`; previously `b7043ab`)

- Owner/phase: process coordinator `/root` / `candidate-bound-canary`. PR #268
  is merged as a process-only documentation slice; A02 is complete and its
  old plain-init handoff finding is historical. The fresh detached 8be6103
  checkout built Assura `0.4.0` with Rust/Cargo `1.94.1` and the login-shell
  identity check matched the fixed executable, version and target.
- One initial source-only fixture attempt used a mismatched preservation
  contract and exited nonzero; it is retained as unfavorable no-credit
  evidence. After correcting the fixture to the contract's expected source,
  two fresh composed Codex initialization runs and full evaluator runs exited
  `0`, all seven dimensions passed and the seeded negative control rejected as
  expected. These are no-credit canaries only.
- The first isolated 8be protocol review found three concrete contract gaps:
  no immutable handle-to-layout freeze record, historical candidate hashes in
  holdout metadata, and a stale 9b review reference. The private packet now
  contains a current candidate freeze, immutable six-handle binding with
  source/contract digests and current provenance, a second read-only
  confirmation, an explicit raw-hook exclusion, and a current manifest review
  reference. The two-condition receipts and 30 reserved cells remain no-credit; the findings are not screening or product results.
- Follow-up packet correction canonicalizes one exact compiler/Cargo identity across freeze, conditions, receipts and all six rows, mirrors six immutable per-handle creation records in the second-readonly check, and records supplemental and primary metadata rereview PASS; this remains no-credit evidence.
- Next owner/action: refresh source/ledger, freeze a new candidate identity and prepare only the separately authorized no-credit screening gate. Private values, mappings, fixtures, raw evaluator output and child transcripts remain outside this record; closure stays `active`.

## Historical current as-of merged-candidate canary — 2026-09-10 UTC (`origin/master=9b410e9`)

- Owner/phase: process coordinator `/root` / `candidate-bound-canary`; the
  clean current-master checkout and exact candidate identity are retained in
  the private harness. The merged source is
  `9b410e936e2afe85f0adc4cd83ef614e35c43d2c`; the release binary is Assura
  `0.4.0` built with Rust/Cargo `1.94.1`.
- The actual login-shell command environment matched the fixed candidate for
  `command -v assura`, version, target path and target SHA. Two fresh
  source-only fixtures used the composed `init --agent codex --activate`
  route followed by one reviewed `--content-template` value each. Both
  initializer runs exited `0`; both full A01 evaluator runs exited `0` with
  `acceptance_eligible: true`, `acceptance_pass: true`, all seven dimensions
  passing and the seeded negative control rejecting as expected.
- These are fresh candidate-bound no-credit canaries only. They do not count
  toward the 30-cell screening matrix, 18-run untouched holdout, follow-up
  feature check or final 10-per-stack threshold. Earlier generic/omitted-policy
  and restrictive-existing-config attempts remain retained as unfavorable
  no-credit evidence and were not retried unchanged.
- The prior private protocol `PASS` remains metadata-only for its reviewed
  older candidate. The next owner/action is to rebind the private manifest rows
  and supplied-input receipts to `9b410e9`, obtain a scoped protocol rereview
  `PASS`, and only then allocate screening cells. Private fixture contents,
  mappings, evaluator output and child transcripts remain outside this record;
  no product, release, deployment, publication or invitation authority changed.
## Historical current as-of private manifest review — 2026-09-10 UTC (`origin/master=af73d8a`)

- Owner/phase: process coordinator `/root` / `review`; clean checkout
  `/private/tmp/assura-a07-manifest-review` on
  `docs/a07-condition-manifest-review`, based on fetched
  `origin/master=af73d8a5004ea8c0d2298202467d1635853434c9`.
- Reset proof: workflow/context routing `43/43 PASS`; ledger
  `items=32`, `ready_pending=0`, `unfinished=5`, `held=3` (A07 active,
  W03 verified, R01/W02/F01 held); no pending card is independently executable.
- The private store has an exactly-two-condition manifest, separate mapping,
  six holdout handles and 30 reserved cells. Final supplied-input receipts
  came from sibling-free parents with byte-identical source-only fixtures;
  both exited `0` and match the frozen candidate. Earlier contaminated or
  non-identical attempts remain private invalid/no-credit evidence.
- Isolated reviewer `/root/a07_protocol_review` inspected only manifest,
  mapping, receipt and matrix metadata and returned redacted `PASS` with no
  findings after the three corrections. Raw output, transcripts and fixture
  contents were excluded. This is only the protocol gate: no screening,
  holdout or acceptance credit or product/authority state changed.
- Next owner/action: refresh source/ledger, freeze a new current-master
  identity and run a fresh candidate-bound no-credit canary. The PASS does not
  verify fixture contents, unseen holdouts, launcher identity, child isolation
  or evaluator outcomes. Closure remains active.
## Historical current as-of private-readiness audit — 2026-09-10 UTC (`origin/master=afcbe967`, superseded by `af73d8a`)

- Owner/phase: process coordinator `/root` / `investigate-prepare`; clean
  owned checkout `/private/tmp/assura-a07-route.WCNcQp` on
  `docs/a07-manifest-readiness-refresh`, based on freshly fetched
  `origin/master=afcbe967fd6f1fcc4fe0d7a606bfa288eb4c8df5`. Candidate: this
  process-only evidence slice (recorded at commit).
- Reset proof: context-routing is `43/43 PASS`; the revision-pinned ledger is
  `items=32`, `ready_pending=0`, `unfinished=5`, `held=3`; A07 is active,
  W03 verified, and R01/W02/F01 remain separately held. No pending card has
  independently executable merged dependencies.
- A bounded metadata-only inspection of the owner-controlled private harness
  found mode `0700`, candidate/contract/holdout/run records, and no discoverable
  versioned two-condition screening manifest or isolated protocol-review
  artifact. Private values, mappings, fixtures, contracts, raw output and child
  transcripts remain outside this repository; no allocation or acceptance credit
  is created.
- Independent impasse/process review by `/root/context_routing_review_fast`
  returned `PASS`: an empty pending queue is an intermediate route, A07 owns
  the next action, and no pending card is independently executable. No concrete
  process finding was returned.
- Next owner/action: the A07 acceptance coordinator privately creates or locates
  and validates the exact-two-condition manifest, supplied-input receipt,
  blinded mapping, complete 30-cell matrix and isolated protocol review `PASS`;
  only then refresh source/ledger and bind a fresh no-credit canary. R01/W02/
  W03/F01 holds remain separate. This slice changes no product/evaluator/
  threshold, allocation, release, deployment, publication or invitation state.
## Historical as-of post-merge continuation checkpoint — 2026-09-10 UTC (`origin/master=6fa806d`, superseded by `afcbe967`)

- Owner/phase: process coordinator / `post-merge-reconcile` / pointer refresh.
  PR #264 merged reviewed process candidate `4cfd917` (based on
  `bcd0386`) as `6fa806d`; candidate/merge tree equality and reachability,
  independent review, and applicable Documentation/CI/Security/Evidence/
  GitGuardian checks are verified. Scope-skipped product/Rust/performance/
  release/installer/website checks remain non-applicable.
- The fresh current-master ledger is `items=32`, `ready_pending=0`,
  `unfinished=5`, `held=3`: A07 active, W03 verified, R01/W02/F01 held.
  Context-routing audit is `43/43 PASS`; no product, evaluator, screening,
  holdout, publication, release, deployment or invitation credit changed.
- The stale pre-PR-264 live-looking pointers in the A07 route, e2e prompt,
  recovery plan, orchestration plan, executor prompt and A07 evidence are
  now marked historical and replaced with this single as-of source pointer;
  the prior PR #263 pointer reconciliation remains preserved as history.
  Every continuation must fetch again and rerun the revision-pinned ledger.
- A07's next action remains the private exactly-two-condition manifest,
  supplied-input receipt, blinded mapping, complete 30-cell matrix and
  isolated protocol-review `PASS`; only after that disposition may a fresh
  candidate identity and no-credit canary run. R01/W02/W03/F01 holds remain
  authority/evidence-bound.

## Historical post-merge continuation checkpoint — 2026-09-10 UTC (`origin/master=373fb01`, superseded by current as-of `6fa806d`)

- Owner/phase: process coordinator / `post-merge-reconcile`. PR #259 merged
  reviewed candidate `ccd3bd2dcfe8ba6177b59ffc16ecc09560558bf8` (based on
  `8c198dcb4c94095e7db1b017908eaadb813cb9e7`) as
  `373fb01ac268285d9b21cd6948862050bd7d8c6e`; the merged tree equals the
  reviewed candidate. Applicable Documentation, CI, Evidence Gates, Security
  Scope and GitGuardian checks passed; scope-skipped product/Rust/performance/
  release/installer/website jobs remain non-applicable.
- The fresh revision-pinned ledger reports 32 items, `ready_pending=0`, five
  unfinished and three held. A07 remains active, W03 verified, and R01/W02/F01
  remain separate held actions. Validation case
  `A07-CONTINUATION-CASE-09` (scenario 9; details in
  [this file's validation record](#orchestration-validation-case-09))
  rejected a whole-goal stop; the next action is A07's private exactly-two-
  condition manifest and isolated protocol review `PASS`.
- The PR #258 `docs/maturity-goal-continuation` and PR #259
  `docs/a07-postmerge-checkpoint` checkouts, branches and remote refs were each
  removed after clean status, `git diff --check`, merged-tree equality and
  reachability proofs. Topology `--report` exited 0; strict remains nonzero
  only for preserved root/user dirt, unrelated dirty work, stale/prunable
  registrations, one unreadable registration and historical branches.
- This phase adds no screening, holdout or acceptance credit. The next
  continuation must fetch and rerun the ledger before binding any source or
  candidate identity, then validate the private manifest/protocol disposition;
  only after a redacted `PASS` may a fresh no-credit canary run.

## Orchestration validation case 09

This read-only decision check maps to scenario 9 in
`assura-orchestration/references/validation-cases.md#scenario-9`. The
independent process-validation agent (`/root/process_validation_case`) found
that a pending-only scan was insufficient and returned “reject the proposed
whole-goal stop.” It changed no files and ran no product or private-evaluator
commands.

At the case revision (`origin/master=c34f917`), the ledger had no ready pending
card, but A07 was active, W03 verified, and R01/W02/F01 retained narrow held
actions. The coordinator revalidated those states at `origin/master=373fb01`.
The action mapping was: continue A07 to its private exactly-two-condition
manifest and isolated protocol review; inspect W03 only for authorized
current-head integration while publication stays separate; and preserve R01,
W02 and F01 evidence/authority boundaries without speculative retries,
deployment coupling or invitations. Ledger owner strings are provenance, so
actual branches, worktrees, PRs, reviews and live handles must be inspected.

Decision: continue the supported runtime goal. After a redacted A07 protocol
`PASS`, refresh source and ledger, freeze a new current-master identity, and
run only the no-credit canary. Recheck live handles and candidate phases at
the next continuation before drawing any conclusion from a pending-only queue.

## Historical post-merge continuation checkpoint — 2026-09-10 UTC (`origin/master=8c198dcb`, superseded by `373fb01`)

- This is the PR #258 post-merge state before PR #259. The pending cleanup
  statement and `8c198dcb` pointer are retained for provenance only; use the
  live section above for execution routing.

## Historical candidate-freeze checkpoint — 2026-09-10 UTC (`origin/master=c34f9178`, superseded by `8c198dcb`)

- Owner/phase: A07 acceptance coordinator / `candidate-freeze`. The refreshed
  source is `c34f917866e45cc122ec07412fa0c630d460f663`; the revision-pinned
  ledger reports 32 items, `ready_pending=0`, five unfinished and three held.
  A07 is active, W03 is verified, and R01/W02/F01 remain separate held
  actions. No pending card is executable ahead of this active lane.
- A clean detached checkout built the exact candidate with Rust/Cargo `1.94.1`
  and released `assura 0.4.0` (exit `0`). The non-symlink binary SHA-256 is
  `95c93052bd1993566d9f8209bdba5625c2b39a19287ed6358d710015d2ac59ff`; a
  minimal `zsh -lic` check matched `command -v`, version and target hash. The
  candidate identity is recorded privately and this observation earns no
  evaluation credit.
- Read-only private reconciliation confirms six valid frozen holdouts and one
  disqualified construction draft. The exactly-two-condition manifest,
  supplied-input receipt, blinded mapping, 30-cell matrix and isolated
  protocol-review `PASS` remain absent; no historical run is promoted or
  reclassified. The held action is concrete: select two values for one
  supported product input, prove receipt, validate the manifest in an isolated
  review, then refresh and run a fresh no-credit canary.
- The configured `vps` endpoint is reachable but not used: 16 CPUs, about
  43 GiB available memory, 94% root-disk use (about 20 GiB free), nightly Rust
  1.95 and no Bun. The documented `vps-dev` alias does not resolve. Local
  exact-toolchain evidence and required hosted/platform gates remain
  authoritative.

## Historical goal-contract checkpoint — 2026-09-10 UTC (`origin/master=a819c0c`, superseded by c34f9178)

- Owner/phase: process coordinator / `goal-contract-reconcile`. A clean
  worktree was created from freshly fetched `origin/master` at
  `a819c0cd2e8e9fdf14a3641cc76f0119ceb9d2bc`; no product checkout or private
  evaluator store was changed.
- The canonical copy-paste goal now routes through the active runtime goal,
  the revision-pinned ledger, active/implemented/verified candidates and
  explicit held-action owners. It records the merged R01 raw-log recovery,
  preserves A07's missing private manifest/protocol `PASS`, and keeps W02
  Cloudflare work, W03 publication and F01 participant outreach separate.
- The recovery-plan header now points to the same as-of source and records
  that the public 2,225-line macOS log confirms `full_rescan_event` but lacks
  raw callback paths/kinds, rescan and config-generation fields. The evidence
  remains negative and bounded; it does not close R01 or authorize a retry.
- The current ledger is 32 items with `ready_pending=0`, five unfinished and
  three held: A07 active, W03 verified, and R01/W02/F01 held. The next real
  action remains private A07 six-holdout/manifest validation and isolated
  protocol review `PASS`, then a fresh no-credit candidate-bound canary.
- Before integration, run the documentation/process validation matrix and an
  independent review of this exact diff. After integration, refresh the
  pointer, ledger and topology; remove only this clean merged worktree/ref.
- Candidate proof: commit `6c61a9e3056b0a372ef91d0b96573c2c9ee40e71` on
  `docs/maturity-goal-continuation`; post-commit workflow `Ready: yes`, clean
  status and `git diff --check` passed. `cargo run --quiet -- check --format
  json .` returned `success=true` with six unchanged low max-line advisories;
  `cargo xtask target-state`, `cargo xtask evidence`, task JSON parsing and
  the progress line-budget check passed.
- The first identical `cargo xtask docs` attempt exited `1` because the clean
  worktree had no `website/node_modules` and `astro` was unavailable. The
  frozen `pnpm --dir website install --frozen-lockfile` exited `0`; the
  identical docs gate then exited `0` and built 48 pages. The environment
  failure is retained as a precondition, not counted as a pass.

## Post-merge process checkpoint — 2026-09-10 UTC (`origin/master=062f6c3`)

- PR #251 merged the reviewed process-only candidate `e777ee7` (based on
  `1bd78cc`) as `062f6c39d15babc8b12299863576a29febda5dd5`; the merged tree
  equals the reviewed candidate. It carries the corrected R01 next-action
  route and the continuation command identity fence.
- The revision-pinned ledger remains 32 items with `ready_pending=0`, five
  unfinished and three held: A07 active, W03 verified, R01/W02/F01 held. No
  product, card, evaluator, threshold, allocation, release, deployment,
  publication or invitation state changed. The next authorized route remains
  A07's private manifest/protocol review; R01 still requires its raw callback
  trace or a maintainer native-readiness decision.
- The owned candidate worktree and branch were removed after clean closure.
  The final topology report exited `0`; strict exited `1` only for preserved
  root/user or external dirt, stale registrations and historical refs. The
  `1bd78cc` diagnostic below is candidate-base evidence; refresh before use.

## Candidate-base diagnostic observation — 2026-09-10 UTC (`origin/master=1bd78cc`; superseded by the post-merge checkpoint)

- Owner/phase: process coordinator / R01 diagnostic preparation. A clean,
  detached checkout at `1bd78cc3705e278d6502637463873de4ad1c2aab` ran the exact
  external-config watch test from its own cwd on Darwin x86_64 with
  Rust/Cargo `1.94.1`. The corrected command used the shared target cache and
  exited `0` after 195 seconds; compilation took 1m54s and the test 1.50s.
- The only event was the expected config-triggered sequence-2 warm-full report
  (`coalesced_events=4`, `changed_paths=[]`, no fallback) with the expected
  `snake_case` violation. No unexpected filesystem event occurred, so the
  historical macOS extra `full_rescan_event` was not reproduced. A prior
  wrong-cwd attempt was discarded as invalid evidence and its clean worktree
  was removed.
- No hosted diagnostic run, retry, filter, threshold, or product edit was
  made. This observation cannot close R01. The next action remains obtaining
  the historical callback paths/kinds/rescan/config-generation trace for
  run34090768850/job101643647551, or a maintainer decision on native readiness;
  speculative debounce/loop changes remain unauthorized.
- The owned diagnostic worktree was clean after the run and removed. The
  current ledger/topology must still be refreshed before the next phase; no
  card state or A07 allocation state changed.
- Scoped documentation/evidence gates on this recovery slice passed: `cargo
  run --quiet -- check --format json .` exited `0` with six pre-existing low
  max-lines advisories; the evidence and target-state xtask checks exited `0`;
  after frozen website dependency installation, `xtask docs` exited `0` and
  built 48 pages; `git diff --check` exited `0`. These checks validate the
  process artifacts only and do not promote R01 or A07.

## Historical continuation checkpoint — 2026-09-10 UTC (`origin/master=2eda17e`; superseded by the candidate-base observation and post-merge checkpoint)

- Owner/phase: process coordinator / `investigate-prepare`; the A07
  acceptance coordinator owns the next card action. A fresh fetch resolved
  `origin/master` to the PR #249 merge
  `2eda17e82d9dab12338805479a33a5774560451f`. Reviewed candidate `5f17f7f`
  was based on `755c28d` and its merged tree matches. This process-only slice
  adds the continuation-control reference and reconciles active source labels;
  no product, evaluator, threshold, allocation, release, deployment,
  publication or invitation state changed.
- Session/live handle: none for A07; the metadata-only audit is complete and
  no initializer, evaluator, CI or review process is running. Worktree/branch:
  private A07 evidence lane / no shared checkout; the process candidate is the
  separate clean `docs/continuation-control-route` branch. Trigger/proof:
  `ready_pending=0` plus the redacted private-manifest metadata audit below;
  the exact next action is isolated manifest/protocol review.
- The revision-pinned ledger reports 32 items, `ready_pending=0`, five
  unfinished and three held: A07 active, W03 verified, R01/W02/F01 held. No
  pending card is executable. A redacted metadata audit still finds seven
  private A07 layout directories (six frozen and one disqualified draft), a
  draft manifest without discoverable condition/supplied-input/mapping/matrix
  fields, and no isolated protocol-review artifact. No screening, holdout or
  final-acceptance credit is awarded.
- Independent impasse/process review recorded `A07-MANIFEST-04`: the smallest
  resolution is a private versioned manifest with exactly two named
  product-input conditions differing in one variable, supplied-input proof,
  blinded mapping, six frozen holdouts and a complete 30-cell matrix, followed
  by an isolated protocol-review `PASS`. After that disposition, fetch again,
  bind candidate identity and run the no-credit canary before allocation.
- Capacity evidence is not a merge or performance claim. The serialized
  `vps` probe found 16 CPUs, low load, about 42.7 GiB available memory and
  about 23 GiB free disk at 94% use, with nightly Rust 1.95, pnpm 10.29.3,
  no Bun and unrelated active processes; no heavy job ran. Use remote work
  only through the exact-toolchain bundle procedure after headroom checks and
  one job at a time; retain hosted platform/performance proof.
- Shared Cargo lock contention observed during parallel local probes is now
  treated as resource timing; heavy Cargo commands are serialized and lock
  wait is never reported as test progress or proof.
- Topology while this owned candidate is present is `worktrees=35 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9`; report exits 0 and strict exits 1 only
  for preserved root/user dirt, external dirt, stale registrations and
  historical refs. The report includes this clean candidate worktree
  (`worktrees=35`); after merged closure it should return to the preserved
  baseline count of 34. Rerun both modes before handoff and remove only this
  slice's clean merged worktree/ref.
- Continuation decision: keep the existing runtime goal active. An empty
  ready-pending set is not completion or a whole-goal block; record the A07
  held action and continue bounded topology/R01/W02/W03 preparation that is
  independently authorized.

## Historical source-pointer reconciliation — 2026-09-10 UTC (`origin/master=755c28d`; superseded by PR #249)

- Owner/phase: process coordinator / `reconcile-handoff`. A fresh fetch
  resolved `origin/master` to the full PR #248 merge SHA
  `755c28ded66d1f2d82b38d27633be83a0233f30e`. The reviewed candidate was
  `89a0430ba9f4bbf141e717ed78de72b41c5ab3b6`, based on
  `6ed43c3c63fab7b60a86f1d587c067c9b04ded93`; its tree matches the merge.
  This documentation/skill process correction changes no product, evaluator,
  threshold, allocation, release, deployment, publication or invitation state.
- The revision-pinned ledger reports `items=32`, `ready_pending=0`,
  `unfinished=5`, `held=3`: A07 active, W03 verified, and R01/W02/F01 held.
  A metadata-only private audit still sees seven layout directories (six
  frozen-layout entries and one disqualified raw-hook draft), a draft
  `holdouts/manifest.md` with no discoverable condition/mapping/matrix fields,
  and no isolated protocol-review artifact. No screening, holdout or
  final-acceptance credit is awarded.
- Independent impasse review accepted `IMPASSE-PTR-02` as a concrete stale
  pointer finding: the `6ed43c3` checkpoint and older A07 pointers could be
  mistaken for live source. This reconciliation labels those records
  historical and makes the next action the private exactly-two-condition
  manifest plus isolated protocol-review `PASS`; only after that disposition
  may a fresh current-master candidate-bound canary run.
- Topology remains `worktrees=34 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9`; report exits `0` and strict exits `1`
  only for preserved root/external dirt and historical registrations/refs.
  The owned reconciliation worktree is clean and will be removed only after
  merged reachability and final scoped gates.

## Historical post-merge source-pointer lifecycle checkpoint — 2026-09-10 UTC (`origin/master=6ed43c3`; superseded by PR #248)

- Owner/phase: process coordinator / `reconcile-handoff` complete. PR #247
  merged the reviewed pointer correction as
  `6ed43c3c63fab7b60a86f1d587c067c9b04ded93` from candidate
  `577b6d63030aab338238088c85cfa7c7750baeb2`, based on
  `d62dd40f0d915e693db25cdea2fb29d1000e9b50`; the merged tree matches the
  reviewed candidate.
- The post-merge revision-pinned ledger remains `items=32`,
  `ready_pending=0`, `unfinished=5`, `held=3`: A07 active, W03 verified, and
  R01/W02/F01 retain their named holds. No card, evaluator, threshold,
  allocation, release, deployment, publication or invitation state changed.
- Hosted Documentation Scope, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks for PR #247 passed. Rust/product/performance/release/
  installer/website jobs were scope-skipped for the documentation-only slice
  and remain non-applicable, not outcome proof.
- The owned process worktree and branch were cleanly removed after merged tree
  comparison and exact cleanup. Post-cleanup topology remains
  `worktrees=34 dirty=2 prunable=3 unreadable=1 goal_branches=13
  unmerged_goal=9`; report exited `0`, strict exited `1` only for preserved
  root/external dirt and stale registrations/history.
- The d62 readiness audit below is historical candidate-base evidence, and this
  `6ed43c3` checkpoint is superseded by the current `2eda17e` reconciliation.
  At the next resume, fetch `origin/master`, rerun the ledger and route A07 to
  its private manifest/protocol-review `PASS` before a fresh current-master
  canary. No historical run receives screening or acceptance credit.

## Historical candidate-base/private-manifest readiness re-audit — 2026-09-10 UTC (`origin/master=d62dd40`; superseded by PR #247)

- Owner/phase: process coordinator / `investigate-prepare`, with the A07
  acceptance coordinator as the named next-action owner. The read-only reset
  fetched `origin/master=d62dd40f0d915e693db25cdea2fb29d1000e9b50`; no product
  source, evaluator, threshold, release, deployment, publication or
  invitation state changed.
- The immutable ledger at that revision reports `items=32`,
  `ready_pending=0`, `unfinished=5`, `held=3`: A07 is active, W03 verified,
  and R01/W02/F01 retain their named holds. Active/implemented/verified rows
  were inspected before pending rows; no pending card is executable.
- A redacted read-only audit of the private A07 store found six valid frozen
  holdout layouts and one disqualified construction draft. The available
  construction document remains explicitly a draft and does not contain the
  required separate exactly-two-condition record, private mapping,
  supplied-input proof, complete 30-cell matrix or isolated protocol-review
  `PASS`. Five distinct screen-named run families are retained as historical
  controls/attempts and receive no allocation credit; private values and raw
  evaluator output remain outside this repository.
- This is a concrete A07 held action, not a whole-goal stop: the smallest
  resolution is a versioned private manifest plus isolated protocol review
  returning redacted `PASS` or findings. After `PASS`, refresh the candidate
  identity against `d62dd40`, run the no-credit full-contract canary, and only
  then allocate the authorized 30-cell screen. No historical canary or run is
  reused.
- Independent process review accepted finding `IMPASSE-PTR-01`: older current
  pointers still routed from PR #245/`27ef54d` (and the ordered table from
  PR #244/`2902a07`). This candidate refreshes those pointers to `d62dd40` and
  labels the older snapshots historical; scoped rereview covers this exact
  correction.
- The topology audit reported `base=origin/master worktrees=34 dirty=2
  prunable=3 unreadable=1 goal_branches=13 unmerged_goal=9`; `--report`
  exited `0` and `--strict` exited `1`. The preserved root unknown file,
  external dirty worktree, stale registrations and historical goal branches
  remain outside ownership and were not changed.

## Post-merge cleanup reconciliation — 2026-09-10 UTC (PR #245 / 27ef54d)

- Owner/phase: process coordinator / `reconcile-handoff` complete. A fresh
  fetch resolved `origin/master` to
  `27ef54d489847e41e5907f7c74f870a2391a7dae`, the merge of PR #245 final
  head `391178676931ba935ad0058fce0bc55e101b3641` from base
  `2902a073f39f8e8a47a9658a6358791c3e7f4655`; exact ancestry exited `0`.
- PR #245's applicable hosted checks were terminal and passing: Documentation
  Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian. Product,
  Rust, performance, release, installer and website jobs were scope-skipped
  and remain non-applicable, not passing outcome proof.
- The owned `/private/tmp/assura-train-continuation-reconcile` worktree was
  clean and removed; local and remote `docs/train-continuation-reconcile`
  refs were deleted only after merged reachability. There is no uncommitted or
  abandoned process work from this slice. Root/user-owned dirt, three prunable
  registrations, one unreadable registration and historical unmerged goal
  refs remain preserved and outside ownership.
- The final post-cleanup topology audit reported
  `base=origin/master worktrees=34 dirty=2 prunable=3 unreadable=1
  goal_branches=13 unmerged_goal=9`; `--report` exited `0`, `--strict` exited
  `1`, and `git worktree prune --dry-run -v` remained read-only. The strict
  nonzero result is the preserved external/history set, not this slice.
- A detached clean checkout at `27ef54d` ran the revision-pinned ledger helper:
  `items=32`, `ready_pending=0`, `unfinished=5`, `held=3`; A07 remains active,
  W03 verified, R01/W02/F01 held, and no pending card is executable. No card,
  evaluator, threshold, release, deployment, publication or invitation state
  changed. The next action is A07's private exactly-two-condition and
  six-holdout manifest, isolated protocol-review `PASS`, then a fresh
  candidate-bound canary after refreshing `origin/master`.

## Historical continuation checkpoint — 2026-09-10 UTC (PR #244 / 2902a07; superseded)

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
