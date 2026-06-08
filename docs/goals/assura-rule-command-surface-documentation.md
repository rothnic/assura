---
id: goal-assura-rule-command-surface-documentation
type: goal
title: Assura command surface documentation rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Command Surface Documentation Rule

## Objective

Create a configurable rule that detects documented Assura commands, flags, and
formats that are not present in the live CLI surface or support matrix.

## Detector Hypothesis

Parse command examples from configured docs paths, normalize `assura` and
`cargo run -- ... assura` invocations, compare flags/formats against generated
CLI help or a checked command contract, and allow explicit unsupported/future
classifications.

## Required Examples

- Passing: `assura check --format agent --agent codex .`
- Failing: `assura check --format codex-hook .`
- Failing: `assura check --maturity .`

## Tests

Add passing/failing fixtures, CLI integration coverage, and a self-check config
example for this repo.
