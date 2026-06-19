---
id: goal-assura-rule-public-surface-support-matrix
type: goal
title: Assura public surface support matrix rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - docs/support-policy.md
  - docs/compatibility-and-surface.md
  - .assura/command-surface.yml
  - src/lib.rs
---

# Assura Public Surface Support Matrix Rule

## Objective

Create a configurable support-matrix rule that compares exported APIs,
documented commands, experimental surfaces, and unsupported claims.

## Revalidation Result

`valid`.

The command-surface documentation rule is already completed, and the
release-contract first slice now covers one class of release drift. The next
P0 detector in the current target-state analysis is broader support
classification: every public command, documented claim, manifest-facing surface,
and public Rust export should be classified as supported, experimental,
internal, roadmap, or unsupported.

This goal should not replace the command-surface docs rule. It should join that
checked command inventory with support-policy rows and public export metadata
so future module-topology, manifest-semantics, and test-relationship rules have
a trustworthy support-status source.

## User Certainty Bar

A maintainer should be able to add or expose a command/API/doc claim and get a
deterministic `assura check` finding when that surface is not classified by the
configured support matrix.

## Current Gap

- `.assura/command-surface.yml` classifies CLI command syntax, but support
  status is still spread across support docs, compatibility docs, source
  comments, and release notes.
- `src/lib.rs` exports experimental/internal modules with comments, but there
  is no reusable rule that checks those exports against a support matrix.
- Manifest metadata and docs claims are adjacent surfaces. The first slice
  should shape the matrix so manifest-semantics and docs-lifecycle follow-ups
  can reuse it instead of inventing separate status vocabularies.

## Detector Hypothesis

Parse configured support-policy tables and source exports, then require every
public command/API family to be classified as supported, experimental,
internal, roadmap, or unsupported.

## First Slice Scope

- Add explicit config notation for one or more support matrices.
- Accept configured support rows with a stable status vocabulary:
  `supported`, `experimental`, `internal`, `roadmap`, and `unsupported`.
- Check at least two inventory sources:
  - command families from an existing command-surface contract; and
  - public Rust exports from configured source files.
- Report unclassified public surfaces with file, surface, and matrix context.
- Add fixtures independent of Assura's own docs and a self-check example that
  dogfoods the rule on the Assura repository.

## Non-Goals

- No semver stability guarantee for pre-1.0 public Rust exports.
- No full Rust parser in the first slice unless current source scanning proves
  inadequate; prefer a bounded export scanner for configured files.
- No broad stale-doc natural language classifier.
- No replacement for the command-surface docs rule.
- No manifest metadata enforcement in this slice; keep that for the manifest
  semantics goal.

## Definition Of Done

- Support-matrix notation is documented before implementation.
- Passing fixture covers a supported command and an internal/experimental Rust
  export.
- Failing fixtures cover an unclassified command family and an unclassified
  public Rust export.
- `assura check --format json` reports actionable support-matrix violations.
- Assura self-check dogfoods the rule without weakening existing
  command-surface docs checks.
- Independent review confirms the rule provides reusable support
  classification rather than hard-coding Assura-specific surface names.

## Required Examples

- Passing: `assura check --format agent` classified supported.
- Passing: Rust `intelligence` exports classified unstable internal.
- Failing: dependency graph validation documented as supported without a
  support-policy row.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R0: Confirm the rule does not duplicate or weaken command-surface docs
  validation.
- R1: Review support status vocabulary and matrix notation for explicitness.
- R2: Review Rust export scanning for bounded behavior and low false positives.
- R3: Review fixtures for reusable examples outside this repository.
- R4: Confirm the rule creates a clean input for manifest-semantics,
  module-topology, and test-relationship follow-ups.

## Reviewer Blocking Criteria

Block the PR if the support status vocabulary is implicit, if the rule only
hard-codes Assura surfaces, if public Rust exports can silently bypass the
matrix, if command-surface coverage regresses, or if diagnostics do not point to
the unclassified surface and owning matrix.

## Progress Log

- 2026-06-19: Revalidated after Release Contract Rules first slice merged in PR
  #59 and archived in PR #60. Result: valid. The first slice should join the
  existing command-surface contract with explicit support rows and configured
  Rust export scanning; manifest metadata and stale-doc prose remain follow-up
  owners.
- 2026-06-19: Implemented `extensions.support_matrices` with stable status
  validation, command-surface contract inventory, bounded top-level Rust export
  scanning, `support_matrix:<id>` diagnostics, compiled artifact portability,
  Assura self-dogfood rows, and independent fixtures for classified and
  unclassified command/Rust surfaces. Local validation passed:
  `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo fmt --all -- --check`, `cargo xtask target-state`,
  `cargo run --quiet -- check --format json .`, `cargo xtask evidence`,
  `cargo xtask docs`, and `git diff --check`.
- 2026-06-19: Review agent flagged grouped Rust re-exports and
  `pub use crate::...` paths as scanner gaps. Fixed the export scanner to
  classify anchored and grouped top-level `pub use` forms without scanning
  nested module bodies, then added regression coverage.
