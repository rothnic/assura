---
id: roadmap-iteration-02-policy-depth-and-ecosystem
type: roadmap_iteration
title: Assura roadmap iteration 02 policy depth and ecosystem
status: planned
created: 2026-06-02
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/roadmap.md
  - docs/release-candidate-checklist.md
  - docs/support-policy.md
  - docs/compatibility-and-surface.md
---

# Assura Roadmap Iteration 02: Policy Depth And Ecosystem

## Objective

Plan the next bounded roadmap iteration after Iteration 01. This document is
planned, not active. It prevents the Iteration 01 completion from being
mistaken for completion of the full product roadmap.

Iteration 02 should deepen Assura's policy coverage and ecosystem fit while
preserving the stable public surface proven in Iteration 01.

## Activation Criteria

Activate this iteration only after:

- all Iteration 01 goals are completed;
- the v0.1.0 release readiness checklist is satisfied or an explicit release
  exception is recorded;
- `.trellis/spec/assura/roadmap.md` is updated to make Iteration 02 active;
- a Trellis task is created to own Iteration 02 execution; and
- the previous active Iteration 01 task is archived or marked complete.

Current handoff state: Iteration 01 is complete, Goal 08 merged in PR #25, and
the Iteration 01 Trellis task is archived under
`.trellis/tasks/archive/2026-06/06-01-roadmap-phase-01-execution`. Iteration 02
still needs an explicit activation PR or task before implementation starts.

## Direction Locks

- Keep `assura check --format agent` as the shared agent feedback format.
- Keep Codex delivery on `--agent codex`.
- Do not add package feedback CLIs, per-agent CLI entrypoints, or per-agent
  `--format` values.
- Do not add remote plugin loading or a marketplace without a separate security
  and support-policy goal.

## Planned Goal Sequence

| Order | Goal Theme | Major Outcome | Review Gate |
| --- | --- | --- | --- |
| 1 | Watch Mode Hardening | `assura watch` becomes release-grade with deterministic debounce, cancellation, and docs. | Cross-platform watch tests and failure-mode review |
| 2 | Policy Expressiveness | Structure policy covers the next highest-value real repo contracts without ad hoc scripts. | Good/base/bad fixtures and migration boundary review |
| 3 | Compatibility Expansion | LS-Lint migration claims expand only where tests prove parity or explicit differences. | Fixture corpus and docs claim review |
| 4 | Extension Authoring | First-party custom constraints gain authoring guidance, examples, and compatibility labels. | Safety, determinism, and support-level review |
| 5 | Ecosystem Integrations | CI templates, editor task snippets, and project bootstrap docs improve adoption without hidden services. | First-run user journey review |
| 6 | Release Operations | Release automation gains repeatable changelog, asset verification, and post-release triage loops. | Release rehearsal and rollback review |

Each goal should be written as a two-week team chunk with objective, scope,
definition of done, validation commands, review tasks, and blocking criteria
before implementation starts.

## Non-Goals

- No hosted SaaS launch.
- No 1.0 semantic-versioning commitment.
- No plugin marketplace.
- No dependency graph validation claim unless a dedicated goal first defines
  the supported contract and tests.

## Handoff Prompt

When maintainers choose to start this iteration, use:

```text
/goal docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
```
