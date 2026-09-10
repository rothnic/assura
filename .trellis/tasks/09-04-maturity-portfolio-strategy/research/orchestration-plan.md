# Maturity train orchestration plan

Status: active process contract, 2026-09-10. This document routes the
supported runtime goal; it does not promote a product card, grant authority,
or create screening, holdout, publication, release, deployment, or invitation
credit. The task ledger and each card packet remain the acceptance source of
truth.

## Outcome invariant

The train advances one explicitly owned slice at a time from its current
source through investigate, implement, verify, independent review, current-
base integration, post-merge reconciliation, and clean closure. A passing
slice is not a passing parent card. A failed, skipped, zero-test, unavailable,
or scope-uncertain required gate remains unresolved; thresholds, negative rows,
and unfavorable evidence are preserved.

The coordinator never ends at an empty pending queue or an observation
timeout. The next checkpoint names the owner/session, source and candidate
SHA, checkout/branch, phase, proof and exact next observation. A card-level
hold records its contract, evidence, owner, smallest resolution and unrelated
work that can continue. Whole-goal completion requires accepted cards, final
current-base proof, and merged or verified-archived owned topology.

## Layered context route

Load the smallest layer that answers the next decision. This keeps AGENTS.md a
universal router instead of a second manual and prevents a compacted session
from routing from an old checkout or private evaluator detail.

| Decision | Read | Output |
| --- | --- | --- |
| Reset/ownership | `AGENTS.md`, workflow-gate output, cwd/branch/remote/HEAD/status, topology report | clean owned checkout or preserved dirt classification |
| Source/queue | refreshed `origin/master`, `audit-ledger.sh`, `backlog.json`, recovery plan | revision-pinned active/implemented/verified/merge-ready route |
| Card execution | PRD, selected packet and `evidence/<ID>.md` only | contract, files, dependencies, focused gate and owner |
| Special procedure | phase-linked reference such as validation routing, runner isolation or screening manifest | bounded commands and privacy/authority limits |
| Review/integration | frozen diff, public evidence, review brief and current-base checks | finding disposition, final-head proof and closure action |

At goal start, after compaction, and before handoff run
`scripts/audit-context-routing.py` from the same clean checkout. A failed
routing audit is a process-repair slice; it is not product acceptance and does
not justify copying detail into AGENTS.md. Keep private conditions, mappings,
fixtures, raw evaluator results and child transcripts outside prompts and
public evidence.

## Execution loop

1. **Reset.** Fetch `origin/master`; record its full SHA and time. Run the
   workflow gate, context-routing audit, revision-pinned ledger and topology
   report. Preserve unknown/user-owned dirt and inspect actual worker/session
   liveness; an owner string alone is not liveness.
2. **Route.** Inspect active, implemented, verified and merge-ready cards before
   pending rows. Choose exactly one real action with one owner: resume a live
   handle, diagnose a failed gate, implement/verify an authorized slice, obtain
   a named authority decision while preparing independent work, or integrate a
   reviewed current-base candidate.
3. **Prove.** Run cheap structure/scope/target checks first, then the focused
   red/green contract and the smallest applicable tier. Keep one heavy Cargo
   sequence per checkout; `cargo xtask pr` already includes `fast`. Record cwd,
   source, toolchain/binary identity, host, elapsed/wait time, exit and test/job
   counts. Do not rerun an unchanged failing method.
4. **Review.** Freeze the committed diff and public evidence. Dispatch an
   independent reviewer with the Assura brief. Vague feedback becomes a
   concrete contract, location, failure scenario and smallest verification;
   accepted findings are fixed as a delta, affected gates rerun, and the delta
   independently rereviewed. Continue only independent authorized preparation
   while review or CI handles remain live.
5. **Integrate.** Merge only reviewed current-base source with all applicable
   local/hosted/performance checks passing, formal review resolved and existing
   authority sufficient. Scope-skipped jobs are recorded as non-applicable.
   Review trigger/deployment coupling read-only; a merge permission never
   grants deploy, release, publication or invitation authority.
6. **Reconcile.** Fetch again, prove merged reachability/tree equality, rerun
   the ledger and topology audits, update the compact checkpoint, and remove
   only the exact clean owned branch/worktree after its closure proof. Keep
   historical snapshots labeled as-of rather than creating SHA-only churn.

## Efficient validation placement

- Local: workflow/context/structure/target-state checks before long work;
  install pinned website dependencies once per isolated checkout; use focused
  tests and the applicable `cargo xtask` tier.
