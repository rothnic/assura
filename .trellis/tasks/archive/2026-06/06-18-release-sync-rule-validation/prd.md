# Release Sync Rule Validation

## Goal

Revalidate `docs/goals/assura-rule-release-sync.md` after Goal 13 added
deterministic release/performance target-state checks, then decide whether the
goal remains valid, is superseded, or needs a narrower next implementation
slice.

## Current Evidence

- PR #55 added explicit release artifact, installer, CI smoke, checksum, public
  docs, performance provenance, and public performance-copy checks to
  `cargo xtask target-state`.
- PR #56 archived Goal 13.
- PR #57 updated the roadmap to route the next candidate to this release-sync
  rule validation.
- The existing release-sync goal is still a short hypothesis and does not yet
  distinguish repo-local target-state governance from a user-facing Assura rule
  family.

## Scope

- Revalidate the release-sync goal against current repo state.
- Update the goal document with one of: `valid`, `superseded`,
  `already-achieved`, or `refresh-needed`.
- If valid, narrow the first implementation slice to missing proof gates and
  reviewer blocking criteria.
- Do not implement the rule family in this validation slice.

## Acceptance Criteria

- [ ] The goal doc has a current revalidation result backed by live evidence.
- [ ] The remaining gap is explicit and not duplicative of Goal 13 target-state
      checks.
- [ ] The next implementation slice has objective, scope, definition of done,
      validation commands, review tasks, and blocking criteria.
- [ ] Trellis task context validates.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
python3 ./.trellis/scripts/task.py validate 06-18-release-sync-rule-validation
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```
