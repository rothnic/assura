# Process corrections and continuation plan

Status: active, current source as-of the latest refresh
`origin/master=dc031527afd400be52dfd8fe9cfdabc7a6caa685`.
This is an as-of checkpoint; fetch again before every phase. The reusable
state machine, checkpoint schema, layered disclosure and validation-budget
rules are in
[`execution-control-plane.md`](../../../../.agents/skills/assura-goal-execution/references/execution-control-plane.md).
This record is a process and evidence route; it does not close A07, grant
screening credit, or authorize release, deployment, publication, invitation,
protection changes or a CI-infrastructure change.

## Current correction checkpoint — PR #305 — 2026-09-11 UTC (`origin/master=dc031527`, no-credit)

- PR #305 merged reviewed head `5363b4a5d2e39beed6087bea6a5f6fa2d2aaf17c`
  from base `284e781` as `dc031527afd400be52dfd8fe9cfdabc7a6caa685`. Applicable
  exact-head and merge-SHA Documentation, CI Scope, Security Scope, Evidence
  Gates and GitGuardian checks passed; scope-skipped jobs remain non-proof.
- The fresh exact-toolchain candidate in
  `/private/tmp/assura-a07-current-reconcile-dc031` is source/tree
  `dc031527afd400be52dfd8fe9cfdabc7a6caa685`/
  `b68b66cc47f7e4081afd2dc21c1387de34b30c32`, with binary and shim
  `3cdaf976d7da61043a5867cb17ffd789b92b484e26f1e7d3a7989bd25ce816cf`.
  Exact Rust/Cargo `1.94.1`, login-shell identity, wrong-target and wrong-root
  controls pass. Two sibling-free canaries and full evaluators pass with zero
  critical failures; ambient skill metadata and hook-verifier disagreement are
  retained limitations and no credit is assigned.
- A rebind correction fixed a process failure observed during preparation:
  blind substitution had left current metadata pointing to nonexistent `r2`
  construction/receipt aliases and a wrong identity-control filename. The
  private packet now uses deliberate current aliases, preserves historical
  construction records, and passes the metadata-only validator (`valid=true`,
  zero errors) for six holdouts, two conditions and 30 reserved cells.
  Independent protocol review returned `PASS_NO_CREDIT` after correcting the
  stale contract digest and construction hash; the private review artifact
  records both findings and the scoped rereview. Screening, allocation, credit
  and product acceptance remain false.

Next action: reconcile this protocol-pass/no-credit checkpoint in a
current-base process slice. Refresh source, release/tag, PR/CI, topology and
the ledger after every merge; if source advances, classify the packet
candidate-base/no-credit and rebuild. Never reuse a candidate packet after its
source advances.

## Historical post-merge checkpoint — PR #304 — 2026-09-11 UTC (`origin/master=284e781`, candidate-base; superseded by dc031527)

- PR #304 merged reviewed head `17fa9197ed79083be4b8ba0714851aeaae9bdbf3`
  from base `d228472` as `284e78156370d49d8315f04391fcc74eaba3acb2`. Its
  exact-head Documentation, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks passed after independent exact-diff review `PASS`.
  Merge-SHA Documentation `34651068660`, Rust CI `34651068675` (CI Scope
  `103433100783`, Evidence Gates `103433136825`) and Security Audit
  `34651068699` completed successfully for applicable scope; scope-skipped
  rows remain non-proof. No product or authority state changed.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3`: A07 active, W03 verified and R01/W02/F01 held. The d228472
  candidate, freeze and two-condition canary/evaluator results are now
  candidate-base/no-credit after this source advance. Preserve their ambient
  skill-metadata limitation and initial structure-placement failure; no
  holdout, manifest, protocol, screening, allocation or acceptance state
  carries forward.
- The owned `docs/a07-canary-d228` branch/worktree was clean, merged and
  removed. Root unknown dirt, foreign dirty work, stale/prunable registrations
  and unfavorable R01 evidence remain preserved. The post-merge topology
  observation was `worktrees=53, dirty=3, prunable=3, unreadable=1` before
  owned cleanup.
- Next owner/action: refresh source, release/tag, PR/CI, topology and the
  revision-pinned ledger again before any candidate or held recovery phase. If
  A07 preparation is separately authorized, build a fresh exact-toolchain
  candidate from `284e781`, run identity and sibling-free no-credit canary,
  rebind six holdouts plus the exactly-two-condition manifest, and obtain
  isolated protocol `PASS` before screening. Otherwise continue the smallest
  independently authorized held recovery slice. Never reuse d228472 evidence
  for credit or let an empty pending queue end the goal.

## Historical candidate-bound checkpoint — PR #303 — 2026-09-11 UTC (`origin/master=d228472`, superseded by `284e781`)

- PR #303 merged reviewed head `bec17c3aa872ee00ec43cf25a291423b1a96667f`
  from base `453a32a` as `d2284724192dbca848bbd135afacc33d2533e06f`. Its
  exact-head Documentation, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks passed after independent exact-diff review `PASS`.
  Merge-SHA Documentation `34646077514`, Rust CI `34646077520` and Security
  Audit `34646077494` completed successfully for applicable scope; skipped
  rows remain non-proof. No product or authority state changed.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3`: A07 active, W03 verified and R01/W02/F01 held. The 453a32a process
  checkpoint and 0dff804 candidate are historical or candidate-base/no-credit.
  The root unknown path, foreign dirty worktree, stale/prunable registrations
  and unfavorable R01 evidence remain preserved.
