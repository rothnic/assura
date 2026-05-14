---
title: Structure Configuration Reference
description: Reference for the current Assura structure-first config
---

The current Assura config is structure-first. Current onboarding should use
`.assura/config.yml` with the fields below.

## Top-Level Fields

```yaml
patterns: {}
structure: {}
exclude: []
ls: null
```

- `patterns`: optional top-level file patterns.
- `structure`: required directory-shape rules.
- `exclude`: paths excluded from validation.
- `ls`: optional LS-Lint compatibility section used by conversion and tests.

## Directory Node

```yaml
structure:
  ./:
    required: true
    inherit: true
    files: {}
    directories: {}
    markdown: {}
    exists: {}
    children: {}
```

- `required`: whether the configured directory itself must exist.
- `inherit`: whether child scopes inherit parent rules.
- `files`: direct file rules.
- `directories`: direct child directory rules.
- `markdown`: markdown-specific checks.
- `exists`: required files or directories.
- `children`: nested directory scopes.

## File Bundle

```yaml
files:
  naming: kebab-case
  naming_patterns:
    "*.rs": snake_case
  allowed_names:
    - README.md
  allowed_patterns:
    - "*.md"
  forbidden_patterns:
    - "*.tmp"
  allow_extra: false
  exists:
    "README.md": "1"
```

## Directory Bundle

```yaml
directories:
  naming: kebab-case
  allowed_names:
    - src
    - tests
  allowed_patterns:
    - "package-*"
  forbidden_patterns:
    - "tmp-*"
  allow_extra: false
  exists:
    "package-*": "1-4"
```

## Validation Command

```bash
assura check --format text
```

Use `json` or `yaml` for machine-readable reports.
