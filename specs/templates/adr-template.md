---
type: template
title: Architecture Decision Record Template
created: YYYY-MM-DD
tags:
  - template
  - adr
  - architecture
related:
  - '[[doc-template]]'
  - '[[feature-template]]'
  - '[[../adr/README]]'
---

# ADR-XXX: [REQUIRED: Short Title of the Decision]

- **Status**: [REQUIRED: proposed | accepted | rejected | deprecated | superseded by [[ADR-YYY]]]
- **Date**: YYYY-MM-DD
- **Deciders**: [List everyone involved in the decision]
- **Category**: [architecture | technology | process | data-model]

## Context

[REQUIRED: What is the issue that we're seeing that is motivating this decision or change?]

[Describe the forces at play, including technological, political, social, and project-specific forces.]

## Decision

[REQUIRED: What is the change that we're proposing or have agreed to implement?]

[State the decision clearly and unambiguously. This should be a single paragraph or a few bullet points.]

## Consequences

[REQUIRED: What becomes easier or more difficult to do because of this change?]

### Positive

- [Benefit 1]
- [Benefit 2]

### Negative

- [Drawback 1]
- [Drawback 2]

### Risks

- [Risk 1: mitigation strategy]
- [Risk 2: mitigation strategy]

## Alternatives Considered

[REQUIRED: Briefly describe the alternatives that were considered and why they were rejected.]

### Alternative 1: [Name]

- **Description**: [What was it?]
- **Pros**: [Benefits of this approach]
- **Cons**: [Why it was rejected]
- **Verdict**: [Why it didn't win]

### Alternative 2: [Name]

- **Description**: [What was it?]
- **Pros**: [Benefits of this approach]
- **Cons**: [Why it was rejected]
- **Verdict**: [Why it didn't win]

## Related Decisions

[Use wiki-links to connect to related ADRs or documents:]

- [[ADR-001-example]] - Brief description of relationship
- [[../requirements/some-requirement]] - Requirement that drove this decision
- [[doc-template]] - Related documentation

## Implementation Notes

[Optional: Any specific implementation details, migration steps, or code references.]

## References

- [External reference](https://example.com)
- [[related-spec]] - Internal specification

---

**Template Usage:**
- Replace `ADR-XXX` with the next sequential ADR number
- Fill in all `[REQUIRED: ...]` sections
- Keep the decision section concise and clear
- Always document alternatives considered
- Update the `related` section in front matter with connected documents