- A fresh exact-toolchain candidate in clean owned checkout
  `/private/tmp/assura-a07-current-d228` built with Rust/Cargo `1.94.1` and
  passed source/tree/binary/shim identity, login-shell, wrong-target and
  wrong-root controls. The public/private freeze copies are byte-identical.
  A first root-level freeze placement failed the structure gate and was moved
  under ignored `target/a07-evidence/`; that unfavorable setup result remains
  recorded and was not hidden.
- Two fresh sibling-free source-only children completed the fixed public
  initializer with exit `0`. Scans found no private evaluator contract/output,
  foreign checkout, repository checkout or sibling-condition references.
  Generic ambient user-level skill metadata appeared in both streams despite
  `--ignore-user-config`; this is an explicit isolation limitation and does
  not earn credit. Full evaluators ran only after child exit with exactly the
  seven declared dimensions; both passed with zero critical failures, the
  expected negative naming probe and a collected native test.
- No current holdout/manifest rebind or isolated protocol review exists yet.
  Screening, allocation, credit and authority flags remain false. Next owner/
  action: rebind the six immutable holdouts and exactly-two-condition manifest
  to d228472, obtain isolated protocol `PASS`, and preserve the ambient
  context limitation in that review before separately authorized screening.
  If a process merge advances source, classify this candidate as candidate-base/
  no-credit and repeat the fresh-source sequence.

## Historical post-merge checkpoint — PR #302 — 2026-09-11 UTC (`origin/master=453a32a`, superseded by `d228472`)

- PR #302 merged reviewed head `74bd69e69ae0673c846f1f2b17a8bc5e1ce432ef`
  from base `24a1966` as `453a32afb7367f93b943ba70c68f4af82bc0e688`; its exact-head
  Documentation Scope, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks passed after independent exact-diff review `PASS`. The
  reviewed tree equals the merge tree.
- Configured push-triggered Documentation run `34644511575` (scope job
  `103411957000`), Rust CI run `34644511679` (CI Scope `103411957349`, Evidence
  Gates `103411992718`) and Security Audit run `34644511551` (Security Scope
  `103411956405`) completed successfully at the merge SHA. Product/Rust/
  performance/release, documentation build/marketing and security-audit jobs
  were scope-skipped and remain non-proof. This is a post-merge process
  observation, not product acceptance.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3` with A07 active, W03 verified and R01/W02/F01 held. The 24a1966
  reconciliation is historical and the 0dff804 candidate is candidate-base/
  no-credit after this source advance. No screening, allocation, acceptance,
  release, deployment, publication, invitation or protection authority
  changed.
- The owned `docs/a07-postmerge-24a1966` branch/worktree was clean after merge;
  remove only that exact owned branch/worktree. The new current reconciliation
  checkout is `/private/tmp/assura-a07-postmerge-453a`. The root unknown path,
  foreign dirty worktree and stale/prunable topology registrations remain
  preserved. The report at this checkpoint is
  `base=origin/master, worktrees=52, dirty=2, prunable=3, unreadable=1,
  goal_branches=13, unmerged_goal=9`.
- A read-only VPS probe confirms the efficiency lane remains held: `vps-dev`
  is not configured or resolvable; configured `vps` has 16 CPUs, 61 GiB RAM,
  20 GiB free of a 339 GiB root volume (95% used), and Rust/Cargo
  `1.95.0-nightly`. No remote heavy job, cache deletion or infrastructure
  change was performed. Remote Linux output cannot replace platform or hosted
  proof.
- Next action: close the merged 24a process checkout/branch, then rebuild and
  identity-freeze a fresh exact-toolchain candidate from 453a in a clean owned
  checkout. Run the sibling-free source-only no-credit canary, rebind all six
  holdouts and the two-condition manifest, and obtain isolated protocol `PASS`
  before any separately authorized screening. Continue held R01/W02/F01 routes
  independently and keep the goal active.

## Historical post-merge checkpoint — PR #301 — 2026-09-11 UTC (`origin/master=24a1966`, superseded by `453a32a`)

- PR #301 merged reviewed head `428c26aee89b449e5c57ed1589d6a9b8d083c3bc` as
  `24a1966`; exact-head Documentation, CI Scope, Security Scope, Evidence
  Gates and GitGuardian checks passed. The reviewed tree equals the merge tree.
- Configured push-triggered Documentation `34642991331`, Rust CI
  `34642991392` and Security Audit `34642991344` completed successfully at
  the merge SHA for their applicable scope. Rust CI's CI Scope and Evidence
  Gates passed; product/Rust/performance/release jobs were scope-skipped and
  remain non-proof. This is a post-merge process observation, not product
  acceptance.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3` with A07 active, W03 verified and R01/W02/F01 held. The 0dff804
  candidate identity freeze and process artifacts are now candidate-base or
  historical no-credit after this source advance. No screening, allocation,
  acceptance, release, deployment, publication, invitation or protection
  authority changed. The merged docs/a07-current-0dff branch/worktree was
  verified clean and removed; root/foreign/stale topology exceptions remain
  preserved.
