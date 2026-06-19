# Support Matrix Surface Expansion Goal Definition

## Goal

Create and validate the next executable Assura goal for expanding
`extensions.support_matrices` beyond the completed first command/Rust-export
slice.

## What I Already Know

- PR #61 completed the first reusable support matrix slice for command
  contracts and public Rust exports.
- PR #64 completed the first manifest-semantics slice for configured Cargo
  metadata.
- PR #68 completed the first test-relationship slice.
- `docs/analysis/2026-06-09-assura-best-practice-target-state.md` still has a
  P0 gap: the support matrix should join CLI commands, docs claims, manifest
  metadata, and public Rust exports.
- `docs/analysis/2026-06-18-goal-12-support-test-matrix-review.md` shows the
  current `cargo xtask target-state` verifier already performs some of this
  join, but the reusable `assura check` support-matrix rule does not.

## Requirements

- Add a new goal under `docs/goals/` rather than reopening the completed first
  support-matrix goal.
- Keep the first implementation slice bounded to docs support-claim sources and
  manifest/package surfaces.
- Reuse existing completed detectors where possible instead of duplicating
  command-surface docs, manifest-semantics, or test-relationship rules.
- Include objective, current gap, user certainty bar, scope, non-goals,
  definition of done, validation commands, review tasks, and blocking criteria.
- Route roadmap and target-state docs to the new goal as the next support
  matrix candidate.

## Acceptance Criteria

- [x] Goal file exists and is distinct from the completed first-slice
      `assura-rule-public-surface-support-matrix.md`.
- [x] Goal is validated against live `.assura/config.yml`, target-state,
      support matrix implementation, manifest semantics, and Goal 12 review
      evidence.
- [x] Roadmap points to the new goal as the next executable candidate.
- [x] Target-state backlog points to the new goal for the remaining P0 support
      matrix join gap.
- [x] `cargo run --quiet -- check --format json .`, `cargo xtask evidence`,
      and `git diff --check` pass.

## Out Of Scope

- No implementation changes to `src/**` or `.assura/config.yml` in this
  planning task.
- No broad natural-language docs classifier.
- No license/source policy, semver compatibility, or dependency usage analysis.

## Technical Notes

- Completed first-slice support matrix goal:
  `docs/goals/assura-rule-public-surface-support-matrix.md`.
- Target-state evidence:
  `docs/analysis/2026-06-09-assura-best-practice-target-state.md`.
- Goal 12 joined-matrix review:
  `docs/analysis/2026-06-18-goal-12-support-test-matrix-review.md`.
- Support matrix implementation: `src/cli/check/support_matrix.rs`.
- Manifest semantics implementation: `src/cli/check/manifest_semantics.rs`.
