# Specifications

This directory contains all specification documents for the Assura project.

## Directory Structure

```
specs/
├── features/       # BDD feature files (Gherkin syntax)
├── adr/            # Architecture Decision Records
├── requirements/   # Traceable requirements
└── templates/      # Reusable document templates
```

## Purpose

The `specs/` directory serves as the single source of truth for:

- **What** the system should do (requirements)
- **Why** architectural decisions were made (ADRs)
- **How** features should behave (BDD feature files)
- **How** to document consistently (templates)

## Quick Start

1. **Writing a new feature?** → See [[features/README.md]]
2. **Recording a decision?** → See [[adr/README.md]]
3. **Defining requirements?** → See [[requirements/README.md]]
4. **Need a template?** → See [[templates/README.md]]

## Document Standards

All documents in this directory should:

- Include YAML front matter with appropriate metadata
- Use wiki-links (`[[document-name]]`) for cross-referencing
- Follow the naming conventions documented in each subdirectory
- Be written in clear, unambiguous language

## Integration with Development

Specifications are not isolated documents—they're integrated into the development workflow:

- **Requirements** → Implemented in code
- **Features** → Verified by tests
- **ADRs** → Referenced in code comments
- **Templates** → Enforced by validation rules

## Validation

Assura validates specification documents against:

- Required front matter fields
- Naming conventions
- Line length limits
- Cross-reference validity

Run `assura check specs/` to validate all specification documents.