- Next action: rebuild and identity-freeze a fresh candidate from 24a1966 in a
  clean owned checkout before the sibling-free source-only no-credit canary,
  six-holdout/two-condition rebind and isolated protocol `PASS`. Continue held
  R01/W02/F01 routes independently and do not reuse the 0dff804 packet.

## Historical A07 preparation checkpoint — 2026-09-11 UTC (`origin/master=0dff804`, superseded by `24a1966`)

- `/root` refreshed the source and ledger before claiming this preparation
  slice. `origin/master=0dff804421c7563b08773eb75d9327fd0194db56` has tree
  `4dcb61add201e72e6b61f3bd8392f508d008160c`; the ledger remains
  `items=32; ready_pending=0; unfinished=5; held=3` with A07 active, W03
  verified and R01/W02/F01 held. Root/user-owned dirt and foreign or stale
  topology entries remain preserved.
- In clean owned checkout `/private/tmp/assura-a07-current-0dff`, pinned
  `cargo +1.94.1 build --release` exited `0` in approximately 297 seconds.
  Candidate version is `assura 0.4.0`; binary SHA-256 is
  `7fed6686a82e6e4a792884c1604518e21818e6494cad1aa09963f84c28cc300b`.
  The regular login-shell shim SHA-256 is
  `bdcce0e34c64d0745fa671613b1992726f4b1326366f0aa5dd0502934e5b4ff1`.
  `/bin/zsh -lic` resolves the shim and the deliberate wrong-target control
  resolves `/usr/local/bin/assura`; the dirty-root head is rejected by source
  SHA mismatch.
- The source-bound freeze and identity records are byte-identical after the
  scoped review finding `A07-0DFF-FREEZE-001` and its path-normalization
  follow-up were corrected. `cmp`, JSON, source/tree, binary/shim, version,
  login-shell and negative-control assertions all pass. The independent
  reviewer returned `PASS`.
- This remains no-credit preparation: `canary_status=not-run`,
  `protocol_review_status=not-run`, screening/allocation/credit are false, and
  no product, holdout, acceptance, release, deployment, publication,
  invitation or protection authority changed. The old 4560c710 packet remains
  candidate-base/no-credit and was not reused.
- Next action: `/root` runs a fresh sibling-free source-only canary against
  this candidate, then rebinds the six holdouts and two-condition manifest and
  obtains isolated protocol `PASS` before any separately authorized screening.

## Historical post-merge reconciliation — PR #299 — 2026-09-11 UTC (`origin/master=651ef31`, superseded by `0dff804`)

- PR #299 merged the reviewed current-checkpoint reconciliation as
  `651ef31ea4d609a83c59af2a458f4313db91c9c3` from reviewed head
  `c6a47ae4e486d4d226a747a3f49653c235f251d1`. Its exact-head applicable
  Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian
  checks passed, and the reviewed head tree equals the fetched merge tree.
  The configured push-triggered Rust CI (`34635475363`), Documentation
  (`34635475400`) and Security Audit (`34635475378`) workflows completed at
  the merge SHA. Product/Rust/performance/release rows were explicitly skipped
  by scope and are not product proof.
- Correction: this reconciliation advances the current source pointers only;
  it does not close a product card, repair the retained R01 macOS SIGINT
  failure, authorize A07 screening/allocation, or change W03/R01/W02/F01
  dispositions. The revision-pinned ledger remains `items=32; ready_pending=0;
  unfinished=5; held=3`, and the 4560c710 candidate and packet remain
  candidate-base/no-credit metadata after the source advance.
- Next action: refresh source/release/tag/PR/CI/topology and the ledger again,
  inspect active, implemented, verified and held records before pending rows,
  and keep one explicitly owned recovery or preparation action live. Close only
  the exact clean owned process branch after current-head review and post-merge
  proof.

## Historical post-merge reconciliation — PR #298 — 2026-09-11 UTC (`origin/master=329bce0`, superseded by `651ef31`)

- PR #298 merged the reviewed current-checkpoint reconciliation as
  `329bce02396d1378bc4306c446294c3f5ebbfd41` from reviewed head
  `ba4c980c13001e97bdd6aa06e68bd5d7a115fd53`. Its exact-head applicable
  Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian
  checks passed, and the reviewed head tree equals the fetched merge tree.
  The configured push-triggered Rust CI (`34633883919`), Documentation
  (`34633883756`) and Security Audit (`34633883854`) workflows completed at
  the merge SHA. Product/Rust/performance/release rows were explicitly
  skipped by scope and are not product proof.
- Correction: this reconciliation advances the current source pointers only;
  it does not close a product card, repair the retained R01 macOS SIGINT
  failure, authorize A07 screening/allocation, or change W03/R01/W02/F01
  dispositions. The revision-pinned ledger remains `items=32; ready_pending=0;
  unfinished=5; held=3`, and the 4560c710 candidate and packet remain
  candidate-base/no-credit metadata.
- Next action: refresh source/release/tag/PR/CI/topology and the ledger again,
  inspect active/implemented/verified/held records, and keep one explicitly
  owned recovery or preparation action live. Close only the exact clean owned
  process branch after current-head review and post-merge proof.

## Historical post-merge reconciliation — PR #297 — 2026-09-11 UTC (`origin/master=6d57b86`, superseded by `329bce0`)

