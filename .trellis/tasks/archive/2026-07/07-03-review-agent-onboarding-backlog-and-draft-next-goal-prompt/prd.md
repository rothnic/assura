# Review Agent Onboarding Backlog And Draft Next Goal Prompt

## Goal

Review the live Assura backlog after the agent-ready onboarding work and create
a concrete next-goal prompt with measurable subgoals and completion gates.

## What I Already Know

- PR #139 is open and currently green remotely.
- The local branch is ahead of PR #139 by nine commits.
- The roadmap marks Agent-Ready Project Onboarding as completed locally.
- The roadmap recommends `docs/goals/assura-performance-polish-program.md` as
  the next planned lane.
- Performance polish must stay separate from onboarding unless PR review or
  product regressions reopen onboarding.

## Requirements

- Inspect live roadmap, current PR state, local branch state, and relevant goal
  docs before choosing the next work.
- Identify the most important next steps in order.
- Create a repo-local goal prompt that another agent can execute directly.
- Make each subgoal measurable, with explicit proof gates and review blocking
  criteria.
- Include current branch/PR cleanup as the first subgoal before larger
  performance work.
- Keep the prompt generic and durable enough to survive context compaction.

## Acceptance Criteria

- [ ] New goal prompt file exists under `docs/goals/`.
- [ ] The prompt names exact source-of-truth files and current PR/branch state.
- [ ] The sequence has measurable subgoals with done criteria.
- [ ] Validation commands are explicit.
- [ ] Reviewer blocking criteria are explicit.

## Validation Commands

- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

## Out Of Scope

- Implementing the performance polish work.
- Pushing PR #139 or changing GitHub state.
- Reopening agent-ready onboarding except to capture branch/PR closure work.
