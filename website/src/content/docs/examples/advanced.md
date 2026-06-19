---
title: Advanced Patterns
description: Supported advanced configuration patterns for Assura v0.1
---

These examples use the current `.assura/config.yml` structure-first
configuration. They avoid undocumented Rust APIs and plugin surfaces.

## Closed Project Shape

Use `extra: false` when a directory should contain only known direct children.

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    Cargo.toml: exists:1
    src/: exists:1
    tests/: exists:0-1
    docs/: exists:0-1
exclude:
  - "target/**"
```

## Directory-Specific Naming

Apply direct file rules where the files live.

```yaml
structure:
  ./:
    README.md: exists:1
    src/: exists:1
    docs/: exists:0-1
  src/:
    .rs: snake_case
  docs/:
    .md: kebab-case
```

Assura supports explicit child scopes and LS-Lint-compatible glob or brace
directory scopes such as `packages/*`, `**`, and `{src,tests}`.

## Direct-Child Existence Counts

Existence counts apply to direct children of the configured directory.

```yaml
structure:
  ./:
    README.md: exists:1
    "*.tmp": exists:0
    package-*/: exists:1-5
```

Supported count forms include `exists:1`, `exists:0`, and inclusive ranges like
`exists:1-5`.

## Reusable Package Rules

Use `rules:` and `use:` when the same policy repeats across package folders.

```yaml
rules:
  "@package-standard":
    README.md: exists:1
    package.json: exists:1
    src/: exists:1

structure:
  packages/:
    "{package}/":
      use: "@package-standard"
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
```

This keeps the package contract in one reusable fragment while each package and
documentation provider still appears where it lives in the tree.

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
