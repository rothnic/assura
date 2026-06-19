---
title: Configuration Reference
description: Supported configuration fields for Assura
template: doc
sidebar:
  order: 1
---

Assura discovers configuration from `.assura/config.yml` by default. The stable
validation command is:

```bash
assura check
```

Use structured output when you need reproducible evidence:

```bash
assura check --format json .
```

## Discovery

Recommended location:

```text
.assura/config.yml
```

The CLI can also receive a config path with the global `--config` option.

## Top-Level Fields

```yaml
structure: {}
exclude: []
rules: {}
ls: null
```

| Field | Behavior |
| --- | --- |
| `structure` | Directory-shaped policy tree used by `assura check`. |
| `exclude` | Glob-like paths excluded from validation and direct-child counts. |
| `rules` | Optional reusable authoring fragments referenced from `structure` with `use:`. Rules compile into the normal structure model before validation. |
| `ls` | Compatibility input used by migration and tests, not the public `assura check` policy surface. Prefer `assura migrate` so LS-Lint rules are converted into `structure`. |
| `patterns` | Library resolver field from the older config model. It is accepted by the config type but is not the public `assura check` policy surface. Use `structure` instead. |

Assura excludes its own `.assura/**` tool-state directory automatically during
checks. Do not add it to ordinary project-shape excludes unless a future command
explicitly asks for that directory to be validated.

## Concise Structure Notation

Simple policy should stay in the tree:

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    AGENTS.md: exists:1
    src/: exists:1
    .rs: snake_case
```

Concise keys expand to the same internal model documented below:

| Notation | Behavior |
| --- | --- |
| `extra: false` | Rejects unrecognized direct files and directories in this scope. |
| `README.md: exists:1` | Requires exactly one direct file named `README.md`. |
| `src/: exists:1` | Requires exactly one direct child directory named `src`. |
| `.rs: snake_case` | Applies `snake_case` naming to direct `*.rs` files. |

Use a mapping under the same path key when the directive needs more detail:

```yaml
structure:
  docs/:
    "{topic}.md":
      exists: 1
      markdown:
        outline:
          - Overview
          - ?? Prerequisites
          - Quick Start:
              - Installation
              - ?? Configuration
          - Why Assura?
          - title: "?? Debug Mode"
            optional: false
```

Captures use single braces such as `{topic}`. Removed alpha capture forms such
as `${name}` and `{{name}}` are not supported in hand-authored structure
notation.

## First-Time Notation Matrix

Start with LS-Lint-equivalent policies, then add Assura-native structure when
the project needs relationships or reusable contracts.

| Use case | Notation |
| --- | --- |
| Direct file naming | `.rs: snake_case` |
| Direct file count | `README.md: exists:1` |
| Optional singleton | `"*.lock": exists:0-1` |
| Forbidden direct children | `draft-*: exists:0` |
| Closed-world scope | `extra: false` |
| Generated output ignore | `exclude: ["target/**", "node_modules/**"]` |
| Captured source/test pair | `"{component}.tsx"` and `"{component}.test.tsx": exists:1` |
| Package documentation need | `needs: doc` with `provides: doc` |
| Reusable package policy | `rules:` plus `use: "@package-standard"` |
| Markdown outline | `markdown.outline` with nested heading lists |

Use the detailed fields below when a rule needs extra attributes or when you
are reading generated migration output.

Markdown outline notation validates ordered heading structure without separate
heading-depth fields. It is for Assura-specific document structure checks, not
a replacement for generic Markdown linting or link validation.

## Directory Nodes

Each key under `structure` is a directory scope. Use `./` for the project root.

```yaml
structure:
  ./:
    required: true
    inherit: true
    files: {}
    directories: {}
    self_directory: {}
    markdown: {}
    exists: {}
    children: {}
```

| Field | Behavior |
| --- | --- |
| `required` | Whether this configured directory itself must exist. Defaults to `true`. |
| `inherit` | Whether child scopes inherit parent file, directory, and markdown rules. Defaults to `true`. |
| `files` | Rules for direct child files in this directory scope. |
| `directories` | Rules for direct child directories in this directory scope. |
| `self_directory` | Rules for the configured directory itself. This is primarily emitted by LS-Lint `.dir` migration. Hand-written policies usually use `directories` for direct children. |
| `markdown` | Markdown checks for direct child `.md` files in this directory scope. |
| `exists` | Legacy required file/directory lists. Prefer `files.required`, `directories.required`, or direct count rules for new config. |
| `children` | Nested directory scopes. |

## File Rules

```yaml
files:
  naming: kebab-case
  naming_patterns:
    "*.rs": snake_case
  max_lines: 500
  max_size: 100KB
  require_docs: true
  extensions:
    - rs
    - md
  severity: high
  required:
    - README.md
  allowed_names:
    - README.md
    - Cargo.toml
  allowed_patterns:
    - "*.lock"
  forbidden_patterns:
    - "draft-*"
  allow_extra: false
  exists:
    "README.md": "1"
    "*.tmp": "0"
