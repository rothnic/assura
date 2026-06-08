---
id: goal-assura-rule-module-topology
type: goal
title: Assura module topology rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Module Topology Rule

## Objective

Generalize module topology validation beyond explicit directory allowlists so
Assura can detect abandoned module families and overly broad public surfaces.

## Detector Hypothesis

Parse Rust module declarations and directory trees, compare them to configured
ownership/status categories, and flag public modules whose status conflicts
with support-policy rows.

## Required Examples

- Passing: current-product modules under `src/cli/check`.
- Passing: experimental modules explicitly labeled unstable.
- Failing: public unsupported module without an experimental/internal marker.

## Tests

Add Rust fixture trees with public, private, experimental, and abandoned module
families plus CLI integration coverage.
