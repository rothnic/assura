---
id: goal-assura-rule-release-sync
type: goal
title: Assura release sync rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Release Sync Rule

## Objective

Create a rule family that verifies release docs, install scripts, workflow
asset names, support matrix rows, and package metadata stay synchronized.

## Detector Hypothesis

Extract archive names, checksums, install URLs, version mentions, and release
workflow matrix entries from configured files, then compare them against a
single release contract.

## Required Examples

- Passing: all platform archive names match the release matrix.
- Failing: docs mention an asset that release workflow does not publish.
- Failing: install script URL points at an unsupported branch or asset name.

## Tests

Add release-contract fixtures, workflow/doc mismatch fixtures, and CLI
integration coverage.
