# Module Topology Goal Revalidation

## Objective

Update the roadmap handoff after the test-relationship rule merge and
revalidate `docs/goals/assura-rule-module-topology.md` against current
target-state evidence.

## Current Evidence

- PR #68 merged the first reusable `extensions.test_relationships` rule.
- PR #69 archived the completed test-relationship Trellis task.
- `cargo xtask target-state` passes on current `origin/master`, so there is no
  live target-state failure to repair before planning the next detector.
- The remaining planned `assura-rule-*` goal is module topology; release
  contracts, public-surface support matrix, Cargo manifest semantics, command
  surface documentation, and test relationships are complete as first reusable
  slices.

## Scope

- Mark the test-relationship goal and roadmap item complete.
- Route the next planned candidate to the module-topology rule.
- Revalidate the module-topology goal against the current target-state
  analysis and completed prerequisite detector goals.
- Add enough objective, scope, definition of done, validation commands, review
  tasks, and blocking criteria for an implementation agent to start without
  repeating this planning pass.

## Non-Goals

- No Rust product-code changes in this validation task.
- No module-topology implementation in this PR.
- No broad docs lifecycle or release-sync work.

## Definition Of Done

- Roadmap no longer routes to the completed test-relationship rule.
- Test-relationship goal metadata records its completed first-slice result.
- Module-topology goal records a current `valid` or non-valid revalidation
  result with evidence.
- The refreshed module-topology goal has concrete first-slice boundaries, proof
  gates, validation commands, review tasks, and reviewer blocking criteria.
- Trellis context files contain only real context entries.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
python3 ./.trellis/scripts/task.py validate 06-19-06-19-module-topology-revalidation
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check origin/master...HEAD
```
