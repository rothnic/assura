# Execution recovery plan

Status: active continuation route, 2026-09-10 UTC. The latest post-merge
refresh resolved `origin/master=8c198dcb4c94095e7db1b017908eaadb813cb9e7`
after PR #258; this pointer is an as-of checkpoint and must be refreshed before
use. PR #258 merged the reviewed A07 candidate-freeze documentation/process
correction from `dba4e33df4a3bbdb1d52a0846aa0c9808268cbfc`, based on
`c34f917866e45cc122ec07412fa0c630d460f663`; its merged tree matches and it
changed no product or acceptance state. The earlier `c34f9178` candidate
checkpoint is historical. The latest diagnostic is the bounded R01 raw-log recovery recorded at candidate base
`c1202af` and merged as process evidence; it does not close R01 or authorize a
retry. This is a plan and audit, not evidence that product cards passed. The
supported runtime goal remains the coordinator; process corrections are
merged separately from product card slices. Product changes stay in their
separately owned card slices.

## Outcome and success gates

The PRD's outcome remains dependable repository conventions and agent-assisted
setup, with credible portfolio evidence. Work is successful when the selected
slice's observable contract passes, independent findings are resolved, required
local/hosted proof covers current source, integration is verified, and owned
branches close cleanly. A card is done only when its full packet outcome exists.

For A07 this retains screening, untouched holdout and final acceptance as
separate allocations; final acceptance is 10 runs per stack, at least 9/10 per
stack, zero destructive overwrites and zero critical misses. Follow-up feature
proof and full evaluator dimensions remain mandatory. CI, generated files and
partial checks cannot substitute. R06/publication/pilots retain their separate
authority and observed-outcome requirements.

## Live post-merge continuation checkpoint — 2026-09-10 UTC (`8c198dcb`)

- Owner/phase: process coordinator / `post-merge-reconcile`. PR #258 merged
  the independently reviewed process candidate `dba4e33` (based on `c34f917`)
  as `8c198dc`; merged-tree equality passed. The applicable Documentation,
  CI, Evidence Gates, Security Scope and GitGuardian checks passed. Product,
  Rust, performance, release, installer and website jobs were scope-skipped
  and remain non-applicable, not acceptance proof.
- The revision-pinned ledger at `origin/master=8c198dc` still has 32 items,
  zero ready pending, five unfinished and three held: A07 active, W03
  verified, and R01/W02/F01 separately held. The independent process review
  rejected a whole-goal stop. A07's next owned action is the private
  exactly-two-condition manifest, supplied-input receipt, blinded mapping,
  complete 30-cell matrix and isolated protocol-review `PASS`.
- The candidate-freeze checkout, branch and remote ref were removed after
  clean-status, `git diff --check`, merged-tree and reachability proofs. The
  topology report passed; strict mode remains nonzero only for preserved
  root/user dirt, the unrelated dirty worktree, stale/prunable and unreadable
  registrations, and historical unmerged goal branches. No broad prune or
  unrelated cleanup was performed.
- This reconciliation creates no screening, holdout or acceptance credit. On
  the next continuation, refresh source and ledger again, then have the A07
  acceptance coordinator validate the private protocol. Only after a redacted
  `PASS` may a newly bound identity and no-credit canary run.

## Historical candidate-freeze checkpoint — 2026-09-10 UTC (`c34f9178`, superseded by `8c198dcb`)

- Owner/phase: A07 acceptance coordinator / `candidate-freeze`. The clean
  current-master checkout resolved the ledger to 32 items, zero ready pending,
  five unfinished and three held: A07 active, W03 verified, and R01/W02/F01
  separately held. No pending card supersedes the active A07 route.
- The exact c34 candidate built locally with Rust/Cargo `1.94.1`; the release
  binary reports `assura 0.4.0`, is not a symlink, and hashes to
  `95c93052bd1993566d9f8209bdba5625c2b39a19287ed6358d710015d2ac59ff`.
  A `zsh -lic` identity check matched `command -v`, version and SHA. This is
  no-credit preparation, not a canary or product result.
- The six valid private holdouts remain frozen and the disqualified raw-hook
  draft remains excluded. The required private two-condition manifest,
  supplied-input proof, blinded mapping, complete 30-cell matrix and isolated
  protocol-review `PASS` do not yet exist. A07 therefore stays active with
  zero screening/holdout/final-acceptance credit; historical run names are not
  condition definitions.
