---
id: goal-assura-notation-03-shorthand-and-rule-composition
type: goal
title: Assura notation 03 - shorthand and typed rule composition
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: [goal-assura-notation-02-root-and-selector-model]
---

# Shorthand And Typed Rule Composition

Make concise scalars and reusable rules normalize into one typed policy model.

## Deliverables

- Bare naming, `directive:value`, `$rule`, and top-level ` | ` composition.
- Node-rule and tree-rule target checking, cycle detection, and bounded expansion.
- Tree-rule rebasing equivalent to copying its selectors at the application point.
- Authored-rule provenance and advisory zero/one-use alias diagnostics.

## Proof Gate

- Every shorthand form has an expanded-form equivalence test.
- Regex alternation is never split as directive composition.
- Invalid target kinds fail before filesystem traversal with actionable context.

## Completion Evidence

Focused notation tests cover shorthand equivalence, left-to-right overrides,
regex pipes, rule cycles, typed targets, provenance, and reuse diagnostics.
