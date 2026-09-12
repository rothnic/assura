# Copy-paste prompt for an execution agent

Current recovery route: [recovery-plan.md](recovery-plan.md). Resolve this task
at refreshed `origin/master`, then read the repository's `assura-goal-execution`
skill for the execution contract and validation matrix. Absolute paths below
identify the task, not the authoritative revision of an old checkout.
Load [`execution-control-plane.md`](../../../../.agents/skills/assura-goal-execution/references/execution-control-plane.md)
only when a state transition, compaction, expensive gate, review, integration
or handoff requires its detailed checkpoint and decision rules.

Resume the existing supported runtime goal; do not create a replacement goal
for a card boundary or a compaction. Refresh the `release` ref and tags as
separate availability facts, and record a missing release branch rather than
turning it into release proof.
After every merge, refresh `origin/master` and reconcile the task pointer,
ledger and owned topology before selecting or handing off; a clean merge is a
phase transition, not completion of the runtime goal.

Current post-merge checkpoint (2026-09-12, refresh before use):
`origin/master=e1c9b78216b5736abd24fbd165ed7e1e383b1744` with tree
`9d57888eec65d892dcc2a65b829369dd90278637`; PR #313 merged reviewed head
`a134fc6815fc76730926d9f8ded57f16a8740e74` from base
`a6ed20f532d93f830099e9095798a09515fead0b`. Exact-head Documentation Scope,
CI Scope, Evidence Gates, Security Scope and GitGuardian passed. Merge-SHA
Documentation (`34675049056`), Security Audit (`34675049038`) and Rust CI
(`34675049055`) passed, including CI Scope (`103503298569`) and Evidence Gates
(`103503318127`); product/Rust/performance/release jobs were scope-skipped and
remain non-proof.

The revision-pinned ledger remains `items=32; ready_pending=0; unfinished=5;
held=3` (A07 active, W03 verified, R01/W02/F01 held). The owned a6ed branch
and worktree were verified clean and ancestor-reachable, then removed. Its
candidate, canaries, packet, validator and isolated `PASS_NO_CREDIT` review
are candidate-base/no-credit after the merge; the immutable review and
coherence corrections remain evidence. No screening, allocation, credit,
acceptance, release, deployment, publication, invitation or protection
authority exists. Preserve unknown/foreign/stale topology and unfavorable
evidence; the optional VPS lane remains held.

Next action: refresh release/tag, PR/CI, topology and the ledger again. Keep
the active A07 route ahead of empty pending rows; if no-credit preparation is
authorized, create one fresh exact-toolchain candidate from `e1c9b78` in a
clean owned worktree and run identity, sibling-free canary, current
holdout/manifest rebind and isolated protocol review in order. Any process
update requires independent review, applicable gates, current-master
integration and merge-SHA observation. Never reuse the a6ed packet or infer
product success from process metadata or skipped checks.

Historical candidate-bound checkpoint (2026-09-12, superseded by the post-merge source):
`origin/master=a6ed20f532d93f830099e9095798a09515fead0b` with tree
`bd399cd480dc7a31630740718dd8248a0c77bef5`; PR #312 merged reviewed head
`f43d1a0cc6cd868a78d4f68dbc97a52a90476abb`.
Its exact-head Documentation Scope, CI Scope, Evidence Gates, Security Scope
and GitGuardian checks passed; merge-SHA Documentation `34670911182`, Security
Audit `34670911121` and Rust CI `34670911124` passed for applicable scope.
Product/Rust/performance/release rows were scope-skipped and remain non-proof.
The revision-pinned ledger is `items=32; ready_pending=0; unfinished=5;
held=3` (A07 active, W03 verified, R01/W02/F01 held). The owned candidate
checkout is `/private/tmp/assura-a07-current-a6ed` on
`goal/a07-current-a6ed`; its exact Rust/Cargo `1.94.1` identity, command-help,
login-shell, wrong-target and wrong-root controls pass/reject as intended.

The candidate/shim SHA is
`1129d4498651fce2679264c5f2464931edd77e5e1bc0bcb3b02914ba1f42913c`.
Fresh sibling-free A/B canaries and post-exit evaluators exit `0` with all
seven dimensions passing and zero critical failures. The packet has six unique
holdouts, two stable conditions and 30 reserved cells; persisted validation is
`valid=true`, exit `0`, protocol `PASS_NO_CREDIT`. Independent rereview
resolved `A07-A6ED-VALIDATOR-001`, `A07-A6ED-REVIEW-TIME-002` and
`A07-A6ED-CANARY-STATE-003`, with coordinator pre/post hash finalization.
No screening, allocation, credit, acceptance, release, deployment,
publication, invitation or protection authority exists.

