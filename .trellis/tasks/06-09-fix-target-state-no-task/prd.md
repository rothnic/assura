# Fix target-state no-task verifier

## Objective

Make `node --run verify:target-state` pass from a clean `master` checkout when
there is no active Trellis task, while continuing to reject stale or mismatched
active task state during task work.

## Acceptance Criteria

- The workflow-state check treats `task.path == null` as no active task.
- A clean checkout with no active task is accepted.
- An active task still requires `planning` or `in_progress`, matching branch,
  and a PRD artifact.
- `node --run verify:target-state` passes on the follow-up branch.
- The workspace ends clean, with the follow-up PR merged or ready to merge.
