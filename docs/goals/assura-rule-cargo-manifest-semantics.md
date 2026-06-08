---
id: goal-assura-rule-cargo-manifest-semantics
type: goal
title: Assura Cargo manifest semantics rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Cargo Manifest Semantics Rule

## Objective

Create configurable Cargo manifest checks for claims, features, workspace
members, bins, and metadata that Assura can validate without replacing Cargo
or specialist tools.

## Detector Hypothesis

Parse `Cargo.toml` structurally, compare selected fields against configured
support claims, feature-policy allowlists, required binary metadata, and
forbidden release-positioning phrases.

## Required Examples

- Passing: structure-first package description and `structure` keyword.
- Failing: package description claiming dependency graph validation.
- Failing: optional dependency listed in a feature but unused by code.

## Tests

Add TOML fixtures, config examples, and integration tests that avoid ad hoc
string matching where a TOML parser is available.
