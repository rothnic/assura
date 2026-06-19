# Support Matrix Surface Expansion Implementation

## Goal

Implement `docs/goals/assura-rule-support-matrix-surface-expansion.md` by
extending `extensions.support_matrices` with bounded docs support-claim sources
and manifest/package-facing surface sources.

## What I Already Know

- `SupportMatrixConfig` currently supports `command_contracts`, `rust_exports`,
  and explicit `entries`.
- Current support-matrix runtime checks command surfaces from
  `.assura/command-surface.yml` and public Rust exports from configured files.
- The first implementation slice should not duplicate manifest-semantics
  field-level validation.
- The goal requires deterministic docs support claims and explicit configured
  source files or narrow globs.

## Requirements

- Add explicit config fields for docs support-claim sources and manifest
  surfaces.
- Require discovered docs/package/binary surfaces to be present in support
  matrix rows.
- Detect docs claims that call a surface supported when the configured matrix
  row is experimental, internal, roadmap, or unsupported.
- Add fixtures covering passing and failing docs/manifest behavior.
- Dogfood on Assura with bounded current docs and Cargo manifests.
- Preserve command-surface docs, manifest-semantics, test-relationship,
  release-contract, docs-lifecycle, and target-state gates.

## Acceptance Criteria

- [x] Config notation documents the new support-matrix source fields.
- [x] Runtime checks support docs claim sources and manifest/package sources.
- [x] Tests cover passing and failing docs/manifest support matrix cases.
- [x] `.assura/config.yml` dogfoods the expanded sources with explicit paths.
- [x] Required validation and independent review pass before PR.

## Out Of Scope

- No broad natural-language prose classifier.
- No license/source policy, dependency usage analysis, or semver checks.
- No replacement for `extensions.manifest_semantics`.

## Technical Notes

- Goal: `docs/goals/assura-rule-support-matrix-surface-expansion.md`.
- Runtime: `src/cli/check/support_matrix.rs`.
- Config structs: `src/config/config/extensions.rs`.
- Config validation: `src/config/config/validation/support_matrices.rs`.
- Compiled artifacts: `src/cli/check/compiled_artifact_extensions.rs`.
- Existing tests: `tests/custom_constraints_tests.rs`,
  `crates/assura-check-cli/tests/compiled_support_matrix_cli.rs`.