- The supported product-input surface was audited (`--recipe-file`, bundled
  `--recipe`, and onboarding `--content-template`). The smallest next action
  is to select and document two concrete values for one of those inputs, prove
  receipt to the initializer, and obtain the scoped private protocol review;
  no value is invented by this checkpoint. Then refresh source identity and
  run the no-credit canary before any allocation.
- The measured `vps` host is not selected for this build: it has 16 CPUs and
  about 43 GiB available memory but 94% root-disk use (about 20 GiB free),
  nightly Rust 1.95, and no Bun; the `vps-dev` alias is unresolved. Remote
  execution remains optional and must use the exact-toolchain bundle procedure
  only after fresh disk/headroom checks.

## Historical post-merge recovery checkpoint — 2026-09-10 UTC (`origin/master=a819c0c`, superseded by c34f9178)

PR #256 merged the independently reviewed R01 raw-log recovery record as
`a819c0cd2e8e9fdf14a3641cc76f0119ceb9d2bc` from candidate `c950fe4`, based on
`c1202af`. The public macOS job log was retrieved with
`gh run view 34090768850 --job 101643647551 --log`, exited `0`, and contained
2,225 lines. It confirms sequence-2 `full_rescan_event` but has no raw
callback paths/kinds, rescan flag or config-generation fields. The missing
causal fields remain the R01 contract gap; no retry, filter, threshold, loop
or product change was made, and R01 remains held for retained raw trace or a
specific maintainer native-readiness decision.

The fresh revision-pinned ledger at this merge is 32 items with
`ready_pending=0`, five unfinished and three held: A07 active, W03 verified,
and R01/W02/F01 held. The private A07 metadata audit still lacks the
exactly-two-condition manifest, supplied-input proof, blinded mapping,
complete 30-cell matrix and isolated protocol-review `PASS`; no screening,
holdout or final-acceptance credit exists. No A07 worker or review handle is
live. The next real action is the A07 acceptance coordinator's private
six-holdout/manifest validation and isolated protocol review; only after a
redacted `PASS` may a fresh current-master candidate-bound canary run.

The process slice's owned worktree and branch were removed after merged
reachability. Its topology report passed; strict mode remains nonzero only
for preserved root/user dirt, unrelated dirty work, stale registrations and
historical goal branches. Keep those outside ownership. W03 publication, W02
Cloudflare work and F01 participant outreach remain separate authority-held
actions.

## Historical post-merge gate-order checkpoint (superseded by PR #256) — 2026-09-10 UTC (`origin/master=7a775371`)

PR #254 merged the independently reviewed process-only A07 gate-order
correction as `7a7753713319d01c9b3b9966cb4028e4929e8df1` from candidate
`6a6adca5f92700a7cba71c05e5d9a2d6f2271aa5`, based on `a14cb02`. Candidate and
merge trees match; required local/hosted process gates passed, while
scope-skipped product/Rust/performance/release/installer jobs remain
non-applicable rather than passing evidence. The owned branch/worktree/ref
were removed after clean closure.

A fresh checkout at this revision reran the ledger and active A07 gate-order
assertion. The ledger remains 32 items, zero ready pending, five unfinished
and three held: A07 is active, W03 is verified, and R01/W02/F01 retain their
card-level holds. The next resume must fetch, rerun the ledger, freeze current
identity, confirm six holdouts, obtain private manifest/protocol `PASS`, then
run the fresh no-credit canary; no screening or acceptance credit is created.

## Post-merge refresh checkpoint — 2026-09-10 UTC (`origin/master=a14cb02`)

PR #253 merged the independently reviewed process-only A07 refresh-before-use
correction as `a14cb02332921e3a06b184381720319767a2b7aa` (candidate
`0f8e012ca4ef2af61c875310e344a1416d92a42c`). The candidate and merge trees
are identical, required local and hosted process gates passed, and the owned
branch/worktree were removed after verification. This is a dated checkpoint,
not a live pointer: every continuation must fetch `origin/master` and rerun
the ledger before canary binding.

The fresh ledger at this revision remains 32 items with `ready_pending=0`,
five unfinished and three held: A07 is active, W03 is verified, and R01/W02/F01
retain their card-level holds. The metadata-only private A07 audit found the
mode-700 owner-controlled store, a 204-line manifest without discoverable
condition/product-input/supplied-input/mapping/matrix/protocol-review fields,
17 private run objects, and no isolated protocol-review artifact. No screening,
holdout or final-acceptance credit is created by these observations.

