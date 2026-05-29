---
title: Advanced Patterns
description: Supported advanced configuration patterns for Assura v0.1
---

These examples use the current `.assura/config.yml` structure-first
configuration. They avoid undocumented Rust APIs and plugin surfaces.

## Closed Project Shape

Use `allowed_names` with `allow_extra: false` when a directory should contain
only known direct children.

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
        - docs
      allow_extra: false
exclude:
  - "target/**"
```

## Directory-Specific Naming

Use nested `children` to apply different rules to explicit subdirectories.

```yaml
structure:
  ./:
    files:
      naming: kebab-case
    children:
      src:
        files:
          extensions:
            rs: snake_case
      docs:
        files:
          extensions:
            md: kebab-case
```

Assura supports explicit child scopes and LS-Lint-compatible glob or brace
directory scopes such as `packages/*`, `**`, and `{src,tests}`.

## Direct-Child Existence Counts

Existence counts apply to direct children of the configured directory.

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

Supported count forms include `1`, `0`, and inclusive ranges like `1-5`.

## Generated Output Exclusions

Keep generated or dependency-heavy paths outside validation:

```yaml
exclude:
  - "target/**"
  - "node_modules/**"
  - "dist/**"
  - "coverage/**"
  - "**/*.generated.*"
```

## JSON Reports

Use JSON for custom CI summaries:

```bash
assura check --format json . > assura-report.json
```

The report contains `success`, `project_root`, `config_path`, `checked_path`,
`files_checked`, `dirs_checked`, and `violations`.

> **Future work**
>
> Long-running watch mode, plugin APIs, and agent feedback are planned separately
> from the v0.1 onboarding release.