Next action: commit this goal/skill/evidence reconciliation, obtain independent
public process review and applicable gates, integrate only a reviewed
current-master candidate, observe configured merge-SHA workflows, then refresh
source/release/tag/PR/CI/topology and ledger. If source advances, classify the
packet candidate-base/no-credit and rebuild. Never stop at an empty queue,
skipped check or process evidence, and never start screening from
canary/protocol metadata without separate authority and the full A07 contract.

Historical post-merge checkpoint (2026-09-11, superseded by `eef7e1a`;
refresh before using only as history): `origin/master=66e0b7e75f644dc4d047853488daee4a1cd716a3`; PR #306
merged the reviewed pointer slice from base `dc031527` and left all product
and authority state unchanged.

Historical candidate-bound checkpoint (2026-09-11, superseded by `66e0b7e`;
refresh before using only as history): `origin/master=dc031527afd400be52dfd8fe9cfdabc7a6caa685`; PR #305
merged reviewed head `5363b4a5d2e39beed6087bea6a5f6fa2d2aaf17c` from base
`284e781`. Its applicable exact-head and merge-SHA Documentation, CI Scope,
Security Scope, Evidence Gates and GitGuardian checks passed; scope-skipped
product/performance/release rows remain non-proof. The revision-pinned ledger
was `items=32; ready_pending=0; unfinished=5; held=3` (A07 active, W03
verified, R01/W02/F01 held).

The clean candidate checkout `/private/tmp/assura-a07-current-reconcile-dc031`
uses exact Rust/Cargo `1.94.1` and source/tree/binary/shim identity
`dc031527`/`b68b66cc47f7e4081afd2dc21c1387de34b30c32`/
`3cdaf976d7da61043a5867cb17ffd789b92b484e26f1e7d3a7989bd25ce816cf`. Identity,
wrong-target and wrong-root controls pass/reject as intended. Two sibling-free
source-only canaries and full seven-dimension post-exit evaluators pass with
zero critical failures. The private packet
`/private/tmp/assura-a07-private-dc031` has six current holdouts, two
conditions and 30 reserved cells; its metadata-only validator is `valid=true`
with zero errors after correcting stale aliases. Independent protocol review
returned `PASS_NO_CREDIT` after correcting the stale contract digest and
construction hash. No screening, allocation, credit or product acceptance is
claimed.

Historical next action: reconcile the protocol-pass/no-credit evidence only in
a reviewed current-base slice; if source advances, classify the packet
candidate-base/no-credit and rebuild. Preserve the R01 macOS watch-SIGINT
failure and all other unfavorable evidence. Do not reuse this historical
checkpoint for screening or acceptance.

Historical current-source post-merge checkpoint (2026-09-11, superseded by
`dc031527`; refresh before use): `origin/master=284e78156370d49d8315f04391fcc74eaba3acb2`;
refresh before use): `origin/master=284e78156370d49d8315f04391fcc74eaba3acb2`;
PR #304 merged the reviewed post-merge reconciliation from head
`17fa9197ed79083be4b8ba0714851aeaae9bdbf3` on base `d228472`. Its applicable
Documentation, CI Scope, Security Scope, Evidence Gates and GitGuardian
checks passed after independent review `PASS`, the reviewed head tree equals
the merge tree, and the configured push-triggered Documentation
(`34651068660`), Rust CI (`34651068675`) and Security Audit (`34651068699`)
workflows completed at merge SHA `284e781` for applicable scope. Rust CI's CI
Scope `103433100783` and Evidence Gates `103433136825` passed; product/Rust/
performance/release and other scope-skipped rows were explicitly skipped by
scope and are not product proof. This is an as-of checkpoint, not a permanent
baseline. The revision-pinned ledger remains `items=32; ready_pending=0;
unfinished=5; held=3` (A07 active, W03 verified, R01/W02/F01 held). The
d228472 process checkpoint and candidate are now historical or candidate-base/
no-credit and must not be reused. If A07 preparation is authorized, build a
fresh candidate from `284e781`, then repeat identity, sibling-free no-credit
canary, packet rebind and isolated protocol `PASS` before separately
authorized screening. Preserve the earlier macOS watch-SIGINT hosted failure
as unfavorable diagnostic evidence: run `34615572565`, job `103316578631`,
failed at `tests/watch_cli.rs:196` with `watch did not stop after SIGINT`.

Read-only capacity evidence keeps remote execution held: `vps-dev` is not a
configured alias; configured `vps` has 16 CPUs, 61 GiB RAM, 20 GiB free on a
339 GiB root volume (95% used), and Rust/Cargo `1.95.0-nightly`. Do not launch
a heavy remote job without a fresh exact-toolchain, free-disk and owned-job
probe; remote Linux output never replaces platform or hosted proof.