No live A07 worker, initializer, evaluator or review handle exists. On the next
resume, first fetch `origin/master`, rerun the revision-pinned ledger and
freeze the candidate identity, then confirm the six frozen holdouts. The exact
private action after that refresh is to create or locate the versioned
exactly-two-condition manifest, prove supplied inputs and blinded mapping,
validate the complete 30-cell matrix, and obtain an isolated protocol review
`PASS`. Only then may the coordinator run a fresh no-credit canary against the
frozen identity. Do not invent private values, reuse historical run names as
conditions, or launch screening early. R01's raw trace or maintainer decision,
W02's external approval and F01's participant authorization remain independent
held actions.

## Post-merge process checkpoint — 2026-09-10 UTC (`origin/master=062f6c3`)

PR #251 merged the reviewed process-only candidate `e777ee7` (based on
`1bd78cc`) as `062f6c39d15babc8b12299863576a29febda5dd5`; its merged tree
matches the reviewed candidate. The recovery route now points to the raw R01
callback trace or a maintainer native-readiness decision, and the continuation
skill enforces explicit command identity before accepting check/test/build
evidence. The revision-pinned ledger still reports 32 items,
`ready_pending=0`, five unfinished and three held: A07 is active, W03 is
verified, and R01/W02/F01 retain their card-level holds. No product, card,
evaluator, threshold, allocation, release, deployment, publication or
invitation state changed. The merged worktree and branch were removed after
clean closure; the final topology report is 0 and strict is 1 only for
preserved external/user conditions and historical registrations. The next
authorized route remains A07's private manifest/protocol review; the R01
diagnostic below is candidate-base evidence and must not be treated as a new
current-master result without a refresh.

## Candidate-base diagnostic observation — 2026-09-10 UTC (`origin/master=1bd78cc`; superseded by the post-merge checkpoint)

Owner/phase: process coordinator / R01 diagnostic preparation. A clean,
detached current-master checkout at `1bd78cc3705e278d6502637463873de4ad1c2aab`
ran the exact external-config watch test from its own cwd. The corrected
command was `cargo test --target-dir /Users/nroth/workspace/assura/target
--test watch_cli watch_observes_an_explicit_config_outside_the_project --
--exact --nocapture`; it exited `0` after 195 seconds on Darwin x86_64 with
Rust/Cargo `1.94.1`. It emitted only the expected config-triggered sequence-2
warm-full report (`coalesced_events=4`, no fallback) and did not reproduce the
historical unexpected filesystem event. A prior attempt that accidentally ran
from the dirty strategy root was explicitly discarded as no evidence.

This is a current-host reproduction result, not native-readiness closure. No
hosted diagnostic run, retry, filter, threshold or product edit was made. The
result is already preserved in `research/evidence/R01.md` and
`research/progress.md`. The R01 owner still needs the raw callback
paths/kinds/rescan/config-generation trace for
run34090768850/job101643647551, or a maintainer decision on an alternative
native-readiness contract. The next independent action is that trace or
decision; avoid speculative debounce/loop changes.

## Historical continuation checkpoint — 2026-09-10 UTC (`origin/master=2eda17e`; superseded by the candidate-base observation and post-merge checkpoint)

Owner/phase: process coordinator / `investigate-prepare`; the next card action
belongs to the A07 acceptance coordinator. A fresh fetch resolved
`origin/master` to the PR #249 merge
`2eda17e82d9dab12338805479a33a5774560451f`. The reviewed candidate
`5f17f7f` was based on `755c28d`; its merged tree matches the candidate. This
process-only correction adds the continuation-control routing reference and
reconciles current source labels; it changes no product, evaluator,
threshold, allocation, release, deployment, publication or invitation state.

Session/live handle: none for A07; the metadata audit is complete and no
initializer, evaluator, CI or review process is running. Worktree/branch:
private A07 evidence lane / no shared checkout; the process candidate is the
separate clean `docs/continuation-control-route` branch. Trigger/proof:
`ready_pending=0` plus the redacted private-manifest metadata audit below;
the exact next action is the coordinator's isolated manifest/protocol review.

The revision-pinned ledger reports 32 items, `ready_pending=0`, five
unfinished and three held: A07 is `active`, W03 is `verified`, and R01/W02/F01
retain their named holds. No pending card is executable. A redacted,
metadata-only audit of the private A07 store still finds seven layout
directories (six frozen-layout entries and one disqualified draft), a draft
manifest without discoverable condition, supplied-input, mapping, matrix or
protocol-review fields, and no isolated protocol-review artifact. The 17
recorded run objects are not reclassified by filename or historical outcome;
screening, holdout and final-acceptance credit remain zero.

