# Revalidate Next Reusable Rule Goal

## Objective

Update the roadmap after the merged Cargo manifest semantics first slice and
revalidate the next reusable rule candidate before implementation starts.

## Current State

- PR #64 merged the first reusable Cargo manifest semantics rule slice.
- PR #65 archived `06-19-cargo-manifest-semantics-implementation`.
- `.trellis/spec/assura/roadmap.md` still routes the next agent to Cargo
  manifest semantics.
- The target-state analysis still lists the test-relationship rule as a P0
  detector after support matrix and manifest semantics work.

## Scope

- Mark Cargo manifest semantics complete in roadmap/goal routing docs.
- Revalidate `docs/goals/assura-rule-test-relationship.md` against current
  roadmap and target-state evidence.
- Add enough objective, scope, definition of done, validation, review tasks,
  and blocking criteria for the next implementation agent to start safely.
- Keep this as a planning/docs slice; do not implement runtime validation.

## Definition Of Done

- Roadmap no longer points to completed Cargo manifest semantics as planned
  next work.
- Test relationship is the explicit next reusable rule candidate with a
  narrowed first-slice scope.
- The goal doc states whether it is valid, already achieved, or superseded.
- Local docs/Trellis validation passes.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check origin/master...HEAD
```
