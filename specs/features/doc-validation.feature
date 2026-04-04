---
type: feature
title: Documentation Validation Feature
created: 2026-04-04
tags:
  - bdd
  - feature
  - validation
  - documentation
  - markdown
related:
  - '[[../templates/feature-template]]'
  - '[[../../.maestro/playbooks/Initiation/Working/validation-report]]'
  - '[[../../../tests/markdown_tests]]'
  - '[[../../../tests/constraint_tests]]'
---

# Feature: Documentation Quality Validation

As a developer or technical writer, I want documentation to meet quality standards so that it is consistent, maintainable, and easy to navigate. Assura validates documentation files against configurable rules to ensure quality.

## Background

Assura provides comprehensive documentation validation including:
- **Frontmatter validation**: YAML metadata at the top of markdown files
- **Heading structure validation**: H1 requirements, hierarchy, and depth limits
- **Template enforcement**: Required sections and ordering
- **Line limit checking**: File size constraints for readability
- **Naming convention enforcement**: Consistent file naming patterns

```gherkin
Given a markdown file exists in the project
And Assura is configured with documentation validation rules
```

## Scenarios

### Scenario: Markdown file has valid frontmatter

**Traceability**: Tests requirement REQ-001 (Documentation must have metadata)

**Related Tests**:
- `tests/markdown_tests.rs::frontmatter_tests::test_frontmatter_required_fields`
- `tests/markdown_tests.rs::frontmatter_tests::test_frontmatter_type_validation`

```gherkin
Given a markdown file with the following content:
  """
  ---
  title: Test Document
  author: John Doe
  date: 2024-01-15
  ---

  # Test Document

  Some content here.
  """
And the frontmatter schema requires fields: title, author, date
When Assura validates the file
Then the validation should pass
And no violations should be reported
```

### Scenario: Markdown file is missing required frontmatter field

**Traceability**: Tests requirement REQ-001 (Documentation must have metadata)

**Related Tests**:
- `tests/markdown_tests.rs::frontmatter_tests::test_frontmatter_missing_required_field`

```gherkin
Given a markdown file with the following content:
  """
  ---
  title: Test Document
  ---

  # Test Document
  """
And the frontmatter schema requires fields: title, author
When Assura validates the file
Then the validation should fail
And a violation should be reported for missing field "author"
And the severity should be "High"
```

### Scenario: Markdown file has frontmatter with invalid type

**Traceability**: Tests requirement REQ-002 (Frontmatter fields must have correct types)

**Related Tests**:
- `tests/markdown_tests.rs::frontmatter_tests::test_frontmatter_type_validation`

```gherkin
Given a markdown file with the following content:
  """
  ---
  count: "not a number"
  ---

  # Test
  """
And the frontmatter schema defines "count" as type: Integer
When Assura validates the file
Then the validation should fail
And a violation should be reported for field "count"
And the error message should indicate "integer" type required
```

### Scenario: Markdown file has frontmatter with invalid email format

**Traceability**: Tests requirement REQ-002 (Frontmatter fields must have correct types)

**Related Tests**:
- `tests/markdown_tests.rs::frontmatter_tests::test_frontmatter_email_validation`

```gherkin
Given a markdown file with the following content:
  """
  ---
  contact: invalid-email
  ---

  # Test
  """
And the frontmatter schema defines "contact" as type: Email
When Assura validates the file
Then the validation should fail
And a violation should be reported for field "contact"
And the error message should indicate "email" format required
```

### Scenario: Markdown file exceeds maximum line limit

**Traceability**: Tests requirement REQ-003 (Documentation must be maintainable size)

**Related Tests**:
- `tests/constraint_tests.rs::test_file_size_constraint_with_large_file`
- `tests/markdown_tests.rs::template_tests::test_template_section_word_count`

```gherkin
Given a markdown file with 600 lines of content
And the validation rule sets max_lines to 500 for documentation files
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating line limit exceeded
And the violation should suggest splitting the file
```

### Scenario: Markdown file naming uses kebab-case convention

**Traceability**: Tests requirement REQ-004 (Files must follow naming conventions)

**Related Tests**:
- `tests/constraint_tests.rs::test_naming_constraint_kebab_case`
- `tests/constraint_tests.rs::test_general_naming_config`

```gherkin
Given a file named "my-document.md"
And the naming convention is set to "kebab-case"
When Assura validates the file
Then the validation should pass
```

### Scenario: Markdown file naming violates kebab-case convention

**Traceability**: Tests requirement REQ-004 (Files must follow naming conventions)

**Related Tests**:
- `tests/constraint_tests.rs::test_naming_constraint_kebab_case`

```gherkin
Given a file named "my_document.md"
And the naming convention is set to "kebab-case"
When Assura validates the file
Then the validation should fail
And a violation should be reported for naming convention violation
```

### Scenario: Markdown file naming uses camelCase convention

**Traceability**: Tests requirement REQ-004 (Files must follow naming conventions)

**Related Tests**:
- `tests/constraint_tests.rs::test_naming_constraint_kebab_case`

```gherkin
Given a file named "myFile.md"
And the naming convention is set to "kebab-case"
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating "camelCase" is not allowed
```

### Scenario: Markdown file has valid heading hierarchy

**Traceability**: Tests requirement REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::heading_tests::test_heading_hierarchy_valid`

```gherkin
Given a markdown file with the following content:
  """
  # Title

  ## Section 1

  Some content.

  ### Subsection

  More content.

  ## Section 2

  Final content.
  """
