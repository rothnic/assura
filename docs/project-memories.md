---
title: Assura Project Memories
status: active
---

# Assura Project Memories

This file preserves compact project context for agents. Current release claims
must defer to [`docs/release-notes.md`](./release-notes.md),
[`docs/support-policy.md`](./support-policy.md), and
[`docs/compatibility-and-surface.md`](./compatibility-and-surface.md).

## Current Product Baseline

Assura is a structure-first repository validation CLI written in Rust. The
supported v0.1 release candidate centers on:

- `assura check` for repository structure validation;
- `assura init` for starter `.assura/config.yml` creation;
- `assura status --format json` for project/config/rule summaries;
- `assura migrate` for supported LS-Lint 2.3 migration paths;
- `assura check --format agent` for stable agent feedback output;
- `assura check --format agent --agent codex` for optional Codex delivery; and
- installable GitHub release archives for Linux, macOS, and Windows.

Do not describe dependency graph validation, hosted dashboards, IDE plugins,
plugin marketplaces, package feedback CLIs, per-agent entrypoints, or
per-agent `--format` values as supported release surfaces.

## Key Dependencies

- `serde` and `serde_yaml` for configuration parsing.
- `clap` for CLI argument parsing.
- `regex` and `glob` for pattern matching.
- `walkdir` and related traversal code for repository validation.
- `petgraph`, `tokio`, and `notify` remain in the dependency set for planned
  or experimental work, but they are not release claims by themselves.

## Backwards Compatibility Policy

No internal backwards compatibility is guaranteed before 1.0.

- Configuration formats, output fields, and experimental extension fields may
  change before 1.0.
- Breaking changes must be called out in release notes.
- Supported LS-Lint migration behavior should be backed by tests or checked
  analysis evidence.
- Once 1.0 is released, standard semantic versioning should apply.

## Current Roadmap State

- Iteration 01 / Phase 01 is completed as a bounded roadmap iteration.
- Goal 08, Release Readiness And Ecosystem, completed in PR #25.
- Iteration 02 is planned in
  [`docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md`](./goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md).
- Completing Iteration 01 does not complete the full product roadmap.

## Coding Standards

- Rust edition: 2021.
- Formatting: `cargo fmt`.
- Linting: `cargo clippy --all-targets --all-features -- -D warnings`.
- Validation logic should have focused unit or integration coverage.
- Public release claims should have local or CI evidence linked from the PR.

## Common Commands

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
node --run verify:fast
node --run verify:evidence
node --run verify:docs
node --run verify:release-smoke
```