Historical current-source reconciliation (2026-09-11, superseded by
`24a1966`; refresh before use): `origin/master=0dff804421c7563b08773eb75d9327fd0194db56`; PR #300
merged the reviewed current-checkpoint reconciliation from head
`f37c20e21ec94116c18dd591638fabf86163f947`. Its applicable Documentation, CI
Scope, Security Scope, Evidence Gates and GitGuardian checks passed, the
reviewed head tree equals the merge tree, and the configured push-triggered
Rust CI (`34638264948`), Documentation (`34638264944`) and Security Audit
(`34638264978`) workflows completed at merge SHA `0dff804` for applicable
scope. Product/Rust/performance/release rows were explicitly skipped by scope
and are not product proof. This is an as-of checkpoint, not a permanent
baseline. The revision-pinned ledger remains `items=32; ready_pending=0;
unfinished=5; held=3` (A07 active, W03 verified, R01/W02/F01 held). A fresh
0dff804 A07 identity freeze in `/private/tmp/assura-a07-current-0dff` passed
exact-toolchain, login-shell, wrong-target and wrong-root controls with
byte-identical freeze records and independent scoped review `PASS`. It is
no-credit preparation; canary, packet rebind and protocol review are not run.
The 4560c710 candidate and packet are candidate-base/no-credit after this
source advance. Preserve the earlier macOS watch-SIGINT hosted failure as
unfavorable diagnostic evidence: run `34615572565`, job `103316578631`, failed
at `tests/watch_cli.rs:196` with `watch did not stop after SIGINT`.

The post-merge push workflow is a separate reconciliation gate: a PR pass or
local rerun cannot close a slice when the merge-SHA workflow is failed,
cancelled, unavailable, zero-test, scope-uncertain or absent. Retain the exact
job/log handle and route the smallest recovery; do not retry an unchanged
invocation or infer product credit.

The next action is to refresh source/release/tag/PR/CI/topology and the ledger,
inspect active/implemented/verified/held records before pending rows, and keep
one explicitly owned recovery or preparation action live. If A07 screening is
separately authorized, rebuild and rebind at the refreshed source before any
allocation; otherwise continue the authorized recovery slice. Never stop at an
empty pending queue, and never infer product credit from process, canary,
protocol, skipped or zero-test evidence.

As-of correction checkpoint (2026-09-11, superseded by `40f1155c`; refresh before use):
`origin/master=851a6b831ea841317b78d94fce658a6974ef401a`; the revision-pinned
ledger is 32 items, zero ready-pending, five unfinished and three held, with
A07 active, W03 verified and R01/W02/F01 held. PR #286 merged the reviewed
ac3 process checkpoint, PR #287 reconciled it as ca81689, and PR #288 merged
the reviewed durable-goal and continuation artifacts as 851a6b8. Their
applicable Documentation, CI Scope, Security Scope, Evidence Gates and
GitGuardian checks passed, as did post-merge Documentation, Security Audit and
Rust CI. Product/Rust/performance/release jobs were scope-skipped and are not
acceptance proof. The 30b, c4f, ac3 and ca816 candidate packets are historical
no-credit after their source advances. A fresh 851a6 identity freeze is private
preparation; complete its bounded no-credit canary, packet rebind and isolated
protocol `PASS` before seeking separately authorized screening. See
[process-corrections-2026-09-11.md](process-corrections-2026-09-11.md) for the
measured VPS decision and layered context route.

Historical as-of routing checkpoint (2026-09-11, superseded by `cd629d4`): the latest reset
resolved `origin/master=961dced162c8fe67ad87d64e0be750249edd5681`; the ledger is
32 items, zero ready pending, five unfinished and three held: A07 active, W03
verified, and R01/W02/F01 held. PR #280's process-only route is merged; its
applicable Documentation, CI Scope, Security Scope, Evidence Gates and
GitGuardian checks passed, while product/Rust/performance/release jobs were
scope-skipped and are not acceptance proof. The root checkout has unknown
user-owned dirt; use clean owned worktrees and preserve foreign or stale
topology exceptions.

