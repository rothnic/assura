# Compact feedback execution plan

Planning only: the user requested organization and a start command. Product
implementation waits for explicit invocation.

Create the discoverable `assura-feedback-execution` skill and bounded backlog
for tiny configurable statistics, cached/asynchronous collection, existing
feedback performance fixes and truthful LS-Lint/installed-hook evidence.

Canonical requirements and card statuses live in the skill's
[feedback contract](../../../.agents/skills/custom/assura-feedback-execution/references/feedback-contract.md)
and [backlog](../../../.agents/skills/custom/assura-feedback-execution/references/backlog.md).
The [execution contract](../../../.agents/skills/custom/assura-feedback-execution/references/execution.md)
defines start, scope, review, authority and closure.

Planning acceptance: references exist, entrypoint is discoverable, byte/cadence/
threshold/async contracts are concrete, cards have observable acceptance, and
no product implementation or execution session has started.

Do not duplicate CF01–CF04 status here or in another task queue.

## Retrospective delta

The supplied review is incorporated as four process invariants inside CF01,
not as a second maturity queue: reconciliation is idempotent; activity,
authority and product acceptance remain separate; live context stays a small
single-source checkpoint; and every owned candidate ends merged or deliberately
archived with an owner, handle, next action and restore evidence. The reported
32-card/count mismatch, process-only churn, oversized progress file and
worktree/disk inventory are as-of diagnostics for negative tests and preflight,
not reasons to restart or rewrite historical records.