- VPS: use the documented `assura-local-build` bundle procedure only after a
  fresh SSH, disk, memory, toolchain and owned-job probe. Start one bounded
  heavy job, retain its session/log/exit, and compare only immutable
  candidate/baseline binaries. Remote Linux supplements, never replaces,
  macOS/Windows/browser/host-permission or hosted proof.
- Hosted: push one prepared candidate, wait on the exact current-head checks,
  preserve failed performance rows and cancelled/unavailable required jobs, and
  never use retries, cache changes, protection changes or a self-hosted runner
  to manufacture a green result. Propose CI efficiency changes only after
  three measured comparable runs show queue/execution, retry and coverage
  effects.

## Current train route — b7043ab (refresh required)

At the post-merge refresh, `origin/master=b7043ab1b402a0dd103de6fe67e307442e67964a`
is the only current source pointer. PR #269 is merged as a process-only
documentation slice with exact tree equality to reviewed `0b3bdca`; its worktree
and branch are closed. The earlier `8be6103` candidate, canaries and isolated
protocol `PASS` are historical no-credit evidence because the merge advanced the
source tree. The ledger remains 32 items, no ready-pending card, five unfinished
cards and three narrow holds. A02 is complete and its plain-init handoff
incident is historical. A07 remains the active lane: the next owner/action is a
fresh b7043ab source/ledger rebind, candidate freeze and no-credit canary before
any separately authorized screening preparation. Do not reuse the 8be packet or
allocate/credit cells from this process merge. R01 still needs the raw callback
trace or a specific maintainer native-readiness decision. W02 needs explicit
Cloudflare approval before any current push that could trigger builds. W03's
technical change is verified but publication remains separately authorized.
F01's pilot kit is prepared, while participant selection/invitations remain
Nick-authorized. These holds do not end independent process, review, topology,
or authorized integration work.

## Checkpoint record

Before yielding, changing cards, dispatching review, or handing off, append a
compact record to the selected evidence:

```text
source/base: origin/master=<full SHA>; fetched_at=<time>
card/slice: <ID and observable outcome>
owner/session: <owner and live handle or none>
phase: investigate | implement | verify | review | integrate | reconcile
checkout: <absolute worktree>; branch=<branch>; candidate=<SHA>
proof: <commands, host/toolchain/binary, exits, elapsed, logs>
review: <review SHA, finding IDs and dispositions>
next: <exact command/observation and trigger>
held: <contract/evidence/owner/smallest resolution/independent work>
closure: active | merge-ready | merged | archived
```

## Process-slice evidence

This plan and its routing audit were validated from a clean detached checkout
at `origin/master=f921cae79d04e98ce729eecc64c79c68e4591fb9` before the candidate
was committed. The positive controls were:

- `python3 .agents/skills/assura-goal-execution/scripts/audit-context-routing.py . .trellis/tasks/09-04-maturity-portfolio-strategy` — exit 0; 42 routing/link/reference checks passed.
- `cargo run --quiet -- check --format json .` — exit 0; `success=true`, 1,827 files, 396 directories, and only existing low-severity line-length advisories.
- `cargo xtask evidence` — exit 0; review-evidence and CI-scope policy checks passed.
- `pnpm --dir website install --frozen-lockfile` followed by `cargo xtask docs` — exits 0; 48 website pages built.
- `git diff --check` and `python3 -m py_compile` for the audit script — exit 0.

The negative control ran the same audit against the pre-correction dirty
checkout. It returned exit 1 with 11 findings: a 295-line AGENTS router, an
overlong skill description, no `.trellis/tasks/` scope, and all seven layered
references absent. This demonstrates the audit catches the exact stale-context
failure without treating the old checkout's task state as current evidence.

The audit's missing-artifact negative control removed
`research/executor-prompt.md` in a disposable detached worktree and returned
exit 1 with `TASK_FILE executor-prompt.md FAIL`; the file was restored and the
worktree removed cleanly. The four task routing artifacts are therefore
required inputs, not optional observations.

The independent review of candidate `e479ac6` raised `CTX-001` because those
four files were initially optional. The finding was accepted and fixed in
`8803376`; the scoped rereview of `5dc8bfa` returned `PASS` with no remaining
concrete findings. The audit remains intentionally structural: it does not
prove task-content freshness, external references, or private evaluator
semantics, so those checks stay in the goal and card contracts.

The independent impasse review also found that the train's `task.json` primary
`branch` still pointed at the historical A05 checkout. The correction clears
that field to `null` and declares the active process branch only in
`meta.execution_branches`; `cargo xtask target-state` then passes without
inventing a single permanent train branch. This keeps session attribution from
silently selecting a completed card while preserving explicit card-branch
provenance.