```

| Field | Behavior |
| --- | --- |
| `naming` | Naming convention for files in the scope. Supports built-in case names and `regex:<pattern>`. |
| `naming_patterns` | Naming conventions keyed by direct file glob pattern. More specific matches win. |
| `max_lines` | Fails when a direct file has more lines than the configured limit. |
| `max_size` | Fails when a direct file exceeds a size such as `100KB` or `2MB`. |
| `require_docs` | For Rust files, requires `//!` or `///` rustdoc text. |
| `extensions` | Allows only the listed extensions when extension validation is configured. Multi-part extensions such as `tar.gz` are supported. |
| `severity` | Severity assigned to violations from this file bundle. |
| `required` | Exact direct files that must exist. |
| `allowed_names` | Exact direct file names allowed by a closed-world policy. |
| `allowed_patterns` | Direct file glob patterns allowed by a closed-world policy. |
| `forbidden_patterns` | Direct file glob patterns that are always rejected. Forbidden patterns override broad allowed patterns. |
| `allow_extra` | When `false`, rejects direct files not covered by exact names, allowed patterns, or allowed extensions. |
| `exists` | Direct child file count constraints keyed by glob or exact pattern. Values are `exists`, `0`, `1`, or ranges such as `1-4`. |

## Directory Rules

```yaml
directories:
  naming: kebab-case
  severity: critical
  required:
    - src
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

| Field | Behavior |
| --- | --- |
| `naming` | Naming convention for direct child directories. |
| `severity` | Severity assigned to violations from this directory bundle. |
| `required` | Exact direct child directories that must exist. |
| `allowed_names` | Exact direct child directory names allowed by a closed-world policy. |
| `allowed_patterns` | Direct child directory glob patterns allowed by a closed-world policy. |
| `forbidden_patterns` | Direct child directory glob patterns that are always rejected. |
| `allow_extra` | When `false`, rejects direct child directories not covered by `children`, exact names, or allowed patterns. |
| `exists` | Direct child directory count constraints keyed by glob or exact pattern. |

## Markdown Rules

```yaml
markdown:
  require_frontmatter: true
  required_fields:
    - title
  max_heading_depth: 3
  required_sections:
    - Summary
  outline:
    - Overview
    - ?? Prerequisites
    - Quick Start:
        - Installation
        - ?? Configuration
```

| Field | Behavior |
| --- | --- |
| `require_frontmatter` | Requires YAML frontmatter in direct child Markdown files. |
| `required_fields` | Requires fields inside YAML frontmatter. |
| `max_heading_depth` | Fails when a Markdown heading is deeper than the configured level. |
| `required_sections` | Requires headings with the configured text. |
| `outline` | Validates ordered nested headings without requiring users to maintain heading depth numbers. Use `?? ` for optional headings and object form such as `title: "?? Debug Mode"` when a required heading starts with literal question marks. |
| `check_links` | Accepted by the config type but not enforced by current `assura check`. |

## Relationships

Captured paths can express relationships without leaving the project tree. A
captured path without `exists` is optional; a captured path with `exists:1`
becomes required for each matching source with the same capture names.
Required captured children inside a captured directory stay ordinary structure
requirements for that directory.

```yaml
structure:
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
```

If `src/components/Button.tsx` exists, Assura requires
`src/components/Button.test.tsx`. If no component exists, no test file is
required.

Use `needs:` and `provides:` when a relationship can be satisfied by more than
one artifact:

```yaml
structure:
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
  docs/:
    required: false
    packages.md:
      sections:
        "{package}":
          provides: doc
```

For each package directory, either `docs/packages/<package>.md` or a heading
named `<package>` in `docs/packages.md` satisfies the `doc` need.

Entries that only declare `provides:` are providers, not producers. Missing
relationship reports name the producer, source pattern, declaring structure
entry, provider kind, and expanded counterpart or provider path. Duplicate
provider alternatives for the same need and capture set are rejected as
ambiguous during config loading.

## Closed-World Example

This policy rejects stray files and directories at the project root while
allowing generated output to stay outside the source contract.

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    Cargo.toml: exists:1
    "*.lock": exists:0-1
    src/: exists:1
    docs/: exists:1
    "package-*/": exists:0-20
    draft-*: exists:0
    tmp-*/: exists:0
exclude:
  - "target/**"
  - "generated/**"
```

Given `draft-plan.md`, `scratch.txt`, and `tmp-cache/`, JSON output includes
stable path, rule, severity, message, and corrective context fields:

```json
{
  "path": "draft-plan.md",
  "rule": "forbidden_file",
  "message": "File 'draft-plan.md' is forbidden by policy",
  "severity": "high",
  "corrective_context": "Remove or rename the file, or narrow files.forbidden_patterns if this file should be allowed."
}
```

## Direct Counts And LS-Lint Boundary

Direct count rules apply only to direct children of the configured directory.

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

LS-Lint extension rules such as `.md: exists:1-2` map to direct file counts and
are treated as LS-Lint parity. Exact direct filename rules such as
`README.md: exists:1` are an Assura compatibility extension when produced by
`assura migrate`; upstream LS-Lint 2.3 does not treat exact filenames as count
targets.

## Report Formats

```bash
assura check --format text
assura check --format json .
assura check --format yaml .
assura check --format agent .
assura check --format agent --agent codex . --warn
```

The JSON report contains `success`, `project_root`, `config_path`,
`checked_path`, `files_checked`, `dirs_checked`, and `violations`.
Each violation contains `path`, `rule`, `message`, `severity`, and
`corrective_context`.
