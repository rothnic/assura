# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records documenting significant architectural choices.

## Purpose

ADRs capture:

- **Context**: The problem or situation requiring a decision
- **Decision**: The choice that was made
- **Consequences**: The resulting trade-offs (positive and negative)

ADRs provide a historical record of why the codebase looks the way it does, enabling future developers to understand the rationale behind key architectural choices.

## Format

Each ADR follows a standardized template:

```markdown
---
type: adr
title: ADR-XXX: Decision Title
status: proposed | accepted | deprecated | superseded
superseded_by: ADR-YYY (if applicable)
decision_date: YYYY-MM-DD
tags:
  - architecture
  - decision
related:
  - '[[adr/ADR-XXX-other]]'
  - '[[features/related-feature]]'
---

# ADR-XXX: Decision Title

## Status

proposed | accepted | deprecated | superseded by [[ADR-YYY]]

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing or have agreed to implement?

## Consequences

What becomes easier or more difficult to do because of this change?

### Positive
- Benefit 1
- Benefit 2

### Negative
- Trade-off 1
- Trade-off 2

## References

- Link to relevant discussions
- Link to external documentation
```

## Naming Conventions

- Use `ADR-XXX-kebab-case-title.md` format
- Sequential numbering (ADR-001, ADR-002, etc.)
- Descriptive title in kebab-case

## Lifecycle

1. **Proposed**: Decision is being considered
2. **Accepted**: Decision has been made and implemented
3. **Deprecated**: Decision is no longer relevant
4. **Superseded**: A newer ADR replaces this one

## Examples

See existing ADRs in this directory for reference patterns.
