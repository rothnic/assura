---
name: assura-goal-execution
description: "Resume Assura goals and Trellis backlogs with layered context routing, evidence, independent review, validation and clean integration."
---

# Assura Goal Execution

Use for goals in `docs/goals/` and canonical execution tasks in `.trellis/tasks/`.
The latest user scope controls whether this session executes product work or
only improves its process. Do not substitute a process PR for product acceptance.
For the compact feedback/performance train, route implementation through
`.agents/skills/custom/assura-feedback-execution/SKILL.md`; keep its CF01
process correction bounded and do not repeat instruction-reconciliation cycles
when the source, ledger and topology inputs are unchanged.

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
   is not identity proof. Scan child events for private/evaluator/foreign paths;
   generic ambient skill metadata is a retained no-credit limitation until the
   isolated protocol reviewer explicitly dispositions it.
   When a phase freezes a candidate, keep the public freeze record in the
   candidate checkout and a private harness copy byte-identical; use relative
   shared evidence references and assert JSON, source/tree, binary/shim,
   toolchain, login-shell and negative-control identity before review. A
   freeze or supporting `assura check` is preparation evidence only and does
   not advance canary, screening or credit state.
6. For A07 screening, read the task's
   [screening manifest contract](../../../.trellis/tasks/09-04-maturity-portfolio-strategy/research/screening-manifest-contract.md)
   before naming conditions or allocating runs. Require its immutable
   six-handle holdout binding with current candidate provenance, hand/freeze
   evidence, per-handle creation time/evidence, second read-only confirmation
   and explicit exclusions; historical hashes or prose are not proof. Use one
   exact canonical toolchain identity in the candidate freeze, both condition
   rows, every supplied-input receipt, every per-layout binding, and the
   second-readonly record, then compare those fields before review. Do not
   invent missing product inputs from historical run names. Before dispatching
   the protocol reviewer, run
   [`scripts/validate-a07-packet.py`](scripts/validate-a07-packet.py) with the
   expected source/tree/binary/shim/contract/prompt/toolchain identities. This
   metadata-only gate must resolve every current receipt, creation and
   second-readonly reference, prove exactly six handles and 30 reserved cells,
   and keep screening/allocation/credit false. A non-zero result blocks review;
   fix the packet and rerun it. Do not create a current packet by blind
   string-replacement of an older one: preserve historical references
   explicitly, write current aliases deliberately, and retain the validator
   output with the review record. Run the companion distinct-identity control
   [`scripts/test-validate-a07-packet.py`](scripts/test-validate-a07-packet.py)
   so a binary/shim hash mismatch has an explicit valid/invalid test.
   Load [candidate-bound packet coherence](references/packet-coherence.md)
   when constructing, correcting, reviewing or reconciling a packet. Require
   an immutable pre-verdict role/hash map, a coordinator-owned post-verdict
   finalization map, and cross-record status/timestamp/reference assertions;
   a printed validator or a plausible-but-stale dependent record is not proof.
7. Use [layered context routing](references/context-routing.md) after
   compaction and at phase transitions. Record the current phase and exact next
   action in the selected card's evidence before/after major phases. A task
   path is portable; an old checkout snapshot, automation prompt or conversation
   summary is not current state.
8. Apply the detailed [execution control plane](references/execution-control-plane.md)
   for the state transition, checkpoint fields, validation budget and VPS
   decision. Keep this procedure out of `AGENTS.md`; it is loaded only for
   execution, review, integration or handoff phases.
9. Apply [continuation control](references/continuation-control.md) after
   resets, compaction, failed gates, review results and empty ready-pending
   queries. Keep one owned next action live; an empty pending set is not a
   stopping condition.

## Review and continuation corrections

An independent review is a live phase, not a reason to stop. If feedback is
vague, restate it as a concrete contract, file/location, failure scenario and
smallest verification; ask the reviewer for a scoped rereview after the delta
is fixed. Preserve the original finding and its disposition in the evidence.
While a review, CI observation or authority decision is pending, continue only
independent authorized preparation with a separate owner/worktree. Before any
handoff, look for a live handle, repair, integration or cleanup action and
execute or record that exact next action.

## Context routing

- Always: AGENTS, this index, selected card's current evidence and contract.
- Phase transition/review/merge: execution contract and exact review delta.
- Test/build/CI placement: validation routing; environment detail only on need.
- Slow, failed or queued hosted checks: `references/ci-gate-triage.md`; keep
  the exact head, scope decision, queue/execution timing and retry reason.
- A07 or any agent-driven evaluation: runner isolation; keep the evaluator
  oracle and private fixture details outside the child context.
- A07 screening condition or matrix: screening manifest contract; keep its
  values and mappings private; do not allocate until the six-handle binding,
  per-handle creation records, canonical toolchain comparison and
  current-candidate protocol `PASS` exist.
- Goal start, compaction or handoff: context-routing; load only the next
  phase/card layer and record a compact checkpoint.
- Any state transition or expensive-gate decision:
  `references/execution-control-plane.md`; keep its checkpoint and validation
  budget fields in the selected card evidence rather than in `AGENTS.md`.
- Source refresh, merge or SHA mismatch: source-pointer-lifecycle; reconcile
  current, candidate-base and historical labels before selecting work.
- Maturity train recovery: the canonical task's `research/recovery-plan.md`.
- Reset, compaction, failed gate, review or empty queue: load
  `references/continuation-control.md` and record the next route.
- Never load all packets, all historical logs or private evaluation fixtures
  into an implementation/reviewer prompt. Link exact evidence on demand.

## Expensive-gate and context discipline

Use a bounded context ladder rather than copying the whole task into each
prompt. At reset or compaction load `AGENTS.md`, the workflow-gate result, the
current source/ledger and `research/orchestration-plan.md`; for a card load
only its packet, evidence file and named contract; for a special phase load
the linked reference (runner isolation, CI triage or local-build VPS
selection). Record the exact source, owner, phase and next command in the
checkpoint. A routing audit is process evidence, never product acceptance.

Place cheap gates before expensive ones: workflow/context/structure/scope and
identity first, one focused test or evaluator next, then one serialized Cargo
or hosted tier for the changed surface. Reuse a build only when source,
dependencies, toolchain, configuration and invocation are unchanged. Select
`vps-dev` only after a fresh SSH, capacity/disk, existing-job and exact
toolchain probe passes; remote Linux supplements and never replaces platform,
browser, permission, performance or hosted proof. Preserve failed, skipped,
zero-test, cancelled and unavailable required gates; do not retry an unchanged
failure or turn a passing canary into card success.

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
