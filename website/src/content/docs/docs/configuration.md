---
title: Configuration
description: Supported Assura structure-first configuration
---

Assura uses `.assura/config.yml` as the recommended configuration file. The
current pre-1.0 product surface is structure-first: configuration mirrors the
repository shape that Assura should allow.

## Minimal Config

```yaml
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
exclude:
  - "target/**"
  - "node_modules/**"
```

Run it with:

```bash
assura check
```

## Structure Nodes

Each key under `structure` is a directory scope. `./` represents the project
root. A node can define direct file rules, direct child directory rules,
markdown rules, existence rules, and child scopes.

```yaml
structure:
  ./:
    files:
      allowed_names:
        - README.md
        - Cargo.toml
      allow_extra: false
    directories:
      allowed_names:
        - src
        - tests
      allow_extra: false
    children:
      src/:
        files:
          naming: snake_case
          naming_patterns:
            "*.rs": snake_case
```

## File Rules

Supported file fields include:

- `naming`
- `naming_patterns`
- `max_lines`
- `max_size`
- `require_docs`
- `extensions`
- `severity`
- `required`
- `allowed_names`
- `allowed_patterns`
- `forbidden_patterns`
- `allow_extra`
- `exists`

## Directory Rules

Supported direct child directory fields include:

- `naming`
- `required`
- `allowed_names`
- `allowed_patterns`
- `forbidden_patterns`
- `allow_extra`
- `severity`
- `exists`

## Markdown Rules

Supported markdown fields include:

- `require_frontmatter`
- `max_heading_depth`
- `check_links`
- `required_sections`
- `outline`
- `lint_trailing_spaces`

Use `require_frontmatter` only for generic Markdown files that must contain a
frontmatter block. Typed frontmatter fields belong to content runtime `models`,
`collections`, and `relations`; `markdown.required_fields` is rejected so the
same field policy is not declared twice.

Set `lint_trailing_spaces: true` to report blank Markdown lines that contain
spaces or tabs. `assura fix markdown --dry-run` previews this safe whitespace
class, and `assura fix markdown --apply` removes it for configured Markdown
scopes.

`outline` uses nested YAML lists to describe required heading order. Prefix a
heading with `?? ` to make it optional, and use object form when a required
heading starts with literal question marks:

```yaml
markdown:
  outline:
    - Overview
    - ?? Prerequisites
    - Quick Start:
        - Installation
        - ?? Configuration
    - title: "?? Debug Mode"
      optional: false
```

## Existence Rules

Use `exists` to require files or directories:

```yaml
structure:
  ./:
    exists:
      files:
        - README.md
      directories:
        - src
```

Use direct child count constraints on files or directories:

```yaml
structure:
  ./:
    files:
      exists:
        "README.md": "1"
    directories:
      exists:
        "packages-*": "0-3"
```

## Closed-World Shape

Set `allow_extra: false` with allowed names or patterns to reject unexpected
files and directories:

```yaml
structure:
  ./:
    files:
      allowed_names:
        - README.md
        - Cargo.toml
      allow_extra: false
    directories:
      allowed_names:
        - src
        - tests
      allow_extra: false
```

## Output

Supported `assura check` formats are `text`, `json`, and `yaml`:

```bash
assura check --format json .
```

See [Getting Started](/guides/getting-started/) for the JSON report shape.
