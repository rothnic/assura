---
id: goal-assura-rule-cargo-manifest-semantics
type: goal
title: Assura Cargo manifest semantics rule
status: completed
created: 2026-06-08
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-06-09-assura-best-practice-target-state.md
  - .trellis/spec/assura/roadmap.md
  - docs/goals/assura-rule-public-surface-support-matrix.md
  - Cargo.toml
  - crates/assura-check-cli/Cargo.toml
---

# Assura Cargo Manifest Semantics Rule

## Objective

Create configurable Cargo manifest checks for claims, features, workspace
members, bins, and metadata that Assura can validate without replacing Cargo
or specialist tools.

## Revalidation Result

`valid`, with narrowed first-slice scope.

The public surface support matrix first slice landed in PR #61 and now provides
explicit support classification for command families and public Rust export
families. `cargo xtask target-state` passes on current `origin/master`, so the
manifest-semantics work is not a live repository repair. It remains a valid P0
product detector because the current target-state analysis still classifies
workspace manifest metadata drift as only partially aligned.

The first implementation should focus on deterministic Cargo metadata policy
that can run from configured manifest files and `cargo metadata` output. It
should not try to solve dependency usage, license/source policy, or semver
compatibility in the first slice.

## User Certainty Bar

A Rust workspace maintainer should be able to declare package metadata policy
for public and internal crates and get an actionable `assura check` finding
when a manifest drifts from version, MSRV, publish, license, description,
keyword, binary, or workspace-member expectations.

## Current Gap

- The root package has rich public metadata, but internal workspace crates can
  drift from inherited version, MSRV, license, description, or publish policy
  without a reusable Assura finding.
- Release-contract checks cover explicit artifact drift, not Cargo manifest
  metadata policy.
- Support-matrix checks classify commands and Rust exports, but manifest-facing
  claims such as descriptions, keywords, package publish status, and binary
  names remain outside that detector.

## Completion Result

Completed as a first reusable rule slice in PR #64 and archived in PR #65. The
shipped surface adds explicit `extensions.manifest_semantics` notation,
semantic config validation, structured TOML runtime checks, workspace-inherited
package field resolution, binary metadata checks, CLI JSON diagnostics,
compiled-config portability, fixture coverage, and Assura self-dogfood rows.

Dependency usage analysis, license/source policy, semver compatibility, and
feature-policy depth remain follow-up owners outside this completed first
slice.

## Detector Hypothesis

Parse `Cargo.toml` structurally, compare selected fields against configured
support claims, feature-policy allowlists, required binary metadata, and
forbidden release-positioning phrases.

## First Slice Scope

- Add explicit config notation for one or more Cargo manifest policy entries.
- Accept configured manifest paths and expected package/workspace semantics.
- Check a bounded set of metadata fields:
  - package version or workspace-inherited version;
  - rust-version or workspace-inherited MSRV where applicable;
  - license or workspace-inherited license;
  - publish policy for public versus internal crates;
  - package description and keywords for configured required/forbidden terms;
  - binary names declared in `[[bin]]` entries when configured.
- Use structured TOML parsing and/or `cargo metadata`; do not parse manifests
  with ad hoc line matching for fields TOML exposes.
- Add fixtures independent of the Assura repository and a dogfood config row
  for the root package plus `crates/assura-check-cli`.

## Non-Goals

- No dependency usage analysis for optional dependencies in the first slice;
  leave that to a future dependency hygiene or external-tool integration.
- No license/source policy rollout beyond checking configured manifest fields.
- No semver API compatibility checks.
- No replacement for Cargo validation, `cargo machete`, `cargo audit`, or
  release-contract checks.

## Definition Of Done

- Manifest-semantics notation is documented before implementation.
- Passing fixture covers a public root crate and an internal workspace crate.
- Failing fixtures cover missing required metadata, forbidden release claim
  text, incorrect internal publish policy, and mismatched binary metadata.
- `assura check --format json` reports actionable manifest-semantics
  violations with file, package, field, and policy context.
- Assura self-check dogfoods the rule without weakening existing
  support-matrix, release-contract, or target-state checks.
- Independent review confirms the rule is reusable outside Assura and does not
  duplicate Cargo or specialist dependency tools.

## Required Examples

- Passing: structure-first package description and `structure` keyword.
- Failing: package description claiming dependency graph validation.
- Failing: internal helper crate has publish settings inconsistent with policy.
- Failing: configured binary metadata is absent or mismatched.

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

- R0: Confirm the rule uses structured TOML or Cargo metadata rather than
  brittle string parsing for manifest fields.
- R1: Review public versus internal crate policy semantics and defaults.
- R2: Review fixtures for reusable root crate, workspace member, binary, and
  forbidden-claim examples.
- R3: Review JSON diagnostics for file, package, field, and configured policy
  context.
- R4: Confirm the first slice does not duplicate `cargo machete`,
  `cargo audit`, release-contract, or support-matrix behavior.

## Reviewer Blocking Criteria

Block the PR if manifest fields are parsed with fragile line matching, if
internal crate publish policy is ambiguous, if the implementation hard-codes
Assura package names instead of using configured policy, if diagnostics do not
identify the package and field, or if existing target-state/support-matrix
coverage is weakened.

## Progress Log

- 2026-06-19: Revalidated after public-surface support matrix PR #61 and
  archive PR #62 merged. Result: valid with narrowed first-slice scope.
  The next slice should implement configurable Cargo manifest metadata policy
  for public/internal crates, using structured TOML or Cargo metadata and
  leaving optional dependency usage, license/source policy, and semver checks
  for later owners.
- 2026-06-19: Implemented the first reusable manifest-semantics slice under
  Trellis task `06-19-cargo-manifest-semantics-implementation`: added
  `extensions.manifest_semantics` config, structured TOML runtime validation,
  semantic config validation, compiled artifact portability, Assura dogfood
  manifest policies, notation docs, and focused fixture/CLI coverage. Local
  gates run so far: `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo run --quiet -- check --format json .`.
- 2026-06-19: Independent review found two gaps before PR creation:
  workspace-inherited `version.workspace = true`, `rust-version.workspace =
  true`, and `license.workspace = true` fields produced false positives, and
  diagnostics did not consistently include package context. Fixed inherited
  field resolution from `[workspace.package]`, updated diagnostics to include
  package names, and added regression coverage. Final local gates passed:
  `cargo fmt --all -- --check`, `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo xtask target-state`, `cargo run --quiet -- check --format json .`,
  `cargo xtask evidence`, `cargo xtask docs`, and `git diff --check`.
- 2026-06-19: Completed the Cargo manifest semantics first slice in PR #64 and
  archived Trellis task `06-19-cargo-manifest-semantics-implementation` in PR
  #65. Remaining dependency hygiene, license/source policy, semver, and
  feature-policy checks route to future specialized goals.