The 5b03c7f A07 packet is historical no-credit after 961dced. The current
961dced candidate was rebuilt with an explicit workdir after a provenance check
invalidated an accidental root-checkout build; its source/tree/binary/shim and
Rust/Cargo `1.94.1` identities are frozen privately. Two fresh source-only
canaries pass the full evaluator, the six-handle/two-condition packet is
rebound, and isolated protocol review returned `PASS` after concrete metadata
corrections. Refresh source/release/tag/PR/CI/topology and the ledger again
before separately authorized screening. A mismatch or later source
advance requires a new candidate sequence; never reuse the 5b packet. A02 is
complete; preserve residual fixture, launcher, child-isolation and evaluator
limitations. Route from `recovery-plan.md`, not this snapshot, after a fresh
fetch and ledger.

The historical parent-`c34f917` candidate-freeze observation used a clean
detached checkout and the exact Rust/Cargo `1.94.1` toolchain. Its non-symlink
`assura 0.4.0` binary hash is
`95c93052bd1993566d9f8209bdba5625c2b39a19287ed6358d710015d2ac59ff`, matching
`command -v`, version and target hash inside a minimal login shell. This is
historical no-credit preparation, not a current candidate binding. The six
valid holdouts are frozen; the private condition values, supplied-input
receipt, mapping, matrix and protocol `PASS` are still required and must not
be inferred from historical run names. After that redacted `PASS`, fetch
`origin/master` again, rebuild and freeze a new candidate identity before any
no-credit canary or screening allocation; never reuse this parent binary hash.

Copy the following prompt into a coding agent that can access the repository and planning task. It is designed for sequential execution with limited context. The queue and solution cards are the source of truth; no knowledge of the earlier conversation is required.