Independent impasse/process review recorded finding `A07-MANIFEST-04`:
the manifest contract is not executable until the coordinator privately
provides exactly two named conditions that differ in one product-input
variable, supplied-input proof, blinded mapping, six frozen holdouts and a
complete 30-cell matrix, then obtains an isolated protocol-review `PASS`.
This is a held card action, not a whole-goal stop. The smallest resolution is
that private manifest/protocol disposition; after `PASS`, fetch again, bind a
fresh candidate identity and run the no-credit canary before any allocation.

The serialized capacity probe found `vps` reachable with 16 CPUs, low load,
about 42.7 GiB available memory and about 23 GiB free disk at 94% use, but
nightly Rust 1.95, pnpm 10.29.3, no Bun, and unrelated active processes. No
heavy job ran. Local cheap gates remain first; remote work is permitted only
through the exact-toolchain bundle procedure after disk/headroom checks, one
job at a time. Hosted platform and performance checks remain final proof.

Topology while this owned candidate is present remains `worktrees=35 dirty=2 prunable=3 unreadable=1
goal_branches=13 unmerged_goal=9`; report exits 0 and strict exits 1 for the
preserved root/user dirt, external dirty worktree, stale registrations and
historical goal refs. None is owned by this slice. Before handoff, rerun both
topology modes and remove only this slice's clean merged worktree/ref.

Continuation route: keep the supported goal active; do not create a duplicate
goal or declare the empty ready set blocked. The A07 coordinator privately
creates/locates the manifest, obtains protocol-review `PASS`, then the
coordinator refreshes source and routes the canary. Independently authorized
work remains bounded topology inventory, R01 diagnostic preparation, W02 local
preparation without deployment, and W03 evidence maintenance without
publication.

## Historical source-pointer reconciliation — 2026-09-10 UTC (`origin/master=755c28d`; superseded by PR #249)

Owner/phase: process coordinator / `reconcile-handoff`. A fresh fetch resolved
`origin/master` to the full merge SHA
`755c28ded66d1f2d82b38d27633be83a0233f30e` after PR #248. The reviewed
candidate `89a0430ba9f4bbf141e717ed78de72b41c5ab3b6` was based on
`6ed43c3c63fab7b60a86f1d587c067c9b04ded93`; the merged tree matches that
candidate. This process-only merge adds the source-pointer lifecycle rule and
changes no product, evaluator, threshold, allocation, release, deployment,
publication or invitation state.

The immutable ledger at this source reports 32 items, zero ready pending, five
unfinished and three held: A07 active, W03 verified, and R01/W02/F01 held.
The redacted private A07 metadata audit still finds seven layout directories,
including six frozen-layout entries and one disqualified raw-hook draft; the
draft manifest has no discoverable condition rows, supplied-input mapping,
complete matrix or isolated protocol-review artifact. No screening, holdout or
final-acceptance credit exists.

At this checkpoint the route was the A07 coordinator's private exactly-two-
condition manifest and isolated protocol-review `PASS`; after that disposition,
refresh the candidate identity against the then-current master and run the
no-credit canary. The source-pointer lifecycle makes this checkpoint an
as-of record. PR #249 subsequently moved the current source to `2eda17e`; every
resume must fetch and rerun the ledger, and later master SHAs supersede this
pointer without an evidence-only chase when labels cannot misroute.

## Historical post-merge continuation checkpoint — 2026-09-10 UTC (`origin/master=6ed43c3`; superseded by PR #248)

Owner/phase: process coordinator / `reconcile-handoff` complete. PR #247
merged the reviewed pointer correction as
`6ed43c3c63fab7b60a86f1d587c067c9b04ded93`; its tree matches candidate
`577b6d63030aab338238088c85cfa7c7750baeb2` and the fetched parent is the
reviewed base `d62dd40f0d915e693db25cdea2fb29d1000e9b50`. The post-merge
ledger remains 32 items with zero ready pending, five unfinished and three
held: A07 active, W03 verified, and R01/W02/F01 held.

The pre-merge `d62dd40` readiness audit below is a historical candidate-base
record, and this `6ed43c3` checkpoint is superseded by the current `2eda17e`
reconciliation above. At the next resume, fetch `origin/master` again before
using any checkpoint. The active route remains A07's private exactly-two-condition
manifest, isolated protocol-review `PASS`, then a fresh current-master
candidate-bound canary; no screening, holdout or final-acceptance credit is
created by this reconciliation.

