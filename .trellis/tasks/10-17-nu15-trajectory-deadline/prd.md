# Diagnose Windows trajectory refresh deadline

## Goal

Find and fix the demonstrated post-merge Windows stable `agent_trajectory_cli` deadline failure while preserving the intentional 2,000 ms default budget, all nine integration tests, reported coverage semantics, process cleanup, and the shallow clone/multiple worktree scenario. The PRD authority is the root-owned NU15 task assignment: `ae-hbx` in the 2026-10-17 new-user-review backlog; implementation authority is limited to this candidate and its scoped acceptance below.

The observed failure is from merge `ba4804bb6abdadc1ccc12c41397dd2402a914171`, workflow run `39094826608`, Windows job `117579008335`: `shallow_clones_and_multiple_worktrees_keep_coverage_honest` failed at `tests/agent_trajectory_cli.rs:77` with `trajectory refresh deadline exceeded`. The pre-merge Windows run passed. Source evidence and the three independent read-only reviews are preserved in `/Users/nroth/workspace/.assura-review-20261017/`.

## Acceptance

- Preserve the default product refresh/inspect timeout at 2,000 ms; do not widen it or weaken assertions, skip coverage, retry unchanged failing runs, or call the failure runner noise without evidence.
- Add a bounded diagnostic that identifies each inspection, discovery/collection stage, elapsed time, and last Git subprocess phase on success and deadline failure, without changing default public output or claiming timings as representative performance. Per-child timings are inclusive process-lifecycle wall time, including setup, output handling, termination and cleanup; they do not isolate Git execution time.
- Use evidence to identify which inspection and serial Git call sites consume the shared deadline. Treat broad contention as a possibility only if the trace supports it; do not attribute an inclusive child duration to Git itself or to Windows process setup/cleanup separately.
- Preserve shallow clone and multiple worktree integration coverage and status-truncation negative control.
- Demonstrate a meaningful local positive/negative diagnostic behavior and run the focused test on the supported Windows lane for final platform evidence. If that lane cannot run from this candidate, state the exact held proof and do not claim the Windows failure fixed.
- Preserve contextual errors, bounded subprocess execution, timeout cancellation, descendant cleanup, and all existing product coverage/report semantics.
- Commit the coherent candidate before independent review; no push, PR creation, merge, release, Beads mutation, or global binary installation in this task.
