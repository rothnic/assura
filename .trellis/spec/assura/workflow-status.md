# Workflow Status Snapshots

Every Assura task should make the current workflow state obvious to both the
developer and future agents. Do not rely on conversation memory to explain
where the work stands.

## When To Include A Snapshot

Include a concise workflow status snapshot when:

- Starting or resuming any implementation, docs, cleanup, CI, or review task.
- Reporting after a meaningful action such as a commit, push, PR update, test
  run, or CI result.
- Pausing, deferring, or choosing not to fix a known issue.
- Handing off next steps at the end of a turn.

For tiny direct answers that do not touch repo state, a snapshot is optional.

## Snapshot Format

Use this shape in user-facing updates and final summaries when the work is
non-trivial:

```text
Workflow status:
- Roadmap epic: <3-5 word epic from .trellis/spec/assura/roadmap.md>
- Task: <active Trellis task or "none">
- State: <planning | in_progress | review | paused | completed>
- Branch/PR: <branch and PR if relevant>
- Git: <branch, worktree, staged/unstaged/untracked summary>
- Current focus: <one sentence>
- Known blockers/deferred items: <short list or "none">
- Options: <2-3 realistic next moves>
- Recommendation: <one concrete next action>
```

Keep it short. The goal is orientation, not a second report.

## Rules

- Prefer facts from repo state: `task.py current --source`, `git status`,
  current branch, PR checks, and active Trellis specs.
- Pull the active roadmap epic from `.trellis/spec/assura/roadmap.md`.
- Include the git summary even when the tree is clean. State staged,
  unstaged, and untracked status explicitly.
- If a check is paused or a failure is accepted temporarily, link it to a
  Trellis spec or task that owns the follow-up.
- Always separate options from the recommendation. Options show tradeoffs;
  recommendation says what the agent should do next.
- Do not bury the current task or branch in prose when a structured snapshot
  would make it clearer.
- If a task has no explicit next step, create or update the Trellis task/spec
  so the next agent can continue without rediscovery.

## Current Assura Default

Until the bootstrap task is complete, the expected next work is tooling and
workflow stabilization:

1. Keep the self-enforcement PR honest and focused.
2. Stabilize CI signal quality.
3. Run dedicated rustfmt and clippy cleanup iterations.
4. Reduce the `assura check .` baseline to zero.
5. Re-enable Windows CI after the documented linker issue is fixed.

## Git Summary Format

Use this concise form:

```text
Git: branch=<name>; worktree=<path>; staged=<none|summary>;
unstaged=<none|summary>; untracked=<none|summary>
```

If a PR exists, include it in `Branch/PR` rather than overloading the git line.
