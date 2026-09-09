# Layered context routing

Use this reference at goal start, after compaction, and whenever a handoff or
phase transition changes what the agent must know. Load the smallest layer that
can answer the next decision; do not paste full histories or private oracles
into a child, reviewer, or public artifact.

## Layers

| Layer | Load | Purpose |
| --- | --- | --- |
| Universal | `AGENTS.md`, workflow-gate output, `pwd`, branch/remote/status | Common policy, ownership and readiness |
| Goal/phase | `assura-goal-execution/SKILL.md`, `execution-contract.md`, `validation-routing.md` | Continuation, acceptance, review and gate placement |
| Card | `prd.md`, `backlog.json`, selected packet, selected `evidence/<ID>.md` | The observable outcome and current next action |
| Special lane | Only the reference named by the phase (for example `runner-isolation.md`) | Fragile or high-risk procedure |
| Private evaluator | Private manifest, contracts and raw results outside the repository | Oracle inputs and detailed provenance; coordinator/evaluator only |

The task's `recovery-plan.md` is a phase reference for maturity-train recovery,
not a replacement queue. Historical logs are read only when a current record
links to a specific decision or unfavorable result.

## Routing sequence

1. Refresh source, PR/CI state, worktree ownership and the workflow gate.
2. Read the universal layer and classify dirty paths before loading detail.
3. Read the goal/phase layer, then the active/implemented/verified card before
   looking at pending cards. Resolve the card's dependencies at current master.
4. Load one special-lane reference only when the selected phase requires it.
5. Keep private evaluator material in its private lane. Public evidence may
   retain only approved candidate identity and redacted aggregates.
6. Record owner, phase, source SHA, worktree, proof, live handle and exact next
   action in the card evidence before yielding or changing cards.

## Context-health checkpoint

Every third meaningful iteration and before handoff, write `context level:
not exposed` when the platform gives no budget signal, then summarize in 3–6
bullets: current base, selected card, last proof, unresolved finding, live
handle, and next action. Remove duplicated historical narrative from the live
progress file by moving exact old entries to its linked history file.

## Prompt boundary

An implementation child receives only its public task, ordinary source context,
and the candidate tool. A reviewer receives the frozen diff, contract and
public evidence needed for the review. Neither receives evaluator contracts,
private paths, hidden expected output, coordinator transcripts, or unrelated
worktree names. If a needed fact is private, record a boolean/aggregate outcome
and keep the fact in the private lane.

## Stop/continue decision

A failed check, review finding, missing permission or observation timeout holds
that action with its owner and smallest resolution. It does not end the goal
while an independent repair, review, integration or cleanup action remains.
After a terminal phase, route immediately to the next card or to the named
held-action check; never issue a status-only handoff.