## Historical candidate-base refresh — 2026-09-10 UTC (`origin/master=d62dd40`; superseded by PR #247)

Owner/phase: process coordinator / `investigate-prepare`. A fresh fetch and
revision-pinned ledger make `d62dd40f0d915e693db25cdea2fb29d1000e9b50` the
live routing source: 32 items, zero ready pending, five unfinished and three
held; A07 is active, W03 verified, and R01/W02/F01 retain their named holds.
This process refresh changes no product, evaluator, threshold, release,
deployment, publication or invitation state.

The redacted private A07 audit found six valid frozen holdout layouts and one
disqualified construction draft, but no discoverable exactly-two-condition
record, supplied-input mapping, complete 30-cell matrix or isolated
protocol-review `PASS`. The next action is to obtain that private protocol
disposition; only then may the coordinator refresh a candidate identity against
`d62dd40` and run the no-credit canary. The checkpoint below is historical and
must not route work from its older hashes.

## Historical last verified checkpoint — 2026-09-10 UTC (after PR #245 / `27ef54d`; superseded)

The last verified integration point is `origin/master=27ef54d489847e41e5907f7c74f870a2391a7dae`,
the merge of reviewed PR #245 final head
`391178676931ba935ad0058fce0bc55e101b3641` from base
`2902a073f39f8e8a47a9658a6358791c3e7f4655`; exact ancestry exited `0`.
Documentation Scope, CI Scope, Security Scope, Evidence Gates and GitGuardian
passed on the exact head. Product, Rust, performance, release, installer and
website jobs were scope-skipped and remain non-applicable.

The owned reconciliation worktree and branch were cleanly removed after merge;
no uncommitted or abandoned process work remains from this slice. Final
topology was `worktrees=34 dirty=2 prunable=3 unreadable=1 goal_branches=13
unmerged_goal=9`, with report exit `0` and strict exit `1`; preserved root/
external dirt and historical registrations remain outside ownership. The
immutable ledger at `27ef54d` is 32 items with zero ready pending, five
unfinished and three held: A07 active, W03 verified, R01/W02/F01 held.

This is the last verified checkpoint, not a permanent pin. At every resume
fetch `origin/master` and rerun the ledger before selecting work. The next
authorized route is A07's private exactly-two-condition and six-holdout
manifest, isolated protocol-review `PASS`, and a fresh candidate-bound canary
against the refreshed master. No card, evaluator, threshold, release,
deployment, publication or invitation state is changed by this process slice.

## Historical checkpoint — 2026-09-10 UTC (PR #244 / `2902a07`; superseded)

The historical reset refreshed `origin/master=2902a073f39f8e8a47a9658a6358791c3e7f4655`,
the merge of reviewed PR #244 final head `03a9b01eb1a5742b0cc84373e53f3f9aed8d0f1e`
from base `80d2d9a7fea55c1413f7e50f0873306049a65a48`; exact ancestry was
verified with `git merge-base --is-ancestor` exit `0`. Documentation Scope,
CI Scope, Security Scope, Evidence Gates and GitGuardian passed on the exact
head. All product, Rust, performance, release, installer and website jobs
were scope-skipped and remain non-applicable. The first docs attempt's missing
Astro dependency and the frozen-install/identical retry are retained as
environment evidence; the retry passed and built 48 pages.

The owned process worktree and `docs/train-continuation-control` refs were
removed only after clean status and merged reachability. The root unknown file,
external dirty worktree, three prunable registrations, one unreadable
registration and historical unmerged goal refs remain preserved. At this
revision the ledger still reports 32 items, zero ready pending, five
unfinished and three held; A07 is active, W03 verified, and R01/W02/F01 retain
their exact holds. This slice changes no card state or external authority.

**Continuation decision (historical).** Keep the supported runtime goal active and carry the
process coordinator's next action into a clean worktree: privately name
and validate A07's exactly two product-input conditions and six holdouts, obtain
an isolated protocol-review `PASS`, then run a fresh candidate-bound canary
against `2902a07`. Do not launch screening or infer release, deployment,
publication or invitation authority from this documentation slice.

## Historical checkpoint — 2026-09-10 UTC (after PR #243; superseded)

This was the historical read-only reset. The root checkout was an older strategy
branch and has one unknown untracked file,
`.trellis/tasks/09-04-maturity-portfolio-strategy/research/a04-host-status-doctor-permission-gap.md`.
It is preserved untouched; all edits for this checkpoint use a clean external
worktree. `git fetch origin master` succeeded and resolved
`origin/master=80d2d9a7fea55c1413f7e50f0873306049a65a48`, the merge of PR #243.
The revision-pinned ledger helper reports `items=32`, `ready_pending=0`,
`unfinished=5`, `held=3`: A07 is `active`; R01, W02 and F01 retain named holds;
W03 is `verified`; no pending card is executable at this revision. This is
routing evidence only and does not prove owner liveness, card acceptance or
merge authority.

