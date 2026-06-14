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

Current shaping default for the live branch `codex/ls-lint-realistic-parity-core-performance`:

1. Keep the handoff aligned with the repaired roadmap before new write-capable work.
   - `.trellis/spec/assura/roadmap.md` truthfully names **Beyond Ls-Lint Rules** as the active epic.
   - The current owning task is `.trellis/tasks/05-21-bring-pr11-performance-home`.
   - The live branch and recent commits are still release/performance verification work, so future workers should stay on that lane instead of reviving the older Agent Nudge ownership story.
2. Treat the fresh verification matrix as source of truth.
   - Fresh 2026-06-12 reruns turned green again: `cargo test --all-targets --quiet` now passes.
   - Both named exact repro commands still pass in isolation: `cargo test --quiet cli::check::prepared::tests::prepared_check_reloads_when_config_changes -- --exact` and `cargo test --quiet cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix -- --exact`.
   - Consume the stale suite-red/shared-state story and keep the lane framed as green verification plus a remaining docs/handoff review gate.
3. Keep worker capacity at zero until the broad dirty branch has a narrower reviewable handoff.
   - The checkout is still dirty in `.trellis/spec/assura/roadmap.md`, `.trellis/spec/assura/workflow-status.md`, and `.trellis/tasks/05-21-bring-pr11-performance-home/prd.md`.
   - The next shaping step is to collapse the docs/handoff batch to this refreshed green-verification checkpoint and turn it into one reviewable PR-update handoff before any claim says the branch is review-ready.
4. Next deterministic inspection commands:
   - `cargo test --all-targets --quiet`
   - `cargo test --quiet cli::check::prepared::tests::prepared_check_reloads_when_config_changes -- --exact`
   - `cargo test --quiet cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix -- --exact`
   - `git status --short`
   - `git diff --stat`
   - `git diff -- .trellis/spec/assura/roadmap.md .trellis/spec/assura/workflow-status.md`
   - `git diff -- .trellis/tasks/05-21-bring-pr11-performance-home/prd.md`
   - `git diff -- .trellis/spec/assura/roadmap.md .trellis/spec/assura/workflow-status.md .trellis/tasks/05-21-bring-pr11-performance-home/prd.md`
   - `git log --oneline -5`

## Git Summary Format

Use this concise form:

```text
Git: branch=<name>; worktree=<path>; staged=<none|summary>;
unstaged=<none|summary>; untracked=<none|summary>
```

If a PR exists, include it in `Branch/PR` rather than overloading the git line.
