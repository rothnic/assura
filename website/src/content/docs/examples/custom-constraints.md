---
title: Custom Constraints
description: Current options and roadmap for project-specific validation
template: doc
sidebar:
  order: 2
---

Assura v0.1 exposes a constrained experimental custom-constraint surface through
`.assura/config.yml`. Custom constraints still run through `assura check`; there
is no separate plugin command, remote loader, marketplace, or per-agent
entrypoint.

> **Experimental extension surface**
>
> The current surface is a first-party registry. A config selects supported
> constraint types by name, and Assura executes them inside the normal structure
> checker after configured exclusions are applied. Rust traits, TypeScript
> plugins, shell hooks, and network-loaded plugins are not public APIs.

## Supported Customization Today

Use `.assura/config.yml` to express project shape first:

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    Cargo.toml: exists:1
    src/: exists:1
    tests/: exists:0-1
  src/:
    .rs: snake_case
  tests/:
    required: false
    .rs: snake_case
exclude:
  - "target/**"
  - "node_modules/**"
```

Supported rule families include naming conventions, direct-child existence
counts, extension rules, directory naming, markdown rules, and exclusions.

## Relationship Notation First

For common repository relationships, prefer structure notation. It keeps the
source and target artifacts in the project tree and avoids a separate
constraint block.

```yaml
structure:
  src/:
    "{module}.rs": {}
  tests/:
    "{module}_test.rs": exists:1
```

Use `needs:` and `provides:` when several provider locations can satisfy the
same relationship.

## First-Party Custom Constraints

`extensions.custom_constraints` remains an experimental first-party execution
surface for specialized checks that cannot yet be expressed by structure
notation. It is not the preferred authoring surface for ordinary source/test or
package/doc relationships.

```yaml
extensions:
  custom_constraints:
    - id: command_surface_docs
      type: command_surface_docs
      source: "docs/*.md"
      target: ".assura/command-surface.yml"
      severity: high

structure:
  ./:
    extra: true
exclude:
  - "target/**"
```

If a docs file includes a command example outside the configured command
surface, `assura check` emits a normal structure violation:

```json
{
  "path": "docs/usage.md",
  "rule": "custom:command_surface_docs",
  "severity": "high"
}
```

Custom constraints use the same report shape as built-in rules and are sorted
with the rest of the check output.

## Pairing With Other Tools

For language semantics or arbitrary scripts outside Assura's current structure
scope, run tools side by side:

```bash
assura check --format text .
cargo clippy --all-targets --all-features -- -D warnings
pnpm lint
```

This keeps Assura responsible for repository shape while language-specific
tools handle language semantics.

## Boundary

Choose a built-in rule when the validation is generally useful across projects.
Choose a custom constraint when the rule is project-specific, deterministic, and
reviewed in the repository. Keep external linters as separate CI steps when the
check needs language analysis, network access, process execution, or untrusted
third-party code.
