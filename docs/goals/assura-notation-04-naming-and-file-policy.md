---
id: goal-assura-notation-04-naming-and-file-policy
type: goal
title: Assura notation 04 - naming and file policy
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: [goal-assura-notation-03-shorthand-and-rule-composition]
---

# Naming And File Policy

Keep common naming concise while preserving explicit scope and overrides.

## Deliverables

- Bare built-in naming as the preferred shorthand.
- Finite extension sets and compound stem-segment naming without regex.
- `exact:NAME` naming alternatives and anchored regex as the advanced escape hatch.
- Specificity-based compound-extension and literal-name overrides.

## Proof Gate

- Common config, test, story, and declaration filenames validate by segment.
- Uppercase, underscore, empty, and repeated segments fail deterministically.
- Built-in naming compiles no regex and adds no per-file selector parsing.

## Completion Evidence

Case, compound-extension, finite-set, exact-alternative, misplaced-skill, and
specificity tests pass through the canonical compiled plan.
