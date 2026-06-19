---
id: roadmap-iteration-02-policy-depth-and-ecosystem
type: roadmap_iteration
title: Assura roadmap iteration 02 policy depth and ecosystem
status: completed
created: 2026-06-02
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - docs/goals/assura-goal-09-first-time-configuration-authoring.md
  - docs/goals/assura-goal-10-relationship-semantics-hardening.md
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - docs/goals/assura-goal-12-self-enforcing-support-and-test-matrix.md
  - docs/goals/assura-goal-13-performance-and-release-evidence-governance.md
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

Iteration 02 deepened Assura's policy coverage and ecosystem fit while
preserving the stable public surface proven in Iteration 01.

The product goal is to stop hoping stability happens. Iteration 02 should make
new-user success deterministic: users should be able to install Assura, write a
useful config, understand failures, and trust documented support and release
claims because the repo enforces those promises.

## Activation Criteria

This iteration was activated and completed after:

- all Iteration 01 goals are completed;
- the v0.1.0 release readiness checklist is satisfied or an explicit release
  exception is recorded;
- `.trellis/spec/assura/roadmap.md` is updated to make Iteration 02 active;
- a Trellis task is created to own Iteration 02 execution; and
- the previous active Iteration 01 task is archived or marked complete.

Completion state: Goals 09 through 13 are merged and archived under
`.trellis/tasks/archive/2026-06/`, ending with Goal 13 PR #55 and archive PR
#56. The next roadmap handoff is recorded in `.trellis/spec/assura/roadmap.md`.

## Direction Locks

- Keep `assura check --format agent` as the shared agent feedback format.
- Keep Codex delivery on `--agent codex`.
- Do not add package feedback CLIs, per-agent CLI entrypoints, or per-agent
  `--format` values.
- Do not add remote plugin loading or a marketplace without a separate security
  and support-policy goal.
- Any notation update must carry performance gates unless the goal documents an
  inherent notation-driven cost, proves the cost is bounded, and explains why
  the user value justifies it.
- Any notation update must update public examples, website examples, generated
  examples, fixture configs, and test-case `.assura/config.yml` files that
  teach or exercise the changed notation.
- Do not preserve backwards compatibility for superseded alpha notation unless
  a support-policy exception and removal plan are explicit.
- Markdown-related goals must evaluate maintained Markdown linting,
  frontmatter, parser/AST, and link-checking tooling before building generic
  checks in Assura.
- Goal creation and stale-goal execution should use the
  `assura-goal-validation` skill when a goal is old, produced in a separate
  context, or possibly superseded by current repo state.
- Iteration 02 notation work must end with a checked use-case matrix that
  starts from LS-Lint-equivalent policies, extends into Assura-native notation,
  and is independently reviewed for modularity and performance.

## Current Gap

Iteration 01 proved an adoption foundation, and PR #49 proved canonical
relationship notation. Iteration 02 then closed the planned proof gaps for
first-time configuration authoring, relationship semantics, Markdown outline
validation, self-enforcing support/test matrices, and release/performance
evidence. Future work should start from the current roadmap rather than
reopening Goals 09 through 13 as planned work.

## Planned Goal Sequence

| Order | Goal Theme | Major Outcome | Review Gate |
| --- | --- | --- | --- |
| 1 | [First-Time Configuration Authoring](./assura-goal-09-first-time-configuration-authoring.md) | New users can write useful Assura config from docs without reading source or historical notation, backed by a notation use-case matrix. | First-run and notation-matrix review |
| 2 | [Relationship Semantics Hardening](./assura-goal-10-relationship-semantics-hardening.md) | Capture-driven relationships are predictable, well-diagnosed, and safe to build on. | Relationship fixture and diagnostics review |
| 3 | [Markdown Outline Validation](./assura-goal-11-markdown-outline-validation.md) | Markdown outline notation works like the config spec without manual heading-depth sync, after a tooling fit decision proves what Assura should adopt versus own. | Markdown tooling, fixture, and docs-claim review |
| 4 | [Self-Enforcing Support And Test Matrix](./assura-goal-12-self-enforcing-support-and-test-matrix.md) | Supported commands, public exports, manifests, docs, and tests cannot drift silently. | Support-matrix and test-relationship review |
| 5 | [Install, Release, And Performance Certainty](./assura-goal-13-performance-and-release-evidence-governance.md) | New-user install, performance, and release claims are current, reproducible, and checked by deterministic gates, including LS-Lint-equivalent notation performance. | Install smoke, performance evidence, notation efficiency, and release-sync review |

Each goal was executed through a Trellis task and archived after merge. Keep
these links as historical routing, not as the next active work queue.

Watch-mode hardening, compatibility expansion beyond these goals, and broader
ecosystem integrations remain valid future Iteration 02 candidates, but they
should not displace the notation and self-enforcement proof gaps left after the
canonical relationship notation work.

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

## Progress Log

- 2026-06-18: Reviewed the Iteration 02 planning slice against current repo
  state and tightened notation gates for public examples, support-policy
  exceptions, performance proof, and independent modularity review.
- 2026-06-19: Completed Iteration 02 execution. Goals 09, 10, 11, 12, and 13
  are merged and archived under `.trellis/tasks/archive/2026-06/`. Goal 13
  merged in PR #55 and its archive move merged in PR #56.
