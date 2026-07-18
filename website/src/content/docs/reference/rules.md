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
  ./**/:
    .{ts,tsx}: $source-file
```

Run `assura explain path/to/file.ts` to see the scopes that apply and the
winning naming, line, and size thresholds. JSON output also includes
`source_rules`, with each authored rule name, rebased selector, target kind,
and whether it was checked or skipped.

## Structure Directives

| Directive | Purpose |
| --- | --- |
| `naming` | Enforce a built-in naming convention or `regex:<pattern>`. |
| `max_lines` | Set a language-agnostic file line threshold. |
| `max_size` | Set a language-agnostic file size threshold. |
| `exists` | Require, allow, or forbid direct files and directories by count. |
| `limit_children` | Set a combined direct file and directory threshold. |
| `severity` | Make the composed constraint advisory (`low`) or blocking. |
| `message` | Append bounded project-owned repair guidance. |
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
  AGENTS.md: exists:1
  docs/: exists:0-1
  packages/:
    ./*/:
      ./: kebab-case
      package.json: exists:1
      src/: exists:1
```

## Scalar And Expanded Forms

Use a scalar when one reusable rule or naming convention is enough:

```yaml config-fragment
structure:
  ./**/:
    .rs: snake_case
    .ts: $source-file
  packages/:
    ./*/: $package-standard
```

Use ` | ` for compatible scalar composition. Use a mapping for a rule with
several attributes:

```yaml
rules:
  guidance:
    max_lines: 160
    severity: low
    message: See docs/agent-guidance.md.

structure:
  AGENTS.md: exists:1 | $guidance
  ./**/:
    .md: kebab-case | exact:AGENTS | exact:README
```

Pipelines split only on a top-level ` | ` surrounded by spaces, so regex
alternation remains intact. They apply left to right; a later value for the
same attribute is an explicit local override. A rule mapping is either a node
constraint or a child tree; mixing both is a configuration error.

## Agent Policy Recipes

Generate editable agent policy into the repository:

```bash
assura init --recipe agentic-core --recipe structure-health
```

The generated rules are normal project YAML and remain available with the
recipe catalog removed. Use `extensions.agent_guidance` for checks inside `AGENTS.md`
and project-local `SKILL.md` files, including required sections, routing,
frontmatter, and line limits.

When adding a recipe to an existing project, use:

```bash
assura config add-recipe agentic-core . --dry-run
assura config add-recipe agentic-core .
```

`assura doctor --format json` reports a low-severity `config_rule_reuse`
diagnostic when a trivial named node rule is used once or not at all. Tree
contracts and multi-directive rules remain valid even when they have one use.

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
