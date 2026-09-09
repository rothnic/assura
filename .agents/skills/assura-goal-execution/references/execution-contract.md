# Execution and acceptance contract

## One state, one owner

Keep card state in the existing `backlog.json`; extend its existing evidence
record with this checkpoint, rather than creating another scheduling queue:

```text
Card/slice and observable acceptance:
Owner/session and last verified liveness:
Phase: investigate | implement | verify | review | integrate | reconcile
Integration base and candidate SHA; checkout/branch:
Proof: command, cwd, host/toolchain/binary, exit, result/log:
Review SHA, findings and disposition:
Live process/CI/review handle, or none:
Next action and triggering observation:
Held action/contract/evidence/owner/smallest resolution/independent work:
Closure: active | merge-ready | merged | archived; restore proof if archived:
```

An owner label is not proof of a live worker. Reconcile actual sessions and
branch provenance before takeover. Preserve unknown dirt in its checkout.
The coordinator owns transitions and closure; a reviewer owns only the frozen
review scope. Independent work can proceed with separate ownership and capacity.

## Success before integration

Before editing, distinguish the slice acceptance from whole-card acceptance.
An improvement PR can merge with its own proven contract while the parent card
remains active. Never mark the card done from that PR alone.

Merge requires every applicable predicate, with evidence for each:

1. The reviewed change meets its written slice acceptance, including failure
   and valid-exception cases; no unresolved accepted correctness finding.
2. The tested/reviewed source equals the submitted candidate. Later metadata
   changes are reviewed as a delta and rerun any affected gates.
3. Current integration base is included; required local and hosted checks on
   the final candidate pass. VPS proof supplements supported-platform CI.
4. All required performance rows/thresholds remain intact and pass. A failed,
   cancelled, unavailable or unexpectedly skipped required check is unresolved.
   Legitimate scope-skipped jobs are recorded as non-applicable, never passing.
5. Formal branch review requirements are satisfied; merge and any coupled
   deployment have authority. Review approval grants no additional authority.
6. The owned worktree is clean and the post-merge outcome/cleanup action has
   an owner. Verify actual merged reachability before closing the branch.

Card `done` additionally requires its full observable acceptance, including
independent evaluation or external outcomes where specified. Preserve private
evaluator separation; public evidence contains aggregates, not hidden oracles.

## Progress across interruptions

At a test/CI observation timeout, retain the exact handle; resume it instead of
starting another run. Commit coherent work before review or handoff. On resume,
refresh source and handles, then execute the checkpoint's next phase. A quiet
notification choice does not mean no work should run.

If two scheduled wakes find no phase change, inspect whether the worker/process
is live, whether instructions are stale, and whether goal runtime state agrees
with the user objective. Choose a discriminating diagnostic or resume the
missing transition; do not issue repeated unverified unchanged-status claims.
Update the existing automation to resolve current evidence, not hardcoded PRs.
Goal lifecycle must use supported product operations; never mark complete or
replace an unfinished goal simply to make runtime state convenient.

Failures hold the specific action. Investigate two distinct evidence-backed
hypotheses, then independently review the method/contract if neither resolves
it. This is a decision checkpoint, not an automatic whole-goal stop. Preserve
unfavorable results and continue independent authorized work.

Before final handoff, audit every owned branch: merged and clean, deliberately
archived with verified bundle/restore instructions, or the single continuing
named candidate when the user explicitly pauses. Never delete unknown work.
Run the topology report/strict audit, retain its exit and limitations. If it
crashes on stale registrations, read `git worktree list --porcelain`, inspect
each path and `git worktree prune --dry-run -v`, and classify the remaining
inventory read-only. That fallback is evidence, not a passing strict audit.