- PR #297 merged the reviewed current-checkpoint reconciliation as
  `6d57b8660d4db72734d11141e61eeaddce2fbb16` from reviewed head
  `004aee7cd67a045d0fc5e040759d670961036870`. Its exact-head applicable
  checks and merge-SHA workflows passed; product/Rust/performance/release
  rows were scope-skipped and were not product proof. This is historical
  process evidence only; PR #298 is the current source reconciliation.
- The ledger, retained R01 diagnostic and 4560c710 candidate-base/no-credit
  packet were unchanged. Refresh source and rerun the ledger before any
  candidate or authority action.

## Historical post-merge reconciliation — PR #296 — 2026-09-11 UTC (`origin/master=83c382a`, superseded by `6d57b86`)

- PR #296 merged the reviewed post-merge reconciliation as
  `83c382a78a6b616c3c420fb117404333f78d4381` from reviewed head
  `a7045dbc9786add7aedb33d9859fbda703926025`. Its exact-head applicable
  Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian
  checks passed, and the reviewed head tree equals the fetched merge tree.
  The configured push-triggered Rust CI (`34630104586`), Documentation
  (`34630104663`) and Security Audit (`34630104624`) workflows completed at
  the merge SHA. Product/Rust/performance/release rows were explicitly
  skipped by scope and are not product proof.
