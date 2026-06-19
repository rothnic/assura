# Docs Lifecycle Goal Definition

## Goal

Create a durable Assura goal for the next roadmap candidate: docs lifecycle and
stale-claim detection.

## What I Already Know

- PR #74 updated the roadmap so docs lifecycle/stale-claim detection is the
  next candidate after module topology completion.
- The target-state analysis classifies dense active docs/history as misaligned
  and calls for detectors covering active/archive lifecycle, stale roadmap
  claims, stale public release/performance claims, and explicit historical
  exceptions.
- Existing `xtask` checks already cover selected hard-coded release,
  performance, support, command-surface, and goal-frontmatter claims.
- Prior first-slice rules in this repo use explicit config extensions instead
  of broad prose inference.

## Requirements

- Add a goal document under `docs/goals/` for the docs lifecycle/stale-claim
  detector.
- Revalidate the goal against current roadmap and target-state evidence.
- Define a narrow first slice that is reusable outside Assura and avoids broad
  natural-language classification.
- Include definition of done, validation commands, review tasks, and reviewer
  blocking criteria.
- Update roadmap routing to point at the new goal path.

## Acceptance Criteria

- The new goal makes clear what is in the first slice and what remains out of
  scope.
- The first slice covers explicit lifecycle metadata and configured stale-claim
  patterns or claim owners, not an implicit prose classifier.
- The roadmap points future agents to the new goal.
- Local docs/Trellis validation passes.

## Out Of Scope

- Implementing the detector in this task.
- Broad archival cleanup of existing docs.
- Replacing existing hard-coded `cargo xtask target-state` checks.
- Claiming semantic understanding of arbitrary prose.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `cargo xtask target-state`
- `git diff --check`
