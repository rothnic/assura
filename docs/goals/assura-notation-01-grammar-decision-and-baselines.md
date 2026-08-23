---
id: goal-assura-notation-01-grammar-decision-and-baselines
type: goal
title: Assura notation 01 - grammar decision and baselines
status: completed
parent: goal-assura-config-notation-rule-composition-and-site-alignment
depends_on: []
---

# Grammar Decision And Baselines

Lock Option A before implementation. `structure:` is the implicit project
root, `./` means the current directory inside a tree, explicit selectors own
their reach, and native inherited extension shorthand is removed.

## Deliverables

- Canonical selector, cardinality, precedence, rebasing, and target-kind tables.
- Accepted homepage target fixture and positive/negative project trees.
- Parser, normalization, cold-check, compiled-check, and warm-loop baselines.
- Explicit migration boundary for legacy Assura and LS-Lint input.

## Proof Gate

- Every public selector has one positive and one negative example.
- Target YAML parses as YAML and is marked unsupported until Goal 02 lands.
- Baseline commands and results are recorded in the parent progress log.

## Completion Evidence

The accepted selector and type contracts are recorded in the governing Trellis
spec and parent goal; cold and warm baselines remain tracked benchmark data.
