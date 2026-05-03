---
type: template
title: BDD Feature File Template
created: YYYY-MM-DD
tags:
  - template
  - bdd
  - feature
  - test
related:
  - '[[doc-template]]'
  - '[[adr-template]]'
  - '[[../features/README]]'
---

# Feature: [REQUIRED: Feature Name]

[Provide a brief description of what this feature does and why it matters. This helps readers understand the business value.]

## Background

[Optional: Any setup or context that applies to all scenarios in this feature.]

```gherkin
Given [some initial context that applies to multiple scenarios]
```

## Scenarios

### Scenario: [REQUIRED: Descriptive scenario name]

**Traceability**: Tests requirement [[../requirements/REQ-001-example]]

```gherkin
Given [a precondition or initial state]
And [another precondition if needed]
When [an action is performed]
Then [an expected outcome occurs]
And [another expected outcome if needed]
```

**Related Code**: `src/module/file.rs::function_name`

### Scenario: [Another descriptive scenario name]

**Traceability**: Tests requirement [[../requirements/REQ-002-example]]

```gherkin
Given [a different initial state]
When [a different action is performed]
Then [a different expected outcome]
```

**Related Code**: `tests/integration/test_file.rs`

## Edge Cases

### Scenario: [Edge case description]

**Traceability**: Edge case for [[../requirements/REQ-001-example]]

```gherkin
Given [edge case precondition]
When [action that triggers edge case]
Then [expected handling of edge case]
```

## Error Handling

### Scenario: [Error condition]

```gherkin
Given [precondition that will lead to error]
When [action that causes error]
Then [expected error message or behavior]
And [any additional error expectations]
```

## Related Features

[Use wiki-links to connect related features or specifications:]

- [[feature-validation]] - Related validation feature
- [[adr-template]] - Architecture decision affecting this feature
- [[../adr/ADR-001-example]] - Architecture decision this implements

## Test Implementation Notes

- **Unit tests**: `tests/unit/module_test.rs`
- **Integration tests**: `tests/integration/feature_test.rs`
- **E2E tests**: `tests/e2e/feature_test.rs`

## Tags

[@tag1 @tag2 @slow @integration]

Use tags to categorize scenarios:
- `@fast` - Quick unit-level scenarios
- `@integration` - Requires external dependencies
- `@e2e` - Full end-to-end test
- `@wip` - Work in progress
- `@smoke` - Critical path smoke tests

---

**Template Usage:**
- Replace `[REQUIRED: Feature Name]` with a clear, concise feature name
- Write scenarios in the "Given-When-Then" format
- Always include traceability links to requirements using wiki-links
- Reference actual test file locations in code
- Use specific, concrete examples rather than abstract descriptions
- Remove scenario sections that aren't needed for your feature
- Update the `related` section in front matter with connected documents
