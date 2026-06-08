---
id: goal-assura-rule-test-relationship
type: goal
title: Assura test relationship rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Test Relationship Rule

## Objective

Create configurable rules for test-to-code relationships, ignored tests,
fixture ownership, and roadmap-only test paths.

## Detector Hypothesis

Map configured source globs to required test globs, track ignored tests by
reason, and classify fixture families through explicit config instead of broad
exclusions.

## Required Examples

- Passing: `src/cli/check/**` has integration/unit coverage.
- Failing: ignored test without an accepted reason category.
- Failing: new `tests/fixtures/**` family not listed in config.

## Tests

Add source/test fixture trees, ignored-test examples, fixture-family examples,
and CLI integration coverage.