And the heading validator requires H1
And the heading validator validates hierarchy
When Assura validates the file
Then the validation should pass
```

### Scenario: Markdown file has missing H1 heading

**Traceability**: Tests requirement REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::heading_tests::test_heading_missing_h1`

```gherkin
Given a markdown file with the following content:
  """
  ## Section

  Some content.
  """
And the heading validator requires H1
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating "H1" is required
```

### Scenario: Markdown file has multiple H1 headings

**Traceability**: Tests requirement REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::heading_tests::test_heading_multiple_h1`

```gherkin
Given a markdown file with the following content:
  """
  # Title 1

  # Title 2

  Some content.
  """
And the heading validator requires single H1
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating only one H1 allowed
```

### Scenario: Markdown file has skipped heading level

**Traceability**: Tests requirement REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::heading_tests::test_heading_skipped_level`

```gherkin
Given a markdown file with the following content:
  """
  # Title

  ### Section

  Content.
  """
And the heading validator validates hierarchy
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating "Skipped" heading level
```

### Scenario: Markdown file exceeds maximum heading depth

**Traceability**: Tests requirement REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::heading_tests::test_heading_max_depth`

```gherkin
Given a markdown file with the following content:
  """
  # Title

  ## Section

  ### Subsection

  #### Deep Section

  Content.
  """
And the heading validator has max_depth set to 3
When Assura validates the file
Then the validation should fail
And a violation should be reported indicating "maximum depth" exceeded
```

## Edge Cases

### Scenario: Empty markdown file

**Traceability**: Edge case for REQ-001 (Documentation must have metadata)

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_empty_markdown`

```gherkin
Given an empty markdown file
When Assura parses the file
Then the document should have no headings
And the document should have no frontmatter
And the word count should be 0
```

### Scenario: Markdown file with only frontmatter

**Traceability**: Edge case for REQ-001 (Documentation must have metadata)

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_markdown_only_frontmatter`

```gherkin
Given a markdown file with the following content:
  """
  ---
  title: Only Frontmatter
  ---
  """
When Assura parses the file
Then the document should have frontmatter
And the document should have no headings
```

### Scenario: Markdown file with invalid YAML frontmatter

**Traceability**: Edge case for REQ-002 (Frontmatter fields must have correct types)

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_invalid_yaml_frontmatter`

```gherkin
Given a markdown file with the following content:
  """
  ---
  invalid: yaml: [
  ---

  # Test
  """
When Assura validates the file with frontmatter required
Then the validation should fail
And a violation should be reported for invalid YAML
```

### Scenario: Markdown file with headings inside code blocks

**Traceability**: Edge case for REQ-005 (Documentation must have proper structure)

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_nested_code_blocks`

```gherkin
Given a markdown file with the following content:
  """
  # Document with Code

  ```markdown
  # This is a markdown heading inside a code block
  ## Another heading
  ```

  ## Real Section

  Actual content.
  """
When Assura parses the file
Then only 2 headings should be counted
And the headings should be "Document with Code" and "Real Section"
```

## Error Handling

### Scenario: Markdown file has multiple validation errors

**Traceability**: Error handling for comprehensive validation

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_multiple_validation_errors`

```gherkin
Given a markdown file with the following content:
  """
  ---
  title: Test
  ---

  ## Section 1

  ### Too Deep

  Content.

  # Another H1

  More content.
  """
And the heading validator requires single H1
And the heading validator validates hierarchy
And the heading validator has max_depth set to 2
When Assura validates the file
Then multiple violations should be reported
And violations should include: multiple H1, skipped level, max depth exceeded
```

### Scenario: Markdown file with nonexistent schema reference

**Traceability**: Error handling for schema configuration

**Related Tests**:
- `tests/markdown_tests.rs::end_to_end_tests::test_constraint_with_nonexistent_schema`

```gherkin
Given a markdown file with valid content
And the constraint references a nonexistent schema "nonexistent"
When Assura validates the file
Then the validation should return an error
And the error should indicate schema not found
```

## Related Features

- [[feature-template]] - Template for creating new feature files
- [[../adr/ADR-001-spec-structure]] - Architecture decision on spec-based development
- [[../../../.assura/config]] - Assura configuration file defining validation rules

## Test Implementation Notes

### Unit Tests
- `tests/markdown_tests.rs` - Frontmatter, heading, and template validation tests
- `tests/constraint_tests.rs` - Naming convention and file size constraint tests

### Integration Tests
- `tests/markdown_tests.rs::end_to_end_tests` - End-to-end markdown validation scenarios

### E2E Tests
- CLI integration tests (planned for Phase 03)

## Tags

@documentation @validation @markdown @frontmatter @naming @quality

### Tag Categories
- `@fast` - Quick unit-level scenarios (all scenarios in this feature)
- `@integration` - Requires external dependencies (not applicable)
- `@e2e` - Full end-to-end test (not applicable yet)
- `@smoke` - Critical path smoke tests

---

**Feature Completion Criteria:**
- [x] Feature file created with Gherkin scenarios
- [x] Scenarios cover frontmatter validation
- [x] Scenarios cover line limit checking
- [x] Scenarios cover naming convention enforcement
- [x] Scenarios cover heading structure validation
- [x] Traceability comments link to existing test files
- [x] Wiki-links connect to related documents
