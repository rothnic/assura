# Roadmap Phase 01 Execution

## Goal

Execute `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`
from updated `master` on branch `codex/roadmap-phase-01-execution`.

This task is not a roadmap-writing task. It owns implementation, validation,
review, and evidence collection for the Phase 01 goal sequence.

## Source Documents

- `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`
- `docs/goals/assura-goal-01-trustworthy-self-enforcement.md`
- `docs/goals/assura-goal-02-policy-language-completeness.md`
- `docs/goals/assura-goal-03-agent-feedback-delivery-loop.md`
- `docs/goals/assura-goal-04-fast-incremental-check-engine.md`
- `docs/goals/assura-goal-05-installable-adoption-path.md`
- `docs/goals/assura-goal-06-review-evidence-and-quality-gates.md`
- `docs/goals/assura-goal-07-extension-and-plugin-foundation.md`
- `docs/goals/assura-goal-08-release-readiness-and-ecosystem.md`

## Execution Rules

- Start from updated `master`; do not stack on the roadmap-publication PR
  branch.
- Execute goals in sequence. A later goal can be researched early, but it is
  not complete until all earlier dependency goals are complete and reviewed.
- Treat each linked goal as if it were run manually as its own goal:
  implementation, validation commands, review tasks R0-R5, reviewer blocking
  criteria, and completion evidence all apply.
- Preserve the stable public feedback surface:
  `assura check --format agent`, with Codex delivery only through
  `assura check --format agent --agent codex`.
- Do not reintroduce package feedback CLIs, per-agent CLI entrypoints, or
  per-agent `--format` values.
- Record progress in the active goal document before and after major phases.
- Use review-agent review before PR creation and address or explicitly reject
  findings with rationale.

## Phase Completion Criteria

Phase 01 is complete only when:

- all eight linked goals are complete;
- the Phase 01 ledger links each goal's evidence;
- each goal's required validation commands pass or have documented,
  owner-approved exceptions;
- review tasks R0 through R5 are complete for every goal;
- checked artifacts are reproducible from committed files;
- CI and local verification evidence are linked from the PR; and
- the next roadmap phase is created or identified without marking the broader
  Assura roadmap complete.

## First Slice

Begin with Goal 01, Trustworthy Self-Enforcement:

- audit active Trellis tasks and archive only completed merged work;
- align roadmap, active goal docs, current branch, and PR state;
- audit `.assura/config.yml` against current repository shape;
- remove or mark stale workflow/command-surface docs as historical;
- add targeted self-check fixtures or tests for any discovered config blind
  spots;
- run the Goal 01 required validation commands.