```text
Execute the Assura maturity backlog, one reviewable task at a time.

Planning root:
/Users/nroth/workspace/assura/.trellis/tasks/09-04-maturity-portfolio-strategy

Read first:
1. research/orchestration-plan.md
2. research/execution-backlog.md
3. research/backlog.json
4. This project's AGENTS.md and the skills relevant to the selected card.
Read prd.md for the overall product intent. Then read only the packet section
for your selected ID and any shared contract it explicitly references.

Objective: make Assura a dependable tool for executable repository conventions
and agent-assisted setup. The professional story supports technical product /
AI systems leadership. Do not expand project intelligence, semantic search,
agent orchestration, remote plugins or generic maturity scoring.

Begin by validating B00's completion evidence; if it is current, do not rerun
or reopen it. Use latest GitHub master as the source baseline. The original planning review
used `ed093668`, but that SHA is historical and never a permanent pin. The
original local checkout is older: do not implement on it. Refresh Git/PR,
release/tag, worktree and owner state, preserve unrelated changes and work in
an isolated current-master checkout. Inspect overlapping PR #142 and the
existing NickRoth case-study branch before creating duplicate work. Record
actual cwd, SHA, binary version and toolchain. Load the worktree skill for
isolation.

First inspect unfinished active/implemented/verified candidates and actual live
owners. Then select a pending item with evidenced merged dependencies and whose
required changes are present in this checkout. A not_needed dependency requires
written evidence/approved scope disposition. If no pending item is ready, keep
one coordinator-owned recovery, review or integration action live and record
its owner, phase, proof and exact next command; do not stop at a status report
or invent a card. If an item is held on publication, people or environment,
record the narrow hold and take another independent ready or process action.
Do not run the whole backlog as one giant patch. Default batch size is one card.

Execution-continuity invariant: a checkpoint is not a stopping condition. Keep
one concrete owned action live at all times: retain an active test, CI, review,
integration, or cleanup handle; resolve a failed gate; or select the next ready
card immediately after a terminal phase. Before yielding, record the action,
owner, source SHA/worktree, required proof, and the exact next observation or
command. An empty pending queue does not satisfy this invariant while an active,
implemented, verified, merge-ready, or integrated card still has a real next
phase. If a card is held by an external prerequisite, record that narrow held
action and continue another independent authorized card; do not convert the
whole train into a status report.

For the selected card:
- State ID, expected outcome, owned files and acceptance checks before editing.
- Follow its prescribed solution. Use existing patterns and commands. Proposed
  new files/options in the card must be implemented/documented/tested together.
- For behavior changes, first reproduce the defect or write the focused failing
  contract test. A test that merely checks the new code exists is not sufficient.
- Implement the smallest cohesive fix. Keep config, generated artifacts, tests,
  CLI help and public behavior aligned where the card requires it.
- Never widen excludes, disable rules/tests, remove benchmark rows, reduce
  severity, change CI scope or claim a skipped check passed to finish a task.
  If such a policy change is justified, record the evidence and request a
  separate maintainer decision.
- Run focused tests from the right cwd, then the relevant verification tier.
  cargo xtask pr already includes fast; one nested invocation proves both at
  readiness. Retain applicable final-source feature/OS/performance gates.
  Use the goal skill's matrix/VPS route; record actual exits and elapsed time.
- Use real temporary fixture repos for hook/installer tests. Restore or discard
  only your own disposable test data; never overwrite user hooks/configuration.
- Evaluate with the candidate binary, not a global older Assura installation.
  A passing check with no enforced policy, zero expected tests, generated-only
  hooks or fabricated evidence is failure.

For a novel failure not covered by the card, investigate up to two distinct
evidence-based hypotheses, record results and the smallest required plan change.
Do not repeatedly retry unchanged commands or guess a broad rewrite. Stop that
card for a real contract/authority decision; continue independent work if available.

For A07 initializer evaluation, read `a07-candidate-binding-plan.md` before
launching any run. The candidate must be bound by absolute executable identity
inside the initializer's actual command environment. Capture
`command -v assura`, `assura --version`, and the candidate SHA from that same
environment; a login-shell `PATH` mismatch invalidates the run and earns no
screening, holdout, or final-batch credit. Repair the runner and complete a
fresh identity canary before repeating the protocol. Keep private fixture and
evaluator details out of prompts, reviewer briefs, and public evidence. Read the
goal skill's `references/runner-isolation.md`: launch a fresh one-shot child
with a source-only fixture, a fixed public task prompt, and a minimal explicit
login-shell-safe `PATH`. Do not inherit the coordinator transcript or expose
evaluator contracts, private harness paths, hidden expected output, or ambient
global Assura installations. A forbidden-context read, global-binary fallback,
or missing identity observation invalidates the run and earns no allocation
credit, even when the evaluator is invoked with the intended binary afterward.
Place each fixture beneath a dedicated disposable parent with no sibling
worktrees or prior-run artifacts; recursive discovery of an out-of-scope parent
path is context contamination even when the fixture itself contains only public
source files.
Before screening allocation, read
`screening-manifest-contract.md` and validate the private manifest's exactly two
named product-input conditions, one-variable difference, blinded mapping and
complete 30-cell matrix. Obtain an isolated protocol-review `PASS` before
launching any screening cell; historical run names are not condition
definitions, and missing supplied-input evidence means the cell is not
executable. Refresh the candidate-bound canary against the current master
after the manifest review and before allocation.
Keep initializer events, private evaluator output and redacted evidence
separate. Run the evaluator only after the identity canary and initializer have
completed. Before supplying `--dimensions`, read the evaluator's declared
dimension set and pass exactly that set; the `policy` dimension owns the
trusted negative probe. An unsupported token is a failed no-credit invocation,
not a reason to weaken the contract or omit a required dimension.

Before a complex PR, request an independent review under project rules. Review
findings critically and fix valid issues. Commit only your owned, verified changes
when the repository workflow requires it. The active supported runtime goal
authorizes pushing and merging reviewed, fully gated code/documentation slices;
without that goal authorization, prepare locally only. Tags/releases, deploys,
branch-protection changes, invitations and publication still require their
specific authority. Local implementation and draft preparation should be
finished before any remote mutation.

Update research/backlog.json and write research/evidence/<ID>.md with:
state, actual source SHA/worktree, reproduction, changed behavior/files, exact
commands and exit results, negative controls, review findings, limitations,
commit/PR/integration state and next-ready ID. Do not mark done until the card's
actual outcome is proven. If local code is verified but hosted proof or publication
is pending, use verified or blocked with that exact reason.

At each card boundary, write a checkpoint containing:
1. ID and outcome achieved (or exact blocker).
2. Changed files and why.
3. Verification evidence and known limitations.
4. Commit/PR status, the named next action, and its owner.

Only make a final handoff after an explicit user pause, completion of the
authorized scope with every owned branch/worktree terminal, or an independently
audited external prerequisite after all independent authorized work is exhausted.
Do not end with another general strategy, a merely informational status, or a
list of concerns while a concrete next action remains.
```

## Starting with a particular card

Append `Execute only R01 after verifying B00 evidence and dependencies.` to select a card. Replace the ID only with an existing queue ID. For a longer session, explicitly authorize a batch size and integration approach; the default avoids accumulating unreviewed patches.

## Status meanings

- `pending`: not started.
- `active`: one agent owns the card; record its checkout.
- `implemented`: patch exists, proof incomplete.
- `verified`: local/card checks pass but required review/hosted/public outcome may remain.
- `done`: all required outcomes and evidence, including external ones if any, exist.
- `blocked`: exact required input/environment/authority unavailable; completed work recorded.
- `not_needed`: proven existing equivalent or explicit scoped exclusion, with rationale.

No automatic scheduler is created by this backlog. The coding agent uses the queue; human-dependent cards remain visible instead of being invented as completed.
