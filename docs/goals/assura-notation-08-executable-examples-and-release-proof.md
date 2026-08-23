---
id: goal-assura-notation-08-executable-examples-and-release-proof
type: goal
title: Assura notation 08 - executable examples and release proof
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: [goal-assura-notation-07-migration-and-diagnostics]
---

# Executable Examples And Release Proof

Promote one checked fixture across the CLI, docs, and landing experience.

## Deliverables

- One canonical `agentic-monorepo.yml` and generated applied-tree data.
- Active docs for targeting, expansion, rebasing, messages, and migration.
- Build-time classification and execution of every active Assura YAML example.
- Responsive segmented config views in intentional light and dark palettes.

## Proof Gate

- Canonical positive and negative fixtures produce only expected findings.
- No active example uses superseded notation or drifts from generated data.
- Parser/normalizer regression stays below 2%, accepted cold rows stay no slower
  than LS-Lint, and the warm loop retains its existing 2x target.
- Independent notation and visual reviews have no unresolved blockers.

## Completion Evidence

All 54 active YAML examples, the canonical positive/negative fixture, 52
responsive Playwright checks, static build, tracked performance gates, and two
independent final reviews pass without a release blocker.
