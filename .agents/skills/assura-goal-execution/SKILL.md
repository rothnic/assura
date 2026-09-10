---
name: assura-goal-execution
description: "Resume Assura goals and Trellis backlogs with evidence, review, validation and clean integration."
---

# Assura Goal Execution

Use for goals in `docs/goals/` and canonical execution tasks in `.trellis/tasks/`.
The latest user scope controls whether this session executes product work or
only improves its process. Do not substitute a process PR for product acceptance.

## Start

1. Run the workflow gate and preserve unrelated dirty work. Refresh the base;
   compare checkout HEAD before reading the ledger. Use `git show
   origin/master:<task>/research/backlog.json` or a clean current-base checkout
   when the supplied canonical path is on an old branch. Apply the
   [source-pointer lifecycle](references/source-pointer-lifecycle.md) to
   classify every dated snapshot before routing from it.
   Run the read-only [context-routing audit](scripts/audit-context-routing.py)
   against the same checkout; a failed routing audit is a process-repair
   route, not product evidence.
2. Read the goal/PRD, queue, selected packet and evidence. Inspect active,
   implemented and verified candidates before pending ones; verify live owners.
   Use [scripts/audit-ledger.sh](scripts/audit-ledger.sh) for a read-only
   revision-pinned card/dependency/branch routing summary; it does not prove
   owner liveness or card acceptance.
   If the supported runtime goal is already active, reconcile it instead of
   creating a duplicate or replacement. When no pending row is ready, retain a
   coordinator-owned recovery/review/integration action with an exact next
   observation; an empty ready set is not a stopping condition.
3. Read [execution contract](references/execution-contract.md) for the phase,
   acceptance, continuation and merge rules. Use the installed personal
   `assura-orchestration` skill and its independent review brief when available;
   the repository contract remains usable without that installation.
4. Before spending on gates, read [validation routing](references/validation-routing.md)
   and, for slow/failed/queued checks, [CI gate triage](references/ci-gate-triage.md).
   Environment failures route to `assura-local-build`, not product changes.
5. For candidate-bound initializer evaluation, read
   [runner isolation](references/runner-isolation.md) before launching the
   child agent. A parent shell's `PATH` prefix or a passing evaluator command
   is not identity proof.
6. For A07 screening, read the task's
   [screening manifest contract](../../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/screening-manifest-contract.md)
   before naming conditions or allocating runs. Require its immutable
   six-handle holdout binding with current candidate provenance, hand/freeze
   evidence, creation time, second read-only confirmation and explicit
   exclusions; historical hashes or prose are not proof. Do not invent missing
   product inputs from historical run names.
7. Use [layered context routing](references/context-routing.md) after
   compaction and at phase transitions. Record the current phase and exact next
   action in the selected card's evidence before/after major phases. A task
   path is portable; an old checkout snapshot, automation prompt or conversation
   summary is not current state.
8. Apply [continuation control](references/continuation-control.md) after
   resets, compaction, failed gates, review results and empty ready-pending
   queries. Keep one owned next action live; an empty pending set is not a
   stopping condition.

## Context routing

- Always: AGENTS, this index, selected card's current evidence and contract.
- Phase transition/review/merge: execution contract and exact review delta.
- Test/build/CI placement: validation routing; environment detail only on need.
- Slow, failed or queued hosted checks: `references/ci-gate-triage.md`; keep
  the exact head, scope decision, queue/execution timing and retry reason.
- A07 or any agent-driven evaluation: runner isolation; keep the evaluator
  oracle and private fixture details outside the child context.
- A07 screening condition or matrix: screening manifest contract; keep its
  values and mappings private; do not allocate until the six-handle binding and
  current-candidate protocol `PASS` exist.
- Goal start, compaction or handoff: context-routing; load only the next
  phase/card layer and record a compact checkpoint.
- Source refresh, merge or SHA mismatch: source-pointer-lifecycle; reconcile
  current, candidate-base and historical labels before selecting work.
- Maturity train recovery: the canonical task's `research/recovery-plan.md`.
- Reset, compaction, failed gate, review or empty queue: load
  `references/continuation-control.md` and record the next route.
- Never load all packets, all historical logs or private evaluation fixtures
  into an implementation/reviewer prompt. Link exact evidence on demand.

## Iteration Review Hook

Treat one iteration as a meaningful implementation/review loop: planning a
slice, editing files, running validation, and deciding the next slice.

Every third iteration, and before any final handoff:

1. Record the current iteration count in the goal progress log.
2. Record available context-health information. If the platform exposes token
   or context budget, include it; otherwise write `context level: not exposed`
   and summarize the relevant prior messages in 3-6 bullets.
3. Review the current conversation, progress log, failed commands, and repeated
   explanations.
4. Decide whether a reusable project skill should be created or updated under
   `.agents/skills/`.
5. If a skill is created or updated, keep `AGENTS.md` as a lean router: add only
   the skill name, trigger, and one-line purpose. Put operational detail inside
   the skill.
6. Re-run `assura check` after changing `.agents/skills/`, `AGENTS.md`, or
   `.assura/config.yml`.

## Skill Creation Bar

Create or update a skill when any of these are true:

- The agent rediscovered the same repo-specific workflow twice.
- A platform/build workaround is needed for repeatable validation.
- A validation failure needs reusable remediation steps.
- The goal introduces a recurring implementation pattern.
- The agent needed more than one paragraph in `AGENTS.md` to explain a
  procedure.

Do not create a skill for one-off facts, short status notes, or details that
belong only in the current goal's progress log.
