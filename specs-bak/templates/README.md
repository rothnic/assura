# Templates

This directory contains reusable templates for creating specifications and documentation.

## Purpose

Templates ensure consistency across all project documentation by providing:

- **Standardized structure** for common document types
- **Required fields** that must be completed
- **Examples** showing proper formatting
- **Guidance** on what content to include

## Available Templates

| Template | Purpose | Usage |
|----------|---------|-------|
| `doc-template.md` | General documentation | Use for guides, references, and notes |
| `adr-template.md` | Architecture decisions | Use when proposing or recording ADRs |
| `feature-template.md` | BDD feature files | Use when defining new features |

## Template Usage

1. Copy the appropriate template to your target directory
2. Rename following the naming conventions for that document type
3. Fill in all required fields (marked with `[REQUIRED]`)
4. Replace example content with your actual content
5. Remove instructional comments as you complete each section
6. Update the YAML front matter with accurate metadata

## Creating New Templates

When adding a new template:

1. Follow the established YAML front matter structure
2. Include clear usage instructions in comments
3. Provide realistic examples
4. Document the template in this README
5. Link related templates using wiki-links

## Format

All templates should include:

- YAML front matter with type, title, and metadata
- Clear section headers
- Placeholder markers like `[REQUIRED]` or `[DESCRIPTION HERE]`
- Example content showing proper formatting
- Comments explaining less obvious fields

## Examples

See individual template files for specific usage patterns:
- [[doc-template.md]]
- [[adr-template.md]]
- [[feature-template.md]]
