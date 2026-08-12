---
title: Advanced Patterns
description: Supported advanced configuration patterns for Assura v0.1
---

These examples use the current `.assura/config.yml` structure-first
configuration. They avoid undocumented Rust APIs and plugin surfaces.

## Closed Project Shape

Compose a closed direct-content rule from the existing `exists:0` directive.

```yaml
rules:
  closed-entry:
    exists: 0
  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

structure:
  ./: $closed
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
  README.md: exists:1
  src/:
    ./: exists:1
    .rs: snake_case
  docs/:
    ./: exists:0-1
    .md: kebab-case
```

Assura supports explicit child scopes and LS-Lint-compatible glob or brace
directory scopes such as `packages/*`, `**`, and `{src,tests}`.

## Direct-Child Existence Counts

Existence counts apply to direct children of the configured directory.

```yaml
structure:
  README.md: exists:1
  ./*.tmp: exists:0
  ./package-*/: exists:1-5
```

Supported count forms include `exists:1`, `exists:0`, and inclusive ranges like
`exists:1-5`.

## Reusable Package Rules

Use a scalar rule reference when the same file attributes repeat, and `use:`
when a whole tree fragment repeats across package folders.

```yaml
rules:
  source-file:
    naming: kebab-case
    max_lines: 500

  package-standard:
    README.md: exists:1
    package.json: exists:1
    src/:
      ./: exists:1
      .{ts,tsx}: $source-file

structure:
  packages/:
    ./*/: $package-standard
  docs/packages/:
    ./: exists:0-1
    "{package}.md":
      provides: doc
```

This keeps simple file directives on one line and rebases the package contract
for every direct package. Expand a file directive in place when one pattern
needs a local override. The [configuration reference](/reference/configuration/#concise-and-expanded-equivalents)
shows both equivalent forms and glob scope controls.

Use explicit file globs when depth matters instead of cascading extension
shorthand:

```yaml config-fragment
structure:
  ./*.ts: $source-file # direct root files
  ./**/*.tsx: $source-file # root and descendants
  packages/*/src/:
    ./*.test.ts: $source-file # direct files in each matched src/
```

Run `assura explain path/to/file.ts` to see the matching hierarchy scopes and
the normalized file pattern that supplies each effective attribute.

## Generated Output Exclusions

Keep generated or dependency-heavy paths outside validation:

```yaml config-fragment
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

For continuous local feedback, run `assura watch --format json`. The event
stream reports whether each result covers the complete project or one affected
path, plus prepared-plan, reload, and fallback state. Public plugin APIs remain
roadmap work.
