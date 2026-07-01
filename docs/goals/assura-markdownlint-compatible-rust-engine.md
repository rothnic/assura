---
id: goal-assura-markdownlint-compatible-rust-engine
type: goal
title: Assura markdownlint-compatible Rust engine
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ../../.trellis/tasks/archive/2026-07/07-01-post-beta-followup-roadmap-goals/research/markdown-linter-options.md
---

# Assura Markdownlint-Compatible Rust Engine

## Objective

Adopt or integrate the most performant practical Rust Markdown linter/fixer
that is consistent with markdownlint, while preserving Assura's severity,
suppression, safe-fix, reference-graph, and daemon contracts.

## Current Gap

`v0.2.0` has Rust-native Markdown checks for headings, links, suppressions,
safe fixes, and common lint classes. It does not yet provide broad
markdownlint-compatible rule/config coverage or prove that its lint/fix engine
is the fastest available Rust path.

Markdown linting is not the top of Assura's validation hierarchy. Assura should
first report structure, repository-shape, and coarse file-level policy issues,
then run Markdown-specific syntax, heading, link, suppression, and safe-fix
checks for files that belong in configured Markdown scopes. Typed frontmatter
fields, IDs, relations, and path scopes remain owned by content models.

## Candidate Direction

Default to evaluating `rumdl` first. Current public research says `rumdl` is a
Rust-native Markdown linter/formatter with markdownlint-compatible rules,
autofix/formatting, config conversion, a library crate, editor support, and
benchmark claims against `markdownlint-cli` and `markdownlint-cli2`. Compare
against `mado`, `markdownlint-rs`/`mdlint`, current Assura Markdown checks, and
Node `markdownlint-cli2` before adopting.

## Scope

- Build a local markdownlint compatibility matrix for rule IDs, config keys,
  suppressions, severities, fixability, and expected diagnostics.
- Preserve staged validation: structure and coarse file-level checks should
  gate or precede deeper Markdown linting in user-facing output and docs.
- Evaluate `rumdl` as an embedded library or subprocess adapter, including
  MSRV, license, binary size, dependency, API stability, and performance.
- Identify common Markdown formatting drift found while dogfooding Assura's
  own docs. If the selected markdownlint-compatible engine cannot safely fix a
  repeatable class of drift, implement a narrowly scoped Assura-owned fixer
  utility with explicit tests instead of leaving agents to repair it manually.
- Add benchmark fixtures for small docs, large docs, many-file repos, generated
  docs, frontmatter-heavy docs, and link-heavy docs.
- Integrate chosen lint/fix results into Assura finding IDs, severity overrides,
  reasoned suppressions, JSON/agent output, and `assura fix markdown`.
- Preserve Assura-specific link/reference graph checks where markdownlint-style
  tools do not cover repository semantics.
- Produce a fallback decision record if `rumdl` is not adopted.

## Non-Goals

- No from-scratch rewrite of commodity markdownlint rules before measuring
  existing Rust candidates.
- No Node runtime dependency for the supported Assura path.
- No unsafe automatic fixes; apply mode must remain bounded, auditable, and
  idempotent.

## Definition Of Done

- A measured Rust lint/fix engine is selected and integrated or explicitly
  rejected with evidence.
- Markdownlint-compatible rule/config fixtures pass for the accepted surface.
- User-facing docs and diagnostics preserve the hierarchy from structure and
  coarse file policy to deeper Markdown checks.
- Fixes are deterministic, idempotent, and preserve frontmatter and line endings
  where required.
- Common Assura-owned fixer utilities, if added, have dedicated valid/invalid
  fixtures and are documented as opinionated defaults that users can configure
  or disable.
- Performance evidence shows the chosen Rust path is no slower than current
  Assura Markdown checks for accepted fixtures and materially faster than
  `markdownlint-cli2` on representative repos.
- Independent review confirms Assura-specific reference checks remain intact.

## Validation Commands

```bash
cargo fmt --check
cargo test --test markdown_link_reference_tests --quiet
cargo test --test markdown_suppression_severity_tests --quiet
cargo test --test markdown_required_section_fix_tests --quiet
cargo test --test markdown_lint_fix_tests --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
```

## Reviewer Blocking Criteria

Block if the goal skips `rumdl` evaluation, adopts a linter without local
markdownlint compatibility fixtures, regresses Assura reference checks, applies
unsafe fixes, lacks benchmark evidence against current Assura, selected Rust
candidates, and `markdownlint-cli2`, or leaves repeatedly observed self-dogfood
Markdown drift without either a selected-engine fix or a tracked Assura-owned
fixer test. Also block if docs or output imply Markdown linting sits above
structure validation.
