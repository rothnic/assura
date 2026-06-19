---
id: goal-assura-rule-support-matrix-surface-expansion
type: goal
title: Assura support matrix surface expansion
status: planned
created: 2026-06-19
owners:
  - assura-maintainers
related:
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - docs/goals/assura-rule-cargo-manifest-semantics.md
  - docs/goals/assura-rule-test-relationship.md
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - docs/analysis/2026-06-18-goal-12-support-test-matrix-review.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/config-notation.md
  - .assura/config.yml
---

# Assura Support Matrix Surface Expansion

## Objective

Expand the reusable `extensions.support_matrices` rule from the completed
command/Rust-export first slice into a bounded support-status join across docs
support claims and manifest/package-facing surfaces.

The goal is to move the current target-state join out of `cargo xtask
target-state` special-case logic where practical, without duplicating the
completed command-surface, manifest-semantics, or test-relationship rules.

## Revalidation Result

`valid`.

PR #61 completed the first reusable support-matrix slice for command contracts
and public Rust exports. PR #64 completed configurable manifest-semantics
checks. PR #68 completed first-slice test relationships. Current
`cargo xtask target-state` still performs a broader joined support matrix, and
`docs/analysis/2026-06-09-assura-best-practice-target-state.md` keeps the P0
support-matrix join open because reusable `assura check` coverage does not yet
join docs support claims or manifest/package-facing surfaces.

This goal should be implemented as a second slice, not by reopening the
completed first-slice goal.

## User Certainty Bar

A maintainer should be able to add or expose a command, binary, package,
public Rust export, or public support claim and get an actionable `assura
check` finding when that surface is unclassified or contradicts the configured
support status.

## Current Gap

- `extensions.support_matrices` currently checks configured command contracts
  and public Rust exports.
- `extensions.manifest_semantics` checks manifest metadata, binary names, and
  public/internal package policy, but support-matrix status does not yet join
  those package and binary surfaces.
- `cargo xtask target-state` checks support-policy, compatibility, docs claim,
  manifest, source, and test markers in one joined matrix, but that verifier is
  Assura-specific and not yet reusable through `assura check`.
- Docs can still drift by claiming that an experimental command or internal
  surface is supported unless the claim appears in a target-state scan or a
  separate command-surface docs rule catches the command name.

## First Implementation Slice

Add a bounded reusable support-matrix expansion that can dogfood Assura without
broad natural-language classification:

- Add explicit configuration for docs support-claim sources, limited to named
  docs/website files or narrow globs.
- Add explicit configuration for manifest/package-facing surface sources,
  preferably by reusing configured `extensions.manifest_semantics` policy ids or
  manifest paths instead of reparsing unrelated Cargo metadata in a second way.
- Require surfaces discovered from those configured sources to have rows in the
  same support matrix vocabulary: `supported`, `experimental`, `internal`,
  `roadmap`, or `unsupported`.
- Detect contradictions where a configured docs support-claim source says a
  surface is supported while its support-matrix row is experimental, internal,
  roadmap, or unsupported.
- Dogfood the slice on the smallest current Assura surfaces that prove the
  join, such as `assura info`, `assura performance-report`, root/internal Cargo
  packages, and configured binaries.

## Scope

- Extend `extensions.support_matrices` notation only where it creates a
  reusable join with docs claim sources or manifest/package surfaces.
- Add focused fixtures independent of the Assura repository for passing and
  failing docs-claim and manifest-surface cases.
- Update `.assura/config.yml` with bounded self-dogfood rows only after fixture
  behavior is proven.
- Keep target-state checks in place until reusable `assura check` coverage
  proves equivalent or deliberately narrower behavior.

## Non-Goals

- No broad natural-language stale-doc classifier.
- No license/source policy, dependency usage analysis, or semver compatibility
  checks.
- No replacement for `extensions.manifest_semantics`; reuse its boundaries
  rather than duplicating field-level package metadata validation.
- No test-coverage adequacy or mutation testing.
- No remote GitHub, crates.io, or release API checks.

## Definition Of Done

- Config notation documents the new support-matrix source fields and their
  boundaries before implementation is considered complete.
- Passing fixture covers a supported command/package/binary/docs-claim join.
- Failing fixtures cover an unclassified manifest/package surface and a docs
  claim contradicting the configured support status.
- Assura self-check dogfoods the expanded support matrix with explicit docs and
  manifest/package sources.
- Existing command-surface docs, manifest-semantics, test-relationship,
  release-contract, and target-state gates are not weakened.
- Roadmap and target-state docs are updated after merge to route the next
  candidate based on the new self-check output.
- Independent review confirms the rule remains reusable and does not hard-code
  Assura-specific support rows.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

Docs-only planning changes may use scoped docs/Trellis gates, but any Rust,
Cargo, config, fixture, or behavior change must run the full Rust gates above
before PR.

## Review Tasks

- R0: Confirm the new source fields are explicit and bounded, not broad docs or
  workspace globs.
- R1: Confirm docs support-claim detection uses deterministic markers or
  constrained table parsing, not generic prose interpretation.
- R2: Confirm manifest/package surface inventory reuses manifest-semantics
  boundaries where possible.
- R3: Confirm diagnostics name the matrix id, discovered surface, source file,
  and expected support status.
- R4: Confirm the implementation does not weaken command-surface docs,
  manifest-semantics, test-relationship, release-contract, or target-state
  gates.
- R5: Confirm Assura dogfood config uses concrete docs and manifest/package
  sources with current evidence.

## Reviewer Blocking Criteria

Block the PR if docs claim detection is broad or non-deterministic, if manifest
metadata is parsed with a competing fragile implementation, if support statuses
are inferred implicitly, if diagnostics omit the source surface and matrix id,
or if existing target-state/evidence gates are weakened before reusable
coverage proves equivalence.

## Progress Log

- 2026-06-19: Created after docs lifecycle coverage PR #83 and archive/sync
  PRs #84/#85 completed. Revalidated against current target-state evidence,
  completed support-matrix PR #61, completed manifest-semantics PR #64,
  completed test-relationship PR #68, and Goal 12 joined-matrix review.
- 2026-06-19: Started implementation task
  `.trellis/tasks/06-19-support-matrix-surface-expansion-implementation` on
  branch `codex/support-matrix-surface-expansion`; revalidated current
  `SupportMatrixConfig`, runtime support-matrix checks, manifest-semantics
  boundaries, compiled artifact plumbing, and existing fixture coverage before
  changing code.
