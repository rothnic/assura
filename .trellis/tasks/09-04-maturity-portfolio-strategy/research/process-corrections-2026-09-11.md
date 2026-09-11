# Process corrections and continuation plan

Status: active, current source as-of the latest refresh
`origin/master=4560c710967d59993b9ea4f9b86613d446443f79`.
This record is a process and evidence route; it does not close A07, grant
screening credit, or authorize release, deployment, publication, invitation,
protection changes or a CI-infrastructure change.

## Current post-merge and candidate reconciliation — 2026-09-11 UTC (`origin/master=4560c710`)

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
  artifact hashes were recomputed and assertions pass. Scoped rereview is
  pending. Screening, allocation, credit and product acceptance remain false.
- Next owner/action: finish the scoped protocol rereview. Convert any vague
  concern into a concrete contract, location, failure scenario and smallest
  verification; repair only accepted metadata deltas, rerun affected checks and
  obtain scoped rereview. On protocol `PASS`, seek the separately authorized
  screening gate. In parallel, track the hosted watch-SIGINT failure as a
  distinct diagnostic; do not weaken performance or cancellation gates and do
  not merge work whose applicable checks or current-base proof are unresolved.

## Current post-merge reconciliation — 2026-09-11 UTC (`origin/master=40f1155c`)

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
