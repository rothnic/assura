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
version: "2.0"
structure: {}
exclude: []
ls: null
```

| Field | Behavior |
| --- | --- |
| `structure` | Directory-shaped policy tree used by `assura check`. |
| `exclude` | Glob-like paths excluded from validation and direct-child counts. |
| `ls` | Compatibility input used by migration and tests, not the public `assura check` policy surface. Prefer `assura migrate` so LS-Lint rules are converted into `structure`. |
| `patterns` | Library resolver field from the older config model. It is accepted by the config type but is not the public `assura check` policy surface. Use `structure` instead. |

## Compact Structure Notation

The native authoring path is a tree under `structure:`. Path-like keys describe
direct files, direct directories, file-extension rules, and wildcard directory
scopes.

```yaml
version: "2.0"

structure:
  ./:
    README.md: exists:1
    AGENTS.md: exists:1
    packages/:
      "*/":
        extra: false
        package.json: exists:1
        src/:
          .ts: kebab-case

exclude:
  - "node_modules/**"
  - "dist/**"
  - "**/dist/**"
```

| Key shape | Behavior |
| --- | --- |
| `README.md` | Exact direct file in the current directory scope. |
| `src/` | Exact direct child directory. |
| `packages/` | Exact direct child directory with nested rules. |
| `*/` | Wildcard child directory scope under the current directory. |
| `packages/*/` | Wildcard directory scope when used directly under `structure`. |
| `.md` | Direct Markdown file rule in the current directory scope. |
| `.test.ts` | Direct multi-part extension rule in the current directory scope. |
| `.dir` | Rule for matched directories themselves. |

Wildcard directory scopes validate existing matching directories. They do not
require a literal `*` directory. Add `required: true` inside the wildcard scope
when at least one matching directory must exist:

```yaml
structure:
  ./:
    packages/:
      "*/":
        required: true
        package.json: exists:1
```

Because `packages/` is an exact directory key, that parent directory is required
by default. Because `*/` is a wildcard scope, it is optional unless
`required: true` is set.

`exists` is the compact count model:

| Shorthand | Behavior |
| --- | --- |
| `exists:1` | Exactly one matching direct child is required and allowed. |
| `exists:0-1` | The direct child is optional and allowed when present. |
| `exists:0` | Matching direct children are forbidden. |
| `exists:N-M` | Matching direct child count must be within the inclusive range. |

Use the expanded fields below for advanced cases such as severity overrides,
inheritance edge cases, closed-world allow/forbid combinations, markdown
frontmatter requirements, file size limits, and custom validation extensions.

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
```

| Field | Behavior |
| --- | --- |
| `require_frontmatter` | Requires YAML frontmatter in direct child Markdown files. |
| `required_fields` | Requires fields inside YAML frontmatter. |
| `max_heading_depth` | Fails when a Markdown heading is deeper than the configured level. |
| `required_sections` | Requires headings with the configured text. |
| `check_links` | Accepted by the config type but not enforced by current `assura check`. |

## Closed-World Example

This policy rejects stray files and directories at the project root while
allowing generated output to stay outside the source contract.

```yaml
structure:
  ./:
    files:
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
    directories:
      required:
        - src
      allowed_names:
        - src
        - docs
      allowed_patterns:
        - "package-*"
      forbidden_patterns:
        - "tmp-*"
      allow_extra: false
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
are treated as native LS-Lint parity. Exact direct filename rules such as
`README.md: exists:1` are an Assura compatibility extension when produced by
`assura migrate`; upstream LS-Lint 2.3 does not treat exact filenames as native
count targets.

## Report Formats

```bash
assura check --format text
assura check --format json .
assura check --format yaml .
assura check --format advice .
assura check --format status .
assura check --format agent .
assura check --format agent --agent codex . --warn
```

The JSON report contains `success`, `project_root`, `config_path`,
`checked_path`, `files_checked`, `dirs_checked`, and `violations`.
Each violation contains `path`, `rule`, `message`, `severity`, and
`corrective_context`.
