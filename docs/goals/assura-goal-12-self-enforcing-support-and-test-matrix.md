---
id: goal-assura-roadmap-12-self-enforcing-support-and-test-matrix
type: goal
title: Assura roadmap 12 self-enforcing support and test matrix
status: planned
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - docs/goals/assura-rule-test-relationship.md
  - docs/goals/assura-rule-cargo-manifest-semantics.md
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
---

# Goal 12: Self-Enforcing Support And Test Matrix

## Objective

Turn Assura's support policy, public command surface, manifest metadata, and
test coverage expectations into deterministic checks that prevent public-surface
drift.

This is a two-week team chunk for support-policy modeling, command-surface
classification, Cargo manifest checks, test relationship checks, docs, and
review evidence.

## Current Gap

This goal is not complete today because Assura has command-surface checks and
planned rule docs, but it does not yet have one deterministic matrix that joins
commands, public Rust exports, docs claims, manifest metadata, support-policy
rows, and required tests. That means a surface can still be partially supported
by convention instead of proof.

## User Certainty Bar

A user should never discover through trial and error that a documented command,
format, install path, or support promise is stale. The repo should fail before
that claim reaches a release or onboarding page.

## Scope

- Join CLI commands, docs claims, manifest metadata, public Rust exports, and
  support-policy rows into a support matrix.
- Implement or advance the planned support-matrix, manifest-semantics, and
  test-relationship rule goals where needed for the support matrix.
- Require supported surfaces to have appropriate docs and tests.
- Classify experimental/internal/roadmap surfaces without deleting contained
  evidence modules prematurely.
- Add fixtures and self-check configuration that prove the matrix catches
  stale or unsupported claims.
- Add a stale-goal validation check or documented workflow hook that points
  future agents to `assura-goal-validation` before executing old or
  separate-context goals.

## Non-Goals

- No broad module deletion before the support matrix distinguishes product code
  from experimental/internal evidence code.
- No 1.0 semantic-versioning commitment.
- No semver checker adoption before public API support policy is stricter.
- No license/source policy rollout unless the manifest rule requires a bounded
  placeholder.

## Definition Of Done

- Every supported public command family has an explicit support classification.
- Public docs and release notes cannot claim unsupported commands or formats as
  supported.
- Workspace manifest metadata follows the chosen internal/public policy.
- Supported surfaces map to required test families or an explicit exception.
- `cargo xtask target-state` or equivalent checked command reports matrix
  failures deterministically.
- Any public claim without support status, tests, or an accepted exception is a
  blocking finding.
- Future goal docs have a deterministic creation/revalidation path before they
  can be treated as current execution contracts.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo xtask target-state
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm existing planned rule docs are reused instead of duplicated.
- R1: Review the support classifications against release notes and support
  policy.
- R2: Review manifest metadata checks for internal versus public crates.
- R3: Review test-relationship checks for supported surfaces and accepted
  exceptions.
- R4: Confirm contained experimental modules are classified rather than
  accidentally promoted.
- R5: Confirm the PR links this goal and a matrix review artifact.
- R6: Confirm old or separate-context goal docs have a revalidation route
  through `assura-goal-validation`.

## Reviewer Blocking Criteria

Block the PR if a public surface can remain unclassified, if supported behavior
has no test or documented exception, or if the rule set forces broad deletion
instead of producing actionable classification evidence.

## Progress Log

- 2026-06-18: Revalidated against live repo state after Goal 11 merged in PR
  #53. Goal remains valid: `cargo xtask target-state` already exists, but the
  repo still needs a joined support/test matrix that connects command support,
  docs claims, manifest metadata, public Rust exports, and required test
  evidence with deterministic failures.
