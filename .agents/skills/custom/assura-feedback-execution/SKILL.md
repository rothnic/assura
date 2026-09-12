---
name: assura-feedback-execution
description: "Start or resume the approved compact-feedback and performance backlog."
---

# Execute compact feedback

Explicit `$assura-feedback-execution` invocation starts or resumes this plan.
Merely reading, reviewing, or creating the skill does not start implementation.
Read [execution](references/execution.md), then the selected card in
[backlog](references/backlog.md). The product contract is
[feedback-contract](references/feedback-contract.md).

The planning task is `.trellis/tasks/09-12-compact-feedback-plan`; attach it
when the host requires a task record. Do not create another queue or resume
the maturity train.

## Route

1. Inspect checkout, clean ownership, refreshed integration base and current
   candidate/PR. Preserve this planning package if it is not yet merged.
2. Read the execution contract and backlog index. Resume an owned candidate
   before selecting the first unfinished card with satisfied prerequisites.
3. Read only that card, the feedback contract and its named implementation
   routes. Apply Rust/build/hook skills only when the card needs them.
4. Implement, validate, independently review and pursue authorized integration
   for that card. Use the Assura orchestration review brief; the reviewer
   returns findings without taking ownership of scope.
5. Update that card once at a real transition. Continue within this backlog
   until acceptance, user pause or a specific held action. Retain live test
   and review handles instead of restarting them.

## Boundaries

- Implement CF01–CF04 only. No framework replacement, private evaluation,
  semantic productivity scoring, hosted service or speculative expansion.
- Routine injection stays <=256 UTF-8 bytes including the Assura wrapper;
  collection and repetition controls are acceptance, not polish.
- Include scoped instruction corrections with CF01. Do not make a separate
  process-reconciliation PR before the first product fix.
- A changed integration SHA is an observation. Reuse proof with unchanged
  actual inputs; retain required current-candidate integration checks.
- Invocation authorizes implementing and preparing/reviewing this backlog.
  Use existing merge authority; this grants no new release, deployment,
  protection, publication or global-setting authority. Name the exact held
  action when authority is missing instead of silently stopping at a PR.
- Preserve the planning branch until its contents are integrated or explicitly
  preserved with an owner. The backlog is the only new card-status ledger.
