---
id: goal-assura-rule-command-surface-documentation
type: goal
title: Assura command surface documentation rule
status: completed
created: 2026-06-08
owners:
  - assura-maintainers
---

# Assura Command Surface Documentation Rule

## Objective

Create a configurable rule that detects documented Assura commands, flags, and
formats that are not present in the live CLI surface or support matrix.

## Detector Hypothesis

Parse command examples from configured docs paths, normalize `assura` and
`cargo run -- ... assura` invocations, compare flags/formats against generated
CLI help or a checked command contract, and allow explicit unsupported/future
classifications.

## Required Examples

- Passing: `assura check --format agent --agent codex .`
- Failing: `assura check --format codex-hook .`
- Failing: `assura check --maturity .`

## Tests

Add passing/failing fixtures, CLI integration coverage, and a self-check config
example for this repo.

## Implementation

- Added `command_surface_docs` under `extensions.custom_constraints`.
- Added `.assura/command-surface.yml` as a checked command contract.
- Dogfooded the rule against README, website docs, active docs, goal docs,
  analysis docs, AGENTS.md, and project skills through `.assura/config.yml`.
- The detector normalizes direct `assura ...` examples and
  `cargo run -- ...` examples, validates command families, flags, and configured
  flag values, and ignores lines explicitly labeled as failing, rejected,
  unsupported, historical, or future.
- Command-surface constraints are evaluated in one filesystem walk with a
  per-contract cache, so multiple source globs do not repeatedly walk the repo
  or reload the same contract.
- The contract supports nested command prefixes and simple flag requirements,
  such as `--agent` requiring `--format agent`.

## Validation Evidence

- Passing fixture: `assura check --format agent --agent codex .`.
- Unsupported failing fixtures: `assura check --format codex-hook .` and
  unsupported `assura check --maturity .`.
- Unsupported cargo-run fixture: `cargo run --quiet -- check --format codex-hook .`.
- Contract-safety fixture: duplicate/colliding aliases are rejected.
- Focused test: `cargo test --test custom_constraints_tests --quiet`.
- Self-check: `cargo run --quiet -- check --format json .`.
