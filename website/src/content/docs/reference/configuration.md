---
title: Configuration Reference
description: Supported configuration fields for Assura
template: doc
sidebar:
  order: 1
---

Assura discovers configuration from `.assura/config.yml` by default. The public
validation path is:

```bash
assura check
```

## Discovery

Recommended:

```text
.assura/config.yml
```

The CLI can also receive a config path with the global `--config` option.

## Structure

```yaml
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
exclude:
  - "target/**"
```

## Naming Conventions

Common naming values include `kebab-case`, `snake_case`, `PascalCase`, and
`regex:<pattern>`.

## Exact Names and Closed-World Rules

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

## Existence Counts

```yaml
structure:
  ./:
    files:
      exists:
        "README.md": "1"
        "*.tmp": "0"
    directories:
      exists:
        "package-*": "1-5"
```

Existence count rules apply to direct children.

## Report Formats

```bash
assura check --format text
assura check --format json .
assura check --format yaml .
```

The JSON report contains `success`, `project_root`, `config_path`,
`checked_path`, `files_checked`, `dirs_checked`, and `violations`.
