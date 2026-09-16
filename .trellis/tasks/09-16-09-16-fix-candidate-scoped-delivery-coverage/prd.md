# Fix candidate-scoped delivery coverage

## Goal

Allow one independently proven delivery candidate to close when unrelated
worktree registrations are unavailable, while keeping fleet-wide audit
coverage incomplete and fail-closed.

## Requirements

1. Keep `Inventory.coverage.complete` and strict audit behavior unchanged.
2. For an integration close, require resolved refs/tasks, fresh remote-base
   verification, complete GitHub coverage, all typed evidence, and no dirty or
   unavailable worktree that matches the candidate.
3. Do not let unrelated `WORKTREE_COVERAGE_UNAVAILABLE` findings turn a valid
   candidate into an unsupported delivery claim; retain those findings in the
   audit report.
4. Add regressions for an unrelated unavailable worktree being allowed and a
   candidate-owned unavailable worktree remaining blocked, with no mutation on
   rejection.

## Boundaries

Do not prune worktrees, change refs, weaken evidence validation, modify the
preserved root checkout, tag, publish, deploy, or edit release behavior.

## Verification

Run the focused delivery contract suite, process checks, Assura structure
checks, and the applicable workflow/hosted gates before integration.