- Correction: this reconciliation advances the source pointers only; it does
  not close a product card, repair the retained R01 macOS SIGINT failure,
  authorize A07 screening/allocation, or change W03/R01/W02/F01 dispositions.
  The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3`, and the 4560c710 candidate and packet remain candidate-base/no-
  credit metadata.
- Next action: refresh source/release/tag/PR/CI/topology and the ledger again,
  inspect active/implemented/verified/held records, and keep one explicitly
  owned recovery or preparation action live. Close only the exact clean owned
  process branch after current-head review and post-merge proof.

## Historical post-merge reconciliation — PR #295 — 2026-09-11 UTC (`origin/master=9047a3d`, superseded by `83c382a`)

- PR #295 merged the reviewed workflow-fence and measured-capacity correction
  as `9047a3d07e704cde0809a97cbe75f2cb1ce4af33`. Its exact-head applicable
  Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian
  checks passed. The configured push-triggered Documentation run
  `34628874282`, Security Audit run `34628874295` and Rust CI run
  `34628874311` each completed successfully at the merge SHA; Rust CI's CI
  Scope and Evidence Gates passed while product/Rust/performance/release jobs
  were explicitly skipped by scope. The reviewed head tree equals the fetched
  merge tree.
- Correction: this is the required second-stage reconciliation observation,
  not product acceptance. The retained R01 macOS SIGINT failure remains
  unfavorable evidence; the ledger remains `items=32; ready_pending=0;
  unfinished=5; held=3` with A07 active, W03 verified and R01/W02/F01 held.
  The 4560c710 A07 candidate and packet remain candidate-base/no-credit.
- Next action: refresh source/release/tag/PR/CI/topology and the ledger again,
  keep one explicitly owned recovery or preparation action live, and close the
  integrated process branch only after exact clean-worktree proof. Do not infer
  card, screening, allocation, release, deployment, publication, invitation or
  authority credit from this process merge.

## Historical measured remote-capacity hold — 2026-09-11 UTC (`origin/master=c9ac2a8`, superseded by `9047a3d`)

- The requested `vps-dev` SSH alias is not configured or resolvable in this
  shell. The configured `vps` alias was probed read-only: 16 CPUs, 61 GiB RAM,
  20 GiB free of 339 GiB root (95% used), Rust/Cargo `1.95.0-nightly`, Node
  `22.22.1`, pnpm `10.29.3`, and unrelated long-running cargo-watch/PM2 jobs.
  No remote state was changed.
- Correction: remote execution is an optional efficiency lane, not a proof or
  authority source. Keep it held until exact toolchain, free-disk margin and
  owned-job isolation are measured for the specific committed candidate. Do
  not retry an unresolved alias, delete unrelated data, or use Linux output as
  a substitute for macOS/Windows/browser/release/Cloudflare/hosted gates.
- Next action: after the process-correction branch is integrated, re-probe only
  an explicitly configured host if a heavy validation job is authorized and
  record remote HEAD, lockfile, toolchain, elapsed time and exit. Otherwise
  continue the smallest authorized local or hosted route while retaining the
  R01 macOS SIGINT diagnostic.

## Historical post-merge workflow correction — 2026-09-11 UTC (`origin/master=c9ac2a8`, superseded by `329bce0`)

- PR #294 merged the reviewed execution-control-plane correction at
  `c9ac2a84a93dce752ec7b5216ad87639ba04dee8`. Exact-head Documentation, CI
  Scope, Security Scope, Evidence Gates and GitGuardian checks passed, and the
  configured post-merge Rust CI, Documentation and Security workflows passed.
  Product/Rust/performance/release jobs were scope-skipped and remain
  non-applicable.
- The retained R01 failure is now bound to its first actionable hosted
  evidence: run `34615572565`, macOS job `103316578631`, failed at
  `tests/watch_cli.rs:196` because `watch_stops_cleanly_without_runtime_artifacts`
  did not stop after SIGINT; 15 `watch_cli` tests passed and one failed in
  11.08s. A focused local Darwin rerun passed once and is not hosted proof.
- Correction: pull-request checks and local passes are pre-merge evidence;
  configured push-triggered workflows at the merge SHA are a separate
  reconciliation gate. Failed, cancelled, unavailable, zero-test,
  scope-uncertain or absent post-merge results retain the exact handle and
  route the smallest recovery. No unchanged retry, threshold weakening,
  product credit or authority is implied.

## Historical post-merge and candidate reconciliation — 2026-09-11 UTC (`origin/master=122fa0b3`, superseded by `c9ac2a8`)

- PR #293 merged the reviewed process/goal reconciliation at
  `122fa0b3d976bb5196359aee48ad67bf272fb541`. Its exact-head Documentation,
  CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed;
  product/Rust/performance/release jobs were scope-skipped and are not
  product proof. The post-merge Rust CI, Documentation and Security workflows
  passed their applicable scope jobs. The earlier macOS watch-SIGINT failure
  remains retained unfavorable hosted evidence.
- The ledger is still `items=32; ready_pending=0; unfinished=5; held=3`:
  A07 active, W03 verified and R01/W02/F01 held. The 4560c710 candidate,
  canaries and protocol packet are candidate-base/no-credit after this source
  advance; no screening, allocation, credit, release, deployment, publication
  or invitation authority exists.
- The coordinator owns a process-only continuation route: refresh
  source/release/tag/PR/CI/topology and the ledger, inspect active/implemented/
  verified/held records, and keep one named recovery or preparation action
  live. A future A07 screening phase must rebuild and rebind at the refreshed
  source before allocation; otherwise continue the authorized recovery slice.
  Do not stop at the empty pending queue.

- PR #292 merged the reviewed agent-content-preservation recovery slice as
  `4560c710967d59993b9ea4f9b86613d446443f79`. Its exact-head applicable
  Documentation, CI Scope, Security Scope, Evidence Gates, Rust/platform,
  performance, release/adoption smoke and GitGuardian checks passed; Security
  Audit was scope-skipped. The separate push-triggered Rust CI run
  `34615572565` failed in the macOS `watch_stops_cleanly_without_runtime_artifacts`
  SIGINT test and cancelled the Ubuntu/Windows matrix siblings. A focused
  local rerun passed once. This hosted result is retained as an unresolved
  diagnostic and is not silently retried or counted as green.
- The ledger is still `items=32; ready_pending=0; unfinished=5; held=3`:
  A07 active, W03 verified and R01/W02/F01 held. The clean current candidate
  is frozen with exact Rust/Cargo `1.94.1`, absolute release target and
  login-shell identity controls that reject wrong target and wrong root.
  Two fresh source-only conditions pass composed initialization and the full
  seven-dimension evaluator with zero critical failures; they remain no-credit
  canaries.
- The private packet is rebound to the current candidate with six immutable
  holdouts, current creation and second-readonly references, source-only fixture
  freshness, shared invariants, evaluation bindings, blinded mapping, exactly
  two conditions differing in one product input and 30 unique reserved cells.
  Independent review found `A07-4560-RECEIPT-ID-001` (missing explicit aliases
  between stable condition rows and receipt IDs) and
  `A07-4560-PROTOCOL-HASH-001` (stale construction digest). An explicit alias
  map was added without changing receipt bytes; the construction and affected
  artifact hashes were recomputed and assertions pass. The scoped rereview
  returned `PASS` and is recorded in the private protocol artifact with pre-
  and post-verdict hashes. Screening, allocation, credit and product acceptance
  remain false.
- Next owner/action: refresh source, release/tag, PR/CI, topology and the
  revision-pinned ledger before any further A07 phase. If a separately
  authorized screening gate is granted, execute only its contract and gates;
  otherwise continue the hosted watch-SIGINT diagnostic or another explicitly
  authorized recovery slice. Convert any future vague concern into a concrete
  contract, location, failure scenario and smallest verification; never weaken
  performance/cancellation gates or merge unresolved current-base work.

## Historical post-merge reconciliation — 2026-09-11 UTC (`origin/master=40f1155c`, superseded by `329bce0`)

- PR #289 merged the reviewed durable goal, layered-routing skill metadata and
  current-route corrections at `40f1155c3d26dc40141d2464d7e2f027f7bb9770`;
  applicable Documentation, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks passed. Product/Rust/performance/release jobs were
  scope-skipped and remain non-applicable. The 851a6 candidate packet and its
  isolated protocol `PASS` are now candidate-base/no-credit and must not be reused.
- Refresh source, release/tag, PR/CI, topology and the revision-pinned ledger,
  then rebuild A07 at the fetched source with a fresh identity freeze,
  no-credit canary, packet rebind and isolated protocol review. No screening,
  acceptance, release, deployment, publication or invitation authority is
  implied.

## Historical post-merge reconciliation — 2026-09-11 UTC (`origin/master=851a6b8`, superseded by `40f1155c`)

- PR #288 merged the reviewed durable-goal and continuation artifacts at
  `851a6b831ea841317b78d94fce658a6974ef401a` after PR #287 reconciled the
  reviewed cd629 process checkpoint as `ca81689`. Its applicable Documentation,
  CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as
  did post-merge Documentation, Security Audit and Rust CI workflows.
  Product/Rust/performance/release jobs were scope-skipped and are not
  acceptance proof. No release branch is available; tags remain a separate
  availability fact.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3`: A07 active, W03 verified and R01/W02/F01 held. The ac3, cd629 and
  ca816 candidate canaries, identity controls, six-handle/two-condition
  packets and isolated protocol `PASS` records are historical no-credit
  metadata after this source advance. A fresh 851a6 identity freeze is
  privately prepared, but its canary, packet rebind and protocol review remain
  pending. No current candidate, screening allocation, product, threshold or
  authority state exists.
