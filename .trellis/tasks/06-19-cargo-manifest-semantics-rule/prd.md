# Cargo Manifest Semantics Rule Validation

## Objective

Revalidate `docs/goals/assura-rule-cargo-manifest-semantics.md` after the
public surface support matrix rule landed in PR #61, then prepare the first
implementation slice for configurable Cargo manifest metadata drift checks.

## Current Evidence

- PR #61 merged the reusable `extensions.support_matrices` rule and Assura
  self-dogfoods command and public Rust export support classification.
- PR #62 archived the completed public-surface Trellis task.
- `cargo xtask target-state` passes on current `origin/master`, so there is no
  live target-state failure to fix before planning the next detector.
- The target-state backlog still lists Cargo manifest semantics as a P0
  detector for workspace metadata, internal/public crate policy, MSRV, publish
  status, and release metadata consistency.

## Scope

- Mark the public-surface support matrix goal/roadmap item complete.
- Revalidate the manifest-semantics goal against current roadmap and
  target-state analysis.
- Add explicit first-slice boundaries, definition of done, validation commands,
  review tasks, and reviewer blocking criteria.
- Leave implementation of the reusable manifest rule to the next branch/PR
  unless the revalidation proves the goal is already achieved.

## Non-Goals

- No Rust product-code changes in this validation task.
- No dependency-audit, license/source, or semver policy rollout.
- No broad test-relationship or module-topology implementation.

## Definition Of Done

- Roadmap routes the next planned candidate to the manifest-semantics rule, not
  the completed public-surface matrix rule.
- Manifest-semantics goal records a current `valid` or non-valid revalidation
  result with evidence.
- The refreshed goal has enough scope, proof gates, and review criteria for an
  implementation agent to start without redoing this planning pass.
- Trellis context files contain only real context entries.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
python3 ./.trellis/scripts/task.py validate 06-19-cargo-manifest-semantics-rule
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check origin/master...HEAD
```
