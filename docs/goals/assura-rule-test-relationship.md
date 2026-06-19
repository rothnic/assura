---
id: goal-assura-rule-test-relationship
type: goal
title: Assura test relationship rule
status: planned
created: 2026-06-08
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - docs/goals/assura-rule-cargo-manifest-semantics.md
  - tests
  - src/cli
  - crates/assura-check-cli/tests
---

# Assura Test Relationship Rule

## Objective

Create configurable rules for test-to-code relationships, ignored tests,
fixture ownership, and roadmap-only test paths.

## Revalidation Result

`valid`, with narrowed first-slice scope.

The public-surface support matrix first slice landed in PR #61, and the Cargo
manifest semantics first slice landed in PR #64. Those rules now provide
classified command, public Rust export, and manifest-policy inputs that a test
relationship rule can consume or mirror without hard-coding Assura-specific
surfaces.

`cargo xtask target-state` passes on current `origin/master`, so this is not a
live repository repair. It remains a valid P0 detector because the target-state
analysis still identifies supported-surface test evidence, ignored/manual test
classification, and fixture ownership as only partially aligned.

## User Certainty Bar

A maintainer should be able to declare which source, CLI, fixture, and support
surfaces require which test evidence, then get an actionable `assura check`
finding when supported behavior loses coverage, an ignored test lacks an
accepted reason, or a fixture family has no explicit owner/purpose.

## Current Gap

- Assura has broad tests, but supported surfaces are not enforced through a
  reusable source-to-test relationship rule.
- Support-matrix rows classify public command and Rust export families, but
  those classifications are not yet joined to required test families.
- Ignored/manual tests and fixture directories can drift unless a local
  target-state check or reviewer notices them.
- Fixture families are currently governed by structure rules and conventions,
  not by a product-level relationship detector that other repositories can
  reuse.

## Detector Hypothesis

Map configured source globs to required test globs, track ignored tests by
reason, and classify fixture families through explicit config instead of broad
exclusions.

## First Slice Scope

- Add explicit config notation for one or more test relationship policies.
- Accept configured source globs, required test globs, fixture families, and
  ignored/manual test allowlists with stable reason categories.
- Check at least three inventory sources:
  - source paths or supported surface identifiers;
  - test files matching configured evidence globs;
  - ignored/manual Rust tests and fixture-family directories.
- Report missing test evidence, unclassified ignored tests, and unowned fixture
  families with file/path, relationship id, and configured policy context.
- Add fixtures independent of Assura's own tests and a self-check example that
  dogfoods the rule on at least one supported Assura surface.

## Non-Goals

- No coverage-percentage reporting.
- No mutation testing or runtime coverage instrumentation.
- No broad semantic proof that a test fully exercises a source file.
- No deletion of experimental/internal modules before the relationship rule can
  distinguish supported behavior from contained evidence code.
- No replacement for Rust's normal test runner, nextest, or CI matrix.

## Definition Of Done

- Test-relationship notation is documented before implementation.
- Passing fixture covers a source glob with matching unit or integration test
  evidence and a declared fixture family.
- Failing fixtures cover missing required tests, an ignored test without an
  accepted reason, and an undeclared fixture family.
- `assura check --format json` reports actionable test-relationship
  violations with path, relationship, and policy context.
- Assura self-check dogfoods the rule without weakening support-matrix,
  manifest-semantics, release-contract, or target-state checks.
- Independent review confirms the rule is reusable outside Assura and does not
  pretend to measure code coverage.

## Required Examples

- Passing: a configured source relationship under `src/cli/check/` has
  integration/unit coverage evidence.
- Failing: ignored test without an accepted reason category.
- Failing: new `tests/fixtures/**` family not listed in config.

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

- R0: Confirm the rule uses explicit configured relationships rather than
  inferring coverage from naming coincidence alone.
- R1: Review ignored/manual test reason categories for clear maintainer
  semantics.
- R2: Review fixture-family ownership checks for reusable behavior outside
  Assura.
- R3: Review JSON diagnostics for source/test/fixture path and relationship
  context.
- R4: Confirm the first slice does not claim coverage percentage or semantic
  test adequacy.
- R5: Confirm the rule composes with support-matrix and manifest-semantics
  outputs without weakening either rule.

## Reviewer Blocking Criteria

Block the PR if the rule implies test coverage percentages, if relationships
are hard-coded to Assura paths, if ignored tests can bypass reason categories,
if fixture families can remain unowned, or if diagnostics do not identify the
specific relationship and missing evidence.

## Progress Log

- 2026-06-19: Revalidated after Cargo Manifest Semantics PR #64 and archive PR
  #65 merged. Result: valid with narrowed first-slice scope. The next slice
  should implement configurable source/test evidence, ignored/manual test
  reason categories, and fixture-family ownership checks while leaving coverage
  percentages and semantic test adequacy to external tools or future goals.
- 2026-06-19: Implemented the first reusable test-relationship slice under
  Trellis task `06-19-test-relationship-rule-implementation`: added
  `extensions.test_relationships` config, semantic config validation, runtime
  source/test evidence checks, ignored/manual test classification,
  fixture-family ownership checks, compiled artifact portability, Assura
  dogfood policy, notation docs, and focused runtime/compiled-config coverage.
  Local gates run so far: `cargo fmt --all -- --check`,
  `cargo test --all-targets --quiet`, `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo xtask target-state`,
  `cargo run --quiet -- check --format json .`, and focused
  `test_relationship` test targets.
- 2026-06-19: Independent review found the initial ignored-test allowlist was
  file-scoped, the required example overclaimed `src/cli/check/**`, subpath
  checks could false-positive when evidence lived outside the checked subtree,
  and fixture-family schema did not prove families were under configured roots.
  Fixed the rule to classify ignored tests by file pattern plus test function,
  use project-wide test evidence for scoped source checks, validate fixture
  families under configured roots, and narrow this goal's passing example to
  configured source relationships rather than whole-tree coverage.