- Owner/phase: `/root` / fresh current-source A07 candidate preparation.
  Refresh source, release/tag, PR/CI, topology and the revision-pinned ledger;
  complete the fresh explicit-workdir 851a6 candidate identity, run the bounded
  no-credit canary, rebind the packet and obtain isolated protocol `PASS` before
  any separately authorized screening request. Do not merge another
  process-pointer update while the packet is in flight; if source advances,
  retain it as historical and repeat the complete sequence. Preserve
  unfavorable evidence, the root unknown path, foreign dirty worktree, stale
  registrations and external holds.

## Historical post-merge reconciliation — 2026-09-11 UTC (`origin/master=cd629d4`, superseded by `ca81689`)

- PR #286 merged the reviewed ac3 process checkpoint at
  `cd629d413491f0214fb16850bba629524a342be1`; its applicable Documentation,
  CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed, as
  did the post-merge Documentation, Security Audit and Rust CI workflows.
  Product/Rust/performance/release jobs were scope-skipped and are not
  acceptance proof. Release/tag availability remains a separate fact.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
  held=3`: A07 active, W03 verified, R01/W02/F01 held. The ac3 candidate,
  canaries, identity controls, six-handle/two-condition/30-cell packet and
  isolated protocol `PASS` are historical no-credit metadata after this source
  advance. No candidate, screening allocation, product acceptance or authority
  state is current.
- Owner/phase: `/root` / fresh current-source candidate preparation. Refresh
  source, release/tag, PR/CI, topology and the revision-pinned ledger, then
  build and freeze a fresh explicit-workdir cd629d4 candidate, run the bounded
  no-credit canary, rebind the packet and obtain isolated protocol `PASS`
  before any separately authorized screening request. Preserve unfavorable
  evidence and all external holds.

## Historical candidate and protocol reconciliation — 2026-09-11 UTC (`origin/master=ac3eb13`, superseded by `cd629d4`)

- PR #285 reconciled the reviewed c4f post-merge route at `ac3eb13`; its
  applicable checks passed. The exact-toolchain candidate, two canaries,
  identity controls and six-handle/two-condition/30-cell packet were no-credit
  preparation; isolated protocol rereview returned `PASS` after two concrete
  metadata findings were repaired. No screening or acceptance credit existed.
- The source advance in PR #286 makes that packet historical. Retain the
  unfavorable wrapper evidence and repeat the complete current-source
  candidate sequence before allocation.

## Historical post-merge reconciliation — 2026-09-11 UTC (`origin/master=c4f57d7`, superseded by `ac3eb13`)

- PR #284 merged the reviewed process/evidence-route slice at `c4f57d7`; its applicable Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed. Product/Rust/performance/release jobs were scope-skipped and are not acceptance proof. The latest tag is `v0.3.0-447-gc4f57d7`; no release branch exists.
- The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5; held=3`: A07 active, W03 verified, R01/W02/F01 held. The topology report still contains only preserved unknown/foreign/stale exceptions; no pending card is executable.
- The 30b candidate, two no-credit canaries, six-handle packet and protocol `PASS` are historical no-credit metadata after this source advance. No candidate, screening allocation, product acceptance or authority state is current at c4f.
- Owner/phase: `/root` / fresh current-source candidate preparation. The next action is an explicit-workdir c4f candidate build and identity freeze, bounded no-credit canary, current packet rebind and isolated protocol review before any separately authorized screening. Preserve all unfavorable runner evidence and external holds.

## Historical current-source candidate canary — 2026-09-11 UTC (`origin/master=30b4c663`, superseded by `c4f57d7`)

- PR #283 reconciled the reviewed process route into `origin/master=30b4c663`; its applicable Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian checks passed. Product/Rust/performance/release jobs were scope-skipped and are not acceptance proof. The latest tag remains `v0.3.0`; no release branch exists.
- The revision-pinned ledger is `items=32; ready_pending=0; unfinished=5; held=3`: A07 active, W03 verified, R01/W02/F01 held. The topology report still contains only preserved unknown/foreign/stale exceptions; no pending card is executable.
- A clean explicit-workdir candidate was built from this source with Rust/Cargo `1.94.1` and Assura `0.4.0`. Two fresh sibling-free source-only children completed the initializer and full seven-dimension evaluator with the expected negative policy control. These are bounded no-credit canaries only.
- Earlier runner defects (minimal PATH omitted `node`; an exit sentinel misreported a successful child) remain unfavorable evidence. The corrected reruns also record ambient user-level skill metadata despite `--ignore-user-config`; no private contract, mapping, hidden oracle, foreign worktree or global Assura fallback was observed.
- The first isolated review found three concrete metadata blockers: historical second-readonly binding refs, historical fixture handles in the current manifest/matrix, and missing evaluator/predicate/threshold identities in condition invariants. A follow-up found one stale top-level second-readonly ref. The private correction adds current fixture-freshness and r2 second-readonly records plus a shared invariant record; the scoped rereview returned `PASS`. Screening authorization is false and zero cells are allocated. No product, threshold, allocation or authority state changed.

