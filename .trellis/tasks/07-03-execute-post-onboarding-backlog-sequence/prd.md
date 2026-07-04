# Execute Post-Onboarding Backlog Sequence

## Goal

Complete `docs/goals/assura-post-onboarding-backlog-execution-sequence.md`
end to end: publish the local PR #139 follow-up commits, revalidate the
performance-polish lane from live evidence, implement checked native
performance reporting, optimize measured bottlenecks, and close with
independent review evidence.

## What I Already Know

- Current branch is `codex/agent-ready-onboarding-backlog`.
- PR #139 is open at remote head `c0e7b881f866027b087afc0ee6580e83f886f4e2`
  and current checks are green on that old head.
- Local branch is ahead of
  `origin/codex/agent-ready-onboarding-backlog` by 10 commits.
- The roadmap marks Agent-Ready Project Onboarding completed locally and
  routes the next work through
  `docs/goals/assura-post-onboarding-backlog-execution-sequence.md`.
- The performance lane remains separate in
  `docs/goals/assura-performance-polish-program.md`.

## Requirements

- Do not start performance implementation until PR #139 is updated or the
  handoff is explicitly parked with evidence.
- Preserve the goal's ordered subgoals and record progress in the goal file
  before and after major phases.
- Revalidate performance claims from checked artifacts, not intuition.
- Keep LS-Lint no-slower gates separate from Assura-native performance rows.
- Use existing `benches/`, `xtask`, checked JSON history, and website data
  surfaces instead of creating a disconnected benchmark path.
- Run independent review before treating the full sequence as done.

## Acceptance Criteria

- [ ] PR #139 remote branch includes the 10 local commits and has green checks
      on the new head.
- [ ] PR #139 has a body update or comment naming the follow-up scope.
- [ ] `docs/goals/assura-performance-polish-program.md` matches current
      performance artifacts and ranks implementation targets from evidence.
- [ ] Native performance fixture matrix covers the required fixture classes and
      reports required metadata.
- [ ] Native performance report artifacts and website data distinguish
      LS-Lint comparison rows from Assura-native rows.
- [ ] Highest-impact rows have before/after proof or bounded written
      rationale.
- [ ] Final release gates and independent review pass, or any blocker is
      recorded with exact missing input.

## Validation Commands

- `git status --short --branch`
- `git log --oneline origin/codex/agent-ready-onboarding-backlog..HEAD`
- `gh pr view 139 --json state,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

Later subgoals add the performance-specific build, benchmark, no-slower, native
regression, full workspace check, and test gates named in the goal file.

## Out Of Scope

- Reopening agent-ready onboarding without a current PR review, CI, or product
  regression blocker.
- Relaxing LS-Lint no-slower requirements.
- Adding a persistent store or cache dependency before same-fixture proof.