The release refresh found no `origin/release` branch (`git fetch origin release`
reported that the remote ref does not exist). The remote tags currently reach
`v0.3.0`, while the current source and `/usr/local/bin/assura` report `0.4.0`;
neither a tag nor the installed binary is public-install or release proof. Open
PR state was refreshed: #194 has a required `Performance Report` failure after
6m53s despite other checks passing; #195 targets the unmerged #194 branch and
has only GitGuardian evidence; #187 is not current and its Cloudflare build
check is coupled to production; #142 retains Alpine/macOS/Windows failures.
Skipped or scope-only checks are not counted as passes.

The serialized host probe could not resolve `vps-dev`. The configured `vps`
host is reachable with 16 CPUs, 43,750 MiB available memory, load below 1,
and 23,217,420 KiB free at 94% disk use; it has nightly Rust 1.95,
Node 22.22.1 and pnpm 10.29.3. No heavy job ran. This is capacity evidence,
not a speedup claim: use the isolated bundle procedure only after confirming
the exact CI toolchain and target-size headroom, one job at a time, and retain
hosted platform/performance checks as final proof. Do not delete caches or
other work to create disk space.

The topology report exited 0 and strict exited 1 with
`base=origin/master worktrees=34 dirty=2 prunable=3 unreadable=1
goal_branches=13 unmerged_goal=9`. The dirty root and external worktree,
unreadable registration, prunable registrations and historical unmerged goal
branches are outside this slice's ownership. `git worktree prune --dry-run`
was read-only; no prune, reset, stash, deletion or ownership reassignment was
performed.

**Continuation decision.** Keep the existing runtime goal active; do not create
a replacement goal or stop at this status. The process coordinator owns this
checkpoint (phase `investigate-prepare`). The next real action is for the A07
acceptance coordinator to create and privately validate exactly two product
conditions and six holdouts, obtain an isolated protocol-review `PASS`, and
then run a fresh candidate-bound canary against `80d2d9a`. Until that private
input and review exist, no screening cell or pending release card is
executable. R01's missing raw macOS trace/native-readiness decision, W02's
Cloudflare approval, F01's participant authorization, and W03's publication
remain separate holds; independently prepared process/topology work may
continue without changing any product threshold or authority boundary.

## Corrected baseline (historical snapshot; refresh before action)

The details in this section describe the recovery snapshot that preceded the
current integration. That historical snapshot used `origin/master=922d7f0`;
the later pre-#235 checkpoint was `origin/master=1f9ebf2`. Always refresh the
live ref and use the newest progress entry and card evidence for routing rather
than copying either historical SHA or the PR state below.

Historical integration source: `da773bce8315cbb9e69941f0fe89d0a91e42829d`.
Master Rust CI `34310994162`, Documentation `34310994191` and Security
`34310994120` report success at that SHA. PR #227 merged; its scoped init
guidance contract is not proof of A07 success. PR #196 is already merged and
is an A04 diagnostic, while R03 completion is recorded through PR #197.
The old heartbeat's PR identity and failure narrative are unsuitable for
candidate selection. Preserve failed historical evidence without reviving it.

