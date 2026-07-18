---
id: goal-assura-notation-06-project-owned-agentic-recipes
type: goal
title: Assura notation 06 - project-owned agentic recipes
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: [goal-assura-notation-05-structure-cardinality-and-repair-context]
---

# Project-Owned Agentic Recipes

Materialize selected agentic policy into editable project YAML.

## Deliverables

- `agentic-core` and `structure-health` recipe sources.
- Required root and workspace guidance plus optional strict project skills.
- `assura init --recipe agentic-core` and additive
  `assura config add-recipe agentic-core` flows.
- Conflict preview, explicit opt-out, and no runtime recipe-catalog dependency.

## Proof Gate

- Generated projects validate with the recipe catalog unavailable.
- Every existing package requires `AGENTS.md` and `package.json`.
- Every existing skill requires bounded `SKILL.md` and declared resources.

## Completion Evidence

Init and additive recipe tests validate materialized YAML, preserve existing
files, and create the project-owned repair documents referenced by each recipe.
