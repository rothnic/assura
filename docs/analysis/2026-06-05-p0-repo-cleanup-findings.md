---
title: P0 Repo Cleanup Findings
date: 2026-06-05
status: active
---

# P0 Repo Cleanup Findings

## Current Cleanup

The active stale command-surface finding was in
`.agents/skills/custom/assura-validation/SKILL.md`. It documented unsupported
`assura check` flags such as `--maturity`, `--constraint`, `--pattern`,
`--convention`, `--require-frontmatter`, `--strict`, and `--parallel`.

The skill now uses the supported structure-first surface:

- `assura check`
- `assura check <path>`
- `assura check --format json .`
- `assura check --format agent . --warn`
- `assura check --format agent --agent codex . --warn`
- `assura status --format json`
- `assura migrate .ls-lint.yml --output .assura/config.yml`

Focused searches across active docs, website docs, skills, Trellis specs,
source, and tests found no remaining active examples for those unsupported
flags after the skill update. Archived docs and proposals were excluded from
the blocking scan because they may intentionally preserve historical or
future-design command examples.

## Deterministic Follow-Up Rules

Assura should eventually be able to detect this class of drift directly:

1. Extract shell snippets and inline command examples from active docs and
   skills.
2. Parse examples beginning with `assura` or `cargo run -- ...`.
3. Compare command names, formats, and flags against a configured support
   matrix derived from CLI help or a checked-in command contract.
4. Allow historical, archived, proposal, or explicit negative-example scopes
   through configuration.

This rule would generalize beyond Assura itself and could be configured for
other repositories with command-line documentation.

## Local Help Caveat

The primary `assura` launcher delegates non-check commands to a sibling
`assura-full` binary when one exists next to it. In local development,
`target/debug/assura-full` can be stale after branch switches. For command
surface audits, prefer:

```bash
cargo run --quiet --bin assura-full -- --help
cargo run --quiet --bin assura-full -- check --help
cargo run --quiet --bin assura -- check --help
```

Use `cargo build --bins` or remove stale `target/debug/assura-full` before
treating `cargo run --bin assura -- --help` as authoritative.
