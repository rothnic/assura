# Execution control plane

This reference is the detailed operator plan for a long-running Assura goal.
It is loaded after the universal router and only when a goal starts, resumes,
changes phase, receives review feedback, or prepares a handoff. It does not
replace the Trellis ledger, a card contract, or a private evaluator packet.

## State machine

Advance one owned slice through these states. A state transition is valid only
when its exit record exists; a passing command or a status poll is not a
transition by itself.

| State | Required observation | Exit record | Next state |
| --- | --- | --- | --- |
| `RESET` | workflow gate, fetched base, release/tag, PR/CI, ledger and topology | source/ledger/ownership snapshot | `ROUTE` |
| `ROUTE` | active, implemented, verified and merge-ready cards inspected before pending rows | selected card, dependency proof, owner and exact action | `OWN` |
| `OWN` | clean current-base checkout and one owner | branch/worktree, source/tree, scope and live handle | `PROVE` |
| `PROVE` | cheap checks, focused red/green contract and smallest applicable tier | commands, host, toolchain, exits, counts, timings and limitations | `REVIEW` |
| `REVIEW` | independent review of the frozen commit and public evidence | finding contract/location/failure/smallest verification and verdict | `INTEGRATE` or `PROVE` |
| `INTEGRATE` | current-base candidate, resolved review, applicable pre-merge local/hosted/performance gates, configured post-merge push workflow observation and authority | merge SHA, pre/post-merge check rollups and acceptance decision | `RECONCILE` |
| `RECONCILE` | fetched post-merge base, reachability/tree equality, ledger and topology | card state, closure proof and next owner/action | `ROUTE` or `CONTINUE` |
| `CONTINUE` | a live repair, review, integration, cleanup or independently authorized recovery action exists | handle/owner and next observation | the named state |

If an action is held, keep the state and record the held contract, current
evidence, owner, smallest resolution step and independent work that continues.
Do not turn an empty pending set or an external hold into a terminal goal
state.

## Checkpoint contract

Write a compact checkpoint before and after each meaningful phase. The public
record contains the following fields; private evaluator values stay in the
private lane.

```yaml
checkpoint:
  recorded_at: <UTC timestamp>
  source_sha: <full fetched integration SHA>
  source_tree: <tree SHA when identity matters>
  ledger: <revision-pinned summary and selected card dependency result>
  card: <ID or recovery slice>
  phase: <reset|route|own|prove|review|integrate|reconcile|continue>
  owner: <agent/session owner>
  checkout: <absolute worktree>
  branch: <branch or detached SHA>
  live_handle: <session/CI/review handle or null>
  proof: <commands, exits, counts, timings, review and authority state>
  holds: <exact held actions, not generic blockers>
  next_action: <one command or decision, with owner>
  context_level: <universal|goal|card|special|private|protocol>
```

The `source_sha`, checkout and command environment are identity fences. If any
one differs from the claimed proof, classify the result as invalid and rerun
from the owned checkout. Keep historical and candidate-base evidence labeled
as-of; never overwrite it with a newer pointer.

## Layered disclosure

Load only the smallest layer that answers the next decision:

1. **Universal:** `AGENTS.md`, workflow-gate output, cwd, repository, branch,
   remote, source and status.
2. **Goal/phase:** the active goal, this control plane, the execution contract,
   validation routing and continuation control.
3. **Card:** PRD, current `backlog.json`, the selected packet and its evidence.
4. **Special lane:** exactly one phase reference such as runner isolation, CI
   triage, local-build/VPS procedure or the screening-manifest contract.
5. **Private/protocol:** private evaluator material or an isolated metadata
   review, never copied into implementation prompts or public evidence.

After compaction or a handoff, reload layers 1–2, verify the source and ledger,
then load only the selected card and one special lane. Do not paste historical
logs, all packets, hidden oracles, child transcripts or unrelated worktrees
into a prompt. Record `context_level: not exposed` when the platform gives no
budget signal and summarize the current base, card, proof, hold and next action
in three to six bullets.

## Validation budget and failure discipline

Use this order for every owned checkout:

