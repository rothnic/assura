# Roadmap Iteration 01 Execution

## Iteration

Execute the roadmap iteration described by
`docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md` from updated
`master` on branch `codex/roadmap-phase-01-execution`.

This task is not a roadmap-writing task. It owns implementation, validation,
review, and evidence collection for the Iteration 01 goal sequence.

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
- Record progress in the active execution goal and the iteration ledger before
  and after major phases.
- Use review-agent review before PR creation and address or explicitly reject
  findings with rationale.

## Iteration Completion Criteria

Iteration 01 is complete only when:

- all eight linked goals are complete;
- the Iteration 01 ledger links each goal's evidence;
- each goal's required validation commands pass or have documented,
  owner-approved exceptions;
- review tasks R0 through R5 are complete for every goal;
- checked artifacts are reproducible from committed files;
- CI and local verification evidence are linked from the PR; and
- the next roadmap iteration is created or identified without marking the broader
  Assura roadmap complete.

## Current Slice

Continue with Goal 02, Policy Language Completeness:

- audit supported structure policy fields against tests and docs;
- separate native LS-Lint migration parity from Assura compatibility
  extensions;
- add realistic passing and failing fixtures for direct files, directories,
  required content, forbidden content, markdown frontmatter, and generated
  output exclusions;
- improve diagnostics for naming drift, unexpected contents, missing required
  paths, and count failures;
- run the Goal 02 required validation commands and review tasks.
