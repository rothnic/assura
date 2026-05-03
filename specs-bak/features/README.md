# Features

This directory contains BDD (Behavior-Driven Development) feature files written in Gherkin syntax.

## Purpose

Feature files define the behavior of the system from the user's perspective. They serve as:

- **Executable specifications** that describe what the system should do
- **Living documentation** that stays in sync with the code
- **Test cases** that can be automated using BDD frameworks
- **Communication tools** between stakeholders, developers, and testers

## Format

Each `.feature` file follows Gherkin syntax:

```gherkin
Feature: Feature Title
  As a [role]
  I want [capability]
  So that [benefit]

  Background:
    Given [common precondition]

  Scenario: Scenario description
    Given [initial context]
    When [action taken]
    Then [expected outcome]

  Scenario Outline: Parameterized scenario
    Given [context with <parameter>]
    When [action with <value>]
    Then [outcome with <result>]

    Examples:
      | parameter | value | result |
      | example1  | 1     | pass   |
      | example2  | 2     | fail   |
```

## Naming Conventions

- Use `kebab-case.feature` for file names
- Files should be organized by feature area (e.g., `validation/`, `reporting/`)
- Each feature file should focus on one capability or user story

## Traceability

- Link to related ADRs: `[[adr/ADR-XXX-decision]]`
- Link to requirements: `[[requirements/REQ-XXX]]`
- Reference test files in comments: `# Test: tests/integration/test_feature.rs`

## Examples

See `doc-validation.feature` for a complete example with traceability links.