1. workflow, routing, structure, scope and target-state checks;
2. focused red/green behavior and negative controls;
3. one serialized heavy local or remote tier for the changed surface;
4. one prepared push and exact-head hosted checks; after integration, observe
   every configured push-triggered workflow at the merge SHA before claiming
   reconciliation or closing the owned slice;
5. post-merge reconciliation and topology closure.

For each command or hosted job, record source/head, invocation fingerprint,
host, toolchain, queue time, execution time, expected and actual test/job
counts, exit and first actionable failure. A skipped, unavailable, cancelled,
zero-test or scope-uncertain result is unresolved, never a pass. Do not repeat
an unchanged failure. Retry only after the candidate, environment or documented
platform condition changed, and link the prior failure to the new run.

Proof may be reused only when source/tree, dependencies, configuration,
toolchain, environment and invocation are unchanged. `cargo xtask pr` already
contains `fast`; do not run an unchanged `fast` immediately before it. Do not
run parallel Cargo commands in one checkout; lock wait is contention, not
progress. Install pinned website dependencies once per isolated checkout.

A passing pull-request check rollup does not erase a later push-triggered
workflow result. If a configured post-merge workflow fails, is cancelled,
unavailable, zero-test or scope-uncertain, retain the exact merge SHA and job
log as unresolved evidence, route the smallest recovery or diagnosis, and keep
the card/goal open. Do not close the branch, call the slice fully gated, or
retry the same invocation without a changed candidate, environment or
documented platform condition. If the expected post-merge workflow is absent,
record `unknown` rather than inferring success.

## Measured VPS use

VPS execution is an optional efficiency experiment, not authority or final
cross-platform proof:

1. Resolve the configured SSH alias and probe OS, CPU/load, available memory,
   free disk, exact Rust/Node/package-manager toolchains and existing owned
   jobs. Record the probe and projected job footprint with safety margin.
2. Use the VPS only if the exact toolchain is available, measured headroom can
   hold the job plus margin, and no unrelated job would contend. If any check
   fails, keep the job local or choose a smaller applicable tier; never delete
   caches/worktrees to manufacture headroom.
3. Transfer one clean committed candidate as a verified Git bundle, check out
   its recorded SHA detached, and verify remote HEAD, tree, lockfile and
   toolchain before running one bounded heavy job. Preserve session identity,
   log, exit, elapsed time and utilization; reconnect to a live job rather than
   launching a duplicate.
4. Compare cold/warm and baseline/candidate rows only when inputs match. VPS
   Linux supplements and never replaces macOS, Windows, browser, host-
   permission, performance or hosted gates. Do not change CI runners,
   protections or caches as part of routine validation.

Propose a CI efficiency change only after at least three comparable prepared
runs show queue/execution time, first-pass success, retries and coverage. The
proposal must preserve the same required tests and include cold/warm proof.

## Review and merge fence

Freeze one coherent commit and its public evidence before dispatching an
independent reviewer. The reviewer receives only the frozen diff, contract and
evidence needed to judge it. Every finding is recorded as:

```text
contract | location | failure scenario | smallest verification | disposition
```

Accept and fix concrete findings as a delta, rerun affected gates and obtain a
scoped rereview of that delta. Reject or defer only with evidence and an exact
authority/dependency reason. Merge only the reviewed current-base SHA with all
applicable pre-merge local, hosted and performance gates resolved; retain
scope-skipped jobs as non-applicable. After the merge, observe the configured
push-triggered workflows at that exact merge SHA before recording a fully
reconciled slice. A failed, cancelled, unavailable, zero-test or
scope-uncertain post-merge job is an unresolved recovery route, not a green
result. A process/documentation merge never promotes a product card or grants
screening, release, deployment, publication, invitation or protection
authority.

## Handoff and topology closure

Before yielding, ask whether a live handle, repair, review, integration or
cleanup action remains. If yes, continue it or leave its exact handle and an
independent authorized route. A final handoff requires completed authorized
scope, an explicit user pause, or an independently audited external hold after
all independent work is exhausted.

Run the topology audit in report mode at reset and after material changes, and
in strict mode before handoff and removal. Preserve unknown/user-owned dirt,
foreign dirty worktrees, unreadable/prunable registrations and historical
branches. Remove only an exact clean goal-owned branch/worktree after proving
merged reachability or recording a verified bundle archive and restore path.
