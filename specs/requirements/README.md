# Requirements

This directory contains traceable requirements for the Assura validation engine.

## Purpose

Requirements documents capture:

- **Functional requirements**: What the system must do
- **Non-functional requirements**: Quality attributes (performance, security, usability)
- **Constraints**: Limitations on the solution space
- **Traceability links**: Connections to features, tests, and design documents

Requirements serve as the foundation for verification and validation activities.

## Format

Each requirement file uses structured YAML front matter with Markdown content:

```yaml
---
type: requirement
title: REQ-XXX: Requirement Title
status: draft | proposed | approved | implemented | verified
category: functional | non-functional | constraint
priority: critical | high | medium | low
tags:
  - validation
  - performance
related:
  - '[[features/related-feature]]'
  - '[[adr/ADR-XXX]]'
---

# REQ-XXX: Requirement Title

## Description

Clear, unambiguous description of what must be implemented.

## Acceptance Criteria

- [ ] Criterion 1: Measurable condition for success
- [ ] Criterion 2: Measurable condition for success
- [ ] Criterion 3: Measurable condition for success

## Rationale

Why this requirement exists and why it matters.

## Dependencies

- Depends on: [[REQ-YYY]]
- Blocks: [[REQ-ZZZ]]

## Test References

- Unit tests: `tests/unit/module_test.rs`
- Integration tests: `tests/integration/feature_test.rs`
- E2E tests: `tests/e2e/scenario_test.rs`
```

## Requirements ID Format

- **REQ-XXX**: Sequential requirement identifier
- Categories: REQ (general), PER (performance), SEC (security), USR (usability)

## Traceability

Requirements should link to:
- Feature files that implement them: `[[features/feature-name]]`
- ADRs that informed the design: `[[adr/ADR-XXX]]`
- Test files that verify them

## Examples

See existing requirement files for reference patterns.
