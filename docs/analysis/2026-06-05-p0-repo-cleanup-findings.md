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

The same skill also advertised an `assura/assura-action@v1` GitHub Action that
is not a released or checked integration surface in this repository. The CI
example now installs the supported release script and runs
`assura check --format json .` directly. `node --run verify:evidence` scans
active skill docs for that unreleased action reference so the stale example
cannot be reintroduced silently.

The cleanup also tightened `.assura/config.yml` for the current `src/cli`
module topology. `src/cli/check` and `src/cli/performance_report` are now the
only allowed `src/cli` subdirectories, and their Rust files inherit the same
snake-case plus 500-line limit as the top-level CLI module files. The repo
self-check passes with that stricter policy.

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

## Workflow Lessons

This cleanup thread exposed two workflow gaps:

1. New work began while prior uncommitted work was still present. Future
   sessions must run a clean-start gate before task routing: commit obvious
   prior/current work, park it on a branch, or move to a fresh worktree/branch.
2. A listed GitHub plugin skill path was stale because the plugin cache hash
   had changed. The GitHub workflow itself was usable; the fallback should be
   to locate the current plugin skill path or use authenticated `gh` directly.

The Trellis start/continue/finish guidance now records the clean-start gate and
requires explicit multi-choice options when dirty-path ownership is unclear.