## Historical post-merge reconciliation — 2026-09-11 UTC (`origin/master=55a38ff`)

- The reset fetched `origin/master=55a38ff68e39aae9ede78bb51c01a9c2e1392de9`.
  PR #282 merged the reviewed process-correction slice from `af5240d`; its
  applicable Documentation Scope, CI Scope, Security Scope, Evidence Gates and
  GitGuardian checks passed. Product/Rust/performance/release jobs were
  scope-skipped for that documentation/skill change and are not acceptance
  proof. The latest tag remains `v0.3.0` (the source describes as
  `v0.3.0-445-g55a38ff`); no release branch is present.
- The revision-pinned ledger at this source is
  `items=32; ready_pending=0; unfinished=5; held=3`: A07 is active, W03 is
  verified, and R01/W02/F01 retain their separate holds. Open PRs #194, #187,
  #142 and #195 remain outside this route; #194 has a failed Performance
  Report, #187 still has unverified Workers-build/deployment coupling, #142
  has macOS/Alpine failures and a cancelled Windows job, and #195 is an
  unmerged R03 collector change. None is merge-ready evidence for this goal.
- The topology report remains non-zero only for preserved state: the root's
  unknown A04 note, one foreign dirty worktree, three stale/prunable
  registrations and historical unmerged goal branches. The owned process
  branch/worktree for PR #282 was merged and removed after tree/HEAD proof; no
  preserved exception was changed.
- The `f4368883` candidate-bound canary and private protocol `PASS` are now
  historical no-credit metadata because this process-only merge advanced the
  source. No candidate, holdout binding or manifest is current at `55a`; no
  screening cell is allocated and no product, threshold or authority state
  changed.
- Owner/phase: `/root` / current-source candidate preparation. The next real
  action is to create a fresh clean owned `55a38ff` candidate, verify its
  explicit-workdir/toolchain identity, run the bounded no-credit canary, and
  rebuild/review private packet metadata before any separately authorized
  screening request. Do not reuse the `f4368883` binary or packet.

## Findings from the `f4368883` reset (historical as-of)

- The root checkout retains one unknown/user-owned path,
  `.trellis/tasks/09-04-maturity-portfolio-strategy/research/a04-host-status-doctor-permission-gap.md`.
  It is outside this work and was not touched. All owned work ran from clean
  detached or branch worktrees.
- Workflow/context routing is healthy: the workflow gate was reattached to the
  canonical task and `audit-context-routing.py` returned `checks=43
  failures=0 PASS`; `AGENTS.md` is 96 lines and remains a universal router.
- The refreshed ledger has 32 items, zero ready-pending rows, five unfinished
  rows and three narrow holds. A07 is the sole active lane; W03 is verified;
  R01, W02 and F01 retain their separate evidence/authority holds. No pending
  card can supersede A07 without new dependency evidence.
- The exact local Rust/Cargo `1.94.1` build from the current source succeeded in
  239.02 seconds. The `vps-dev` alias is unresolved. The reachable `vps` host
  has useful CPU/RAM but about 95% root-disk use, only about 20 GiB free, a
  nightly Rust toolchain rather than the exact pinned toolchain, and no Bun.
  It was correctly not selected for this candidate; Linux remains a supplement
  rather than macOS/Windows/browser/host-permission proof.
- Two early child launches failed before product work because the minimal PATH
  omitted `codex`/`node`; an authenticated retry with the original private
  home then completed. These event streams remain unfavorable no-credit
  runner evidence and the unchanged method is not retried.
- Two fresh, sibling-free source-only Codex children ran against the explicit
  current candidate. The login-shell identity matched the regular candidate
  shim, fixed release target, version `assura 0.4.0`, source/tree and the
  exact Rust/Cargo toolchain. Both initializer and full evaluator exits were
  zero; all seven declared dimensions passed and the trusted negative policy
  control rejected as expected. This is a candidate-bound no-credit canary,
  not screening or acceptance evidence.
- The child event streams show ambient user-level skill metadata being read
  despite `--ignore-user-config`. No evaluator contract, private mapping,
  hidden oracle or foreign worktree was supplied or read. This is a runner
  isolation limitation that must be retained and explicitly reviewed; it is
  not silently treated as product credit or as proof that host activation was
  approved.

## Protocol-review corrections — current packet

The first isolated metadata review returned six concrete blockers rather than
silently allowing the canary to advance: the current public-contract digest
was stale; condition rows lacked complete contract/prompt/candidate identity;
the identity record lacked fresh wrong-target and wrong-root controls; the
six holdouts were not individually bound to the current candidate; the
second-readonly record did not repeat the six creation records; and the
reserved matrix was only a pointer to a superseded 30-cell matrix.

The private r2 packet corrected each finding without changing product code,
thresholds, allocation, screening authority or privacy boundaries. It now
contains the authoritative current contract reference, complete per-condition
and per-cell identity, fresh identity controls, a current six-layout overlay,
six explicit second-readonly creation comparisons and a materialized current
30-cell reservation. Private JSON/schema checks pass; the isolated protocol
rereview returned `PASS` within its metadata scope. The packet remains
no-credit preparation and cannot authorize screening.

