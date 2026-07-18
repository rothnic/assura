---
id: goal-assura-notation-02-root-and-selector-model
type: goal
title: Assura notation 02 - root and selector model
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: [goal-assura-notation-01-grammar-decision-and-baselines]
---

# Root And Selector Model

Implement an implicit root and an explicit compiled selector model.

## Deliverables

- `./`, `./*`, `./*/`, `./**/*`, and `./**/` file/directory semantics.
- Direct nested selectors rebased onto each matched directory exactly once.
- A compiled selector representation reused by check and explain.
- Removal of context-dependent native `.md` and `.ts` inheritance.

## Proof Gate

- Root and descendants match exactly once with no duplicate findings.
- Reordered YAML compiles to equivalent selector precedence.
- Direct and compiled checks return identical findings.

## Completion Evidence

Compiled scope-pattern and direct/compiled equivalence tests cover root,
literal, capture, wildcard, and recursive rebasing behavior.