The previous thread review read `docs/maturity-portfolio-strategy` at
`f3a8f66`, not the current ledger. Its claim that only B00 was recorded done
was therefore stale. Current master records A07 active and R03/A04/A05/A06
done. R01, W02 and F01 retain specific held outcomes; W03 is verified with
publication outstanding. Reconcile packet-level contradictions before promotion
(including A06's R01 dependency), using source and reviewed evidence.

The historical inventory recorded unknown root dirt, a clean
`goal/a07-composed-init` at `da773bc`, and other clean branches that required
ownership classification. That pre-#236 checkpoint had removed only the five
verified-merged goal worktrees/refs and the superseded detached build; it keeps
the unknown root dirt, the pre-existing prunable registration, and the clean
`922d7f0` candidate archive. The global topology report still exits 128 on the
preserved missing worktree; record incomplete coverage and use the read-only
fallback before proposing a targeted audit repair.

## Prior process checkpoint — 2026-09-10 (PR #236; superseded)

Refresh evidence identifies `origin/master=35cce811532c793f9446d13b8ef42f6390d370bf`,
the merge of PR #236 from reviewed head
`2008f3fe7eecb6806492490511b3a8d6a47c4ab1`. That process correction makes the
A07 manifest, privacy boundary, layered context routing and live action
ordering executable. Its scoped local and hosted gates passed; skipped
product/Rust/performance/release jobs remain explicitly skipped and are not
outcome proof.

A07 remains `active` with zero screening, holdout or final-acceptance credit.
The earlier `77b41fe` canary is historical and no-credit. The next exact route
is: refresh current-master identity and freeze holdouts; create and validate
the private two-condition manifest; obtain an isolated protocol-review `PASS`;
run a fresh candidate-bound canary against `35cce81`; then launch the 30-cell
screen only if every preceding gate passes. The 18-run untouched holdout,
follow-up feature checks and final ten-per-stack at-least-9/10 threshold remain
mandatory.

The root unknown file and known missing-gitdir registration remain preserved.
The owned PR #236 worktree and branch were clean, merged, verified reachable
from `origin/master`, and removed. Other goal refs/worktrees are retained only
as classified historical archives or user-owned work; do not delete them by
pattern. This checkpoint is retained for provenance; it is not the current
integration baseline.

## Historical checkpoint — 2026-09-10 (post-PR #237 refresh; superseded)

A fresh read-only fetch identifies `origin/master=8cabc53658530a239c00a0c55cbae9b050ad74ca`,
the merge of PR #237 from independently reviewed head
`19c5941a8717f05f305032be56080f626fdd06b5`. PR #237 reconciles the recovery
plan/evidence and current task metadata; it does not change product behavior,
the backlog's acceptance thresholds, or A07 allocation credit. The prior
`35cce811` process checkpoint and `5329abd` candidate baseline are historical
after this merge.

A07 remains `active` with zero screening, holdout, or final-acceptance credit.
The current route is: freeze the two private product-input conditions and six
holdouts; validate the private manifest and complete 30-cell matrix; obtain an
isolated protocol-review `PASS`; build and identity-check a fresh candidate
bound to `8cabc536`; then screen only if every preceding gate passes. The
earlier `77b41fe` and `5329abd` canaries remain no-credit historical evidence.
The 18-run untouched holdout, follow-up feature checks, and final
ten-per-stack at-least-9/10 threshold remain mandatory.

The root unknown file, the missing-gitdir registration, and the two clean
detached evidence archives remain preserved. The current pointer-reconciliation
slice is the sole new owned worktree; all other branches and worktrees remain
classified historical or user-owned and must not be deleted by pattern.

## Ordered recovery slices after current reconciliation

| Priority / owner | Action | Exit proof / next action |
| --- | --- | --- |
| 1 / process coordinator | Keep the current-master pointer and continuation route reconciled across the recovery plan, A07 routing evidence, continuation prompt, task metadata and goal-execution skill | PR #256 merged the bounded R01 raw-log recovery record at `a819c0c`; fetch and rerun the ledger before the next phase |
| 2 / train coordinator | Apply the current A07 evidence and manifest contract before any allocation; keep private values outside public artifacts | Refresh `origin/master`, rerun the ledger and freeze candidate identity; confirm holdouts; create/validate the private manifest and obtain protocol-review `PASS`; then run the canary against that frozen identity |
| 3 / A07 implementation owner | Test explicit composed init proposal against public semantics; plain init remains config-only; preserve conflicts and honest runtime permission reporting | Focused negative/positive controls, review, required final-source local/hosted gates; merge slice only if its own goals pass; rerun blinded protocol before A07 completion |
| 4 / independent R01 investigator | Read latest R01 packet/evidence and rejected diagnostics; determine missing causal observable before spending another platform run | Concrete hypothesis, distinguishing observation and reviewed method amendment; do not repeat prior unchanged diagnostic or invent native readiness |
| 5 / topology owner | Inventory existing goal branches, live owners and base reachability; repair audit parser as a separately tested slice if needed | Complete report; merged clean branches removed by exact identity or verified archive; unknown work preserved |
| 6 / release/portfolio coordinator | Prepare R06/W02/W03 and later cards against actual dependencies and observed trigger coupling | Complete authorized local preparation; specific authority remains attached only to publication/deployment/invitation action |

Priorities express dependency and urgency, not compulsory single-lane execution.
R01 investigation/topology preparation can run while A07 review or CI waits,
with explicit owners and resource capacity. Do not spawn a worker just to poll.
This process slice does not implement A07, performance changes, releases or CI
policy changes. Those require their own recorded acceptance and review.

## Instruction audit and corrections

| Observed defect | Durable correction |
| --- | --- |
| Old checkout mistaken for current canonical ledger | Goal skill requires revision comparison and current-base task reads |
| Long AGENTS includes stale Rust 1.70, generic examples and setup advice | Shared router points to manifest, skills and actual specs; retain universal ownership/review/authority rules and managed Trellis block |
| Goal skill only routes `docs/goals`, while train lives in Trellis | Add Trellis routing and checkpoint/merge contract references |
| Pending-only queue selection misses unfinished candidates | Inspect active/implemented/verified candidates first, then ready pending rows |
| Strong prose but repeated idle heartbeat replies | Resume exact checkpoint; after two unchanged wakes inspect liveness/state and choose a diagnostic; notifications stay quiet when appropriate |
| Environment recipe repeats unrelated suites and chained cwd errors | Resume original applicable command, use root-relative package setup and validation matrix |
| VPS diff helper destroys/reuses labeled directories and lacks clean-commit identity | Use isolated exact-commit bundle procedure; no automatic helper promotion |
| Evidence-policy check mistaken for outcome proof | Require card consumer/negative evidence separately from `cargo xtask evidence` |
| Initializer inherited an ambient binary or coordinator/evaluator context | Launch a fresh one-shot child from a source-only fixture with a fixed public prompt and minimal login-shell-safe `PATH`; prove identity and context separation before allocation |

Read layers: AGENTS → goal skill → selected card/current checkpoint → required
phase reference → exact source/log. Do not load all historical packets or
evaluator internals into implementers. Current facts live here and in card
evidence; reusable rules live in skills. No second scheduler/ledger is added.

## Validation efficiency and VPS decision

The latest serialized SSH audit still cannot resolve `vps-dev`; configured
`vps` reached a 16-CPU Linux host with 62,787 MiB total / 43,750 MiB available
memory, load below 1, and 23,217,420 KiB free at 94% disk use. Default Rust is
nightly 1.95; Node 22.22.1 and pnpm 10.29.3 are present. This shows available
CPU/RAM, not measured speedup or validated build capacity. No heavy remote job
ran. The 94% disk condition is an explicit preflight hold for large builds.

Use the goal skill's validation matrix and local-build VPS procedure. Start
with one isolated job after checking target-size headroom and explicit matching
toolchain. Keep platform-specific CI and idle-host benchmark controls. Never
clean others' artifacts to obtain disk space. Capture three prepared candidate
measurements before proposing cache/concurrency/runner changes.

`cargo xtask pr` includes `fast`; avoid unchanged back-to-back runs. Existing
hosted scope reuse verifies successful prior jobs and falls back to full checks;
preserve it. `cargo xtask evidence` checks repository evidence policy, not card
acceptance. Do not change workflow coverage or performance thresholds here.

For a fresh documentation worktree, preflight `website/node_modules` and the
locked package manager before running `cargo xtask docs`. If the documented
build tool is absent, run `pnpm --dir website install --frozen-lockfile` in the
disposable worktree and rerun the same gate; a missing tool is a failed
environment precondition, never a skipped or passing docs result.

For A07, after refreshing current source identity, freezing holdouts, and
obtaining an isolated protocol-review `PASS` for the private manifest, perform
the cheap identity/context canary before native tests or a screening batch. One
candidate build may support multiple diagnostics only when its source, binary,
fixture and invocation remain unchanged; a child that sees private evaluator
material or a global Assura install is invalid and must not be repeated
unchanged. A safety-guard rejection is an operational observation, not
permission to weaken the guard; use a disposable fixture and preserve the
rejection in evidence.

## Verification and continuation

Process review must exercise: stale checkout vs current ledger; local green
with failed required hosted performance; live CI timeout; process-only scope;
VPS nightly/low disk; clean branches with unknown dirt elsewhere; passing slice
while A07 acceptance fails. Expected decisions follow the execution contract.
Record actual reviewer decisions and limitations in `recovery-evidence.md`.

Next action: keep the supported runtime goal active and follow the refreshed
checkpoint's identity/holdout → manifest validation → isolated protocol review
`PASS` → fresh current-master canary against the refreshed `origin/master` →
screening sequence. R01's retained raw trace/maintainer decision, W02's
approval, W03's publication and F01's participant authorization remain
separate held actions.
Do not replace or complete an unfinished goal to repair a status mismatch, and
do not treat this process reconciliation as A07 acceptance.