That rereview found two additional contract-shape omissions: each condition
row lacked its required redacted `public_summary`, and the `invariants` object
did not repeat the complete candidate/contract/prompt/fixture/toolchain bundle.
Those fields are now present in both private condition rows, the manifest
assertions pass again, and the same isolated reviewer recorded the second
scoped rereview as `PASS`. This correction also remains metadata-only and
cannot grant credit.

## Corrected execution route

1. **Freeze identity once per source.** Record the full source/tree SHA, regular
   executable and fixed target SHA, version, exact compiler/Cargo identity and
   login-shell `command -v` result before any child. A target directory or
   parent-shell PATH is not identity proof.
2. **Use a clean-room fixture per condition.** Give each child a dedicated
   disposable parent containing one source-only fixture and no sibling
   worktrees, harnesses, contracts or previous results. Keep the prompt fixed,
   conditions private and event/evaluator records separate.
3. **Preflight and classify context.** Scan the retained child stream for
   evaluator/private paths, hidden labels, foreign worktrees and global Assura
   fallback. Any such read invalidates the run and earns no credit. Generic
   ambient skill metadata is recorded as a limitation and must be dispositioned
   by the isolated protocol reviewer; it never upgrades a run by itself.
4. **Evaluate only after the child exits.** Read the evaluator's declared
   dimensions and pass exactly that set; keep the negative policy probe under
   `policy`. Preserve zero/nonzero exits, elapsed time, test count and every
   failed or invalid attempt. Do not weaken the contract to accommodate a
   runner failure.
5. **Rebind before screening.** The current private rebind points the two
   supplied-input receipts, candidate identity, six immutable holdout handles,
   creation records, exact toolchain, excluded draft and reserved 30-cell
   matrix to the current candidate. An independent protocol review must return
   `PASS`; its scope is metadata only and it cannot grant screening authority.
6. **Gate the product screen separately.** After protocol `PASS`, request or
   verify the separately authorized screening gate. Allocate zero cells until
   that authority exists. Preserve the existing 30-run screen, untouched
   holdout, follow-up feature and final ten-per-stack (at least 9/10) thresholds.
7. **Merge process or product work only at the merge fence.** A candidate must
   be committed, independently reviewed, current-base, fully locally and
   hosted-gated for its changed surface, and have no unresolved performance,
   zero-test, skipped-required or authority findings. After merge, fetch again,
   prove reachability/tree equality, rerun the ledger and strict topology audit,
   and close only the exact clean owned branch/worktree.

## Efficient validation placement

- Run workflow/context/structure/scope/target identity checks before any heavy
  command. Use one serialized Cargo heavy sequence per checkout; `cargo xtask
  pr` already nests `fast`, so do not pay for an unchanged duplicate.
- Build one candidate per source SHA and reuse only when source, dependencies,
  configuration, toolchain, environment and invocation are unchanged. A source
  advance or metadata change invalidates dependent proof.
- Use the VPS only after a fresh alias, load, memory, disk-margin, existing-job
  and exact-toolchain probe. The current probe failed the exact-toolchain/disk
  selection test, so local execution was the safer measured choice. Remote
  Linux results never replace platform-specific or hosted gates.
- Propose CI cache/parallelism changes only after three comparable prepared
  candidate runs record queue time, execution time, first-pass success, retries
  and coverage. Keep thresholds and suite coverage identical while measuring.

## Layered context contract

Keep `AGENTS.md` as the short universal router. At reset or compaction load the
workflow gate, current source/ledger and this plan; load only the selected card
packet and the phase reference needed for the next decision. Load runner
isolation for A07, CI triage for slow/failed hosted checks, validation routing
for gate placement, and continuation control before yielding or handing off.
Do not copy historical snapshots, private values, raw child events or evaluator
oracle details into prompts or public evidence. The context audit is the cheap
proof that these links remain reachable; a passing audit is not product
acceptance.

## Historical live checkpoint — `f4368883`

Owner: `/root`; phase: A07 candidate-bound canary and corrected private rebind.
Current candidate: f4368883 with identity recorded in the private freeze and
receipts. The two evaluator results are private and no-credit. The initial
provenance findings and the two follow-up manifest-shape findings have been
corrected in the private r2 packet; the isolated reviewer recorded `PASS`.
Raw child transcripts and evaluator output remain excluded. Exact next action:
refresh source/release/tag/PR/CI/topology and the ledger, prove candidate and
packet identity at that revision, then seek the separately authorized
screening gate. Even a protocol `PASS` leaves the packet no-credit until that
authority is verified.

## Historical live checkpoint — `55a38ff` (superseded by `30b4c663`)

Owner: `/root`; phase: current-source candidate rebuild and identity freeze.
The ledger is `32/0/5/3` (A07 active, W03 verified, R01/W02/F01 held), with no
ready-pending card. The prior `f4368883` canary, six-handle rebind and protocol
`PASS` are retained as historical no-credit evidence after PR #282 advanced
master. Exact next action: keep the clean owned `55a38ff` worktree, build once
with the selected exact toolchain, prove source/tree/binary/shim identity, run
the fresh no-credit canary and only then recreate/review the private holdout /
manifest packet. Preserve the root unknown path and foreign/stale topology;
do not allocate screening cells or claim A07 acceptance.
