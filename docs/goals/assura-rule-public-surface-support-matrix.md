---
id: goal-assura-rule-public-surface-support-matrix
type: goal
title: Assura public surface support matrix rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Public Surface Support Matrix Rule

## Objective

Create a configurable support-matrix rule that compares exported APIs,
documented commands, experimental surfaces, and unsupported claims.

## Detector Hypothesis

Parse configured support-policy tables and source exports, then require every
public command/API family to be classified as supported, experimental,
internal, roadmap, or unsupported.

## Required Examples

- Passing: `assura check --format agent` classified supported.
- Passing: Rust `intelligence` exports classified unstable internal.
- Failing: dependency graph validation documented as supported without a
  support-policy row.

## Tests

Add support-matrix fixtures, source export fixtures, docs-claim fixtures, and
CLI integration coverage.
