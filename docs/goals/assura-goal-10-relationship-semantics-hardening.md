---
id: goal-assura-roadmap-10-relationship-semantics-hardening
type: goal
title: Assura roadmap 10 relationship semantics hardening
status: completed
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/2026-06-15-notation-clean-start-roadmap.md
  - docs/analysis/2026-06-18-goal-10-relationship-semantics-review.md
  - tests/structure_config_notation_tests.rs
---

# Goal 10: Relationship Semantics Hardening

## Objective

Make capture-driven relationships predictable, well-diagnosed, and safe to
build on before Assura expands relationship-heavy policy coverage.

This is a two-week team chunk for config parsing, runtime validation,
diagnostics, fixtures, and reviewer-led edge-case analysis.

## Current Gap

This goal is not complete today because the canonical relationship syntax and
basic happy-path fixtures exist, but the edge cases that make users trust the
feature are not yet proven: ambiguous providers, overlapping provider kinds,
same-name captures in separate scopes, and diagnostics that point back to the
declaring structure entry.

## User Certainty Bar

When a relationship check fails, a user should know which artifact produced the
requirement, which counterpart or provider is missing, which config entry
declared the relationship, and what edit will make the project pass.

## Scope

- Add fixtures for ambiguous capture providers, overlapping provider kinds,
  missing counterparts, and same-name captures in separate scopes.
- Cover package documentation alternatives through dedicated
  `docs/packages/{package}.md` files and aggregate `docs/packages.md` sections.
- Update every affected public example, website example, generated example,
  fixture config, and test-case `.assura/config.yml` that teaches or exercises
  relationship notation.
- Improve diagnostics so relationship failures name the producer, missing
  counterpart or provider kind, and the structure entry that declared it.
- Verify counterpart and provider artifacts remain configured where they live in
  the tree.
- Preserve the already-merged canonical relationship notation; this goal closes
  edge cases and diagnostics, not the base syntax.

## Non-Goals

- No arbitrary shell validators.
- No dependency graph validation claim.
- No backwards compatibility for removed alpha notation.
- No broad performance optimization before the relationship behavior is
  correct and measurable.
- No notation compatibility shim for an old relationship form unless a
  support-policy exception and removal plan are explicit.

## Definition Of Done

- Good/base/bad fixtures cover the supported relationship semantics.
- Ambiguous or invalid relationships fail with actionable diagnostics.
- Existing relationship passing cases still pass.
- Relationship behavior is documented as Assura-native notation, not as
  LS-Lint parity.
- Reviewers can tell whether a failure is a missing producer, counterpart,
  provider, section, or declaration problem.
- A first-time user can fix every relationship fixture failure from the report
  text and docs without knowing the internal normalized `extensions` model.
- Relationship notation changes have checked performance evidence or a bounded
  inherent-cost record.
## Required Validation

```bash
cargo fmt --all -- --check
cargo test structure_notation --quiet
cargo test --test structure_config_notation_tests --quiet
cargo run --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm the goal does not duplicate the completed canonical notation PR.
- R1: Review relationship fixtures for false positives and false negatives.
- R2: Review diagnostics against first-time user comprehension.
- R3: Confirm provider alternatives work across file and section providers.
- R4: Confirm removed alpha notation remains rejected or undocumented.
- R5: Confirm public examples, generated examples, fixtures, and test-case
  configs no longer teach superseded relationship notation.
- R6: Confirm the PR links this goal and a relationship behavior review record.

## Reviewer Blocking Criteria

Block the PR if relationship errors do not identify the declaring structure
entry, if ambiguous providers silently pass, or if the implementation requires
targets to be declared away from the tree location where they live. Also block
if a relationship notation change skips performance evidence or keeps
backwards-compatibility support for superseded alpha notation.

## Progress Log

| Date | Entry | Evidence |
| --- | --- | --- |
| 2026-06-18 | Revalidated Goal 10 as still needed. Canonical relationship notation and happy-path tests existed, but live code still allowed provider-only artifacts to become producers, lacked same-name capture isolation across scopes, did not reject duplicate provider alternatives, and emitted generic relationship diagnostics without the declaring structure entry. | `docs/analysis/2026-06-18-goal-10-relationship-semantics-review.md`; `src/config/config/structure_notation/relationships.rs`; `src/cli/check/custom_constraints.rs`; `tests/structure_config_notation_tests.rs` |
| 2026-06-18 | Hardened relationship compiler and runtime behavior for local counterpart pairing, provider-only entries, duplicate provider ambiguity, overlapping file/section provider alternatives, missing counterparts, and actionable diagnostics. | `cargo test structure_notation --quiet`; `cargo test --test structure_config_notation_tests --quiet` |
| 2026-06-18 | Recorded bounded performance evidence for the relationship notation/runtime change. The report completed with 392 result rows; the implementation adds constant-size relationship metadata and preserves the existing relationship validation traversal shape. | `cargo run --quiet -- performance-report --output target/performance/current.json`; `docs/analysis/2026-06-18-goal-10-relationship-semantics-review.md` |
| 2026-06-18 | Independent review found no blockers and called out two residual test gaps; added compiler tests for provider-only captured entries not becoming counterpart producers and ambiguous cross-tree counterparts failing config loading. Split relationship helpers out of `custom_constraints.rs` to keep the repo line-count policy green. | Review agent Kant; `cargo test structure_notation --quiet`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo run --quiet -- check --format json .`; `git diff --check` |
| 2026-06-19 | Revalidated status against live PR and Trellis evidence. Goal 10 merged in PR #52 (`73690fad9299b10324146ccd37b75b73cdd1d0e7`) and is archived under `.trellis/tasks/archive/2026-06/06-18-goal-10-relationship-semantics`. | `gh pr view 52`; `.trellis/spec/assura/roadmap.md` |
