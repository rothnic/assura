---
title: Rules Reference
description: Reusable structure, content, and agent policy in Assura
---

Assura rules are mappings, not a list of plugin names. Define a reusable
fragment once under `rules`, then apply it from the project-shaped `structure`
tree. Definitions use plain names; references add `$` so they remain distinct
from ordinary scalar directives without requiring YAML quotes.

```yaml
rules:
  source-file:
    naming: kebab-case
    max_lines: 500

structure:
  ./:
    .ts: $source-file
    .tsx: $source-file
```

Run `assura explain path/to/file.ts` to see the scopes that apply and the
winning naming, line, and size thresholds.

## Structure Directives

| Directive | Purpose |
| --- | --- |
| `naming` | Enforce a built-in naming convention or `regex:<pattern>`. |
| `max_lines` | Set a language-agnostic file line threshold. |
| `max_size` | Set a language-agnostic file size threshold. |
| `exists` | Require, allow, or forbid direct files and directories by count. |
| `extra` | Close a scope to paths not represented by its policy tree. |
| `use` | Apply a reusable project rule. |
| `inherit` | Keep or reset parent policy in a more specific scope. |
| `needs` / `provides` | Connect captured paths to one or more valid providers. |
| `markdown` | Apply Markdown structure, links, and safe lint checks. |

An exact literal file or directory in the tree is required by default. Use
`exists:0-1` for an optional singleton and `exists:0` to forbid a direct path.
Patterns and captures are match-only unless they declare `exists`.
File captures use `exists:1` for per-source counterparts; directory captures
use `exists` ranges for direct-child counts.

```yaml
structure:
  ./:
    AGENTS.md: exists:1
    docs/: exists:0-1
    packages/:
      "{package}/":
        .dir: kebab-case
        package.json: exists:1
        src/: exists:1
```

## Scalar And Expanded Forms

Use a scalar when one reusable rule or naming convention is enough:

```yaml config-fragment
structure:
  ./:
    .rs: snake_case
    .ts: $source-file
    packages/:
      "{package}/": $package-standard
```

Use a mapping when a path needs local composition or an override:

```yaml
rules:
  project-standard:
    extra: false

structure:
  ./:
    use:
      - $agentic-project
      - $project-standard
    extra: true
    README.md: exists:0-1
```

Rules in `use` are applied in order. Local attributes are applied last, so the
root above reopens extra entries and adds an optional README after composing
both reusable rules.

## Built-In Agent Policy

`$agentic-project` provides the broad repository-level baseline used by agent
onboarding. It requires root agent guidance and composes the standard local
skill structure without guessing language or domain rules.

```yaml
rules:
  project-baseline:
    use: $agentic-project

structure:
  ./:
    use: $project-baseline
```

Use `extensions.agent_guidance` for deterministic checks inside `AGENTS.md`
and project-local `SKILL.md` files, including required sections, routing,
frontmatter, and line limits.

## Relationships

Captured paths can require a matching artifact:

```yaml
structure:
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
```

Use `needs` and `provides` when more than one artifact can satisfy a project
relationship. The [Configuration Reference](/reference/configuration/) covers
the full notation and first-party extension policies.

## Severity And Feedback

Structure violations default to a blocking policy finding. First-party
extension policies can set `low`, `medium`, `high`, or `critical` severity.
Low findings are advisory; medium and above are blocking. Every report format
preserves the rule ID, path, threshold context, severity, and corrective
guidance used by agents.
