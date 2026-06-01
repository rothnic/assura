---
id: goal-assura-codex-feedback-install-status-verify
type: goal
title: Assura Codex feedback install status verify
status: completed
created: 2026-05-31
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/codex-agent-feedback.md
  - docs/goals/assura-agent-nudge-mvp.md
  - docs/goals/assura-real-project-policy-proof.md
---

# Assura Codex Feedback Install Status Verify

## Objective

Turn the current optional Codex hook proof into user-verifiable Codex hook
output through the stable `assura check` CLI surface, without making Codex hooks
mandatory for ordinary developer workflows or adding one feedback management
entrypoint or output format per agent.

The goal is complete only when a reviewer can start from a clean checkout, run
`assura check` with stable agent output and feedback filters, verify valid
Codex `additionalContext` when the Codex delivery adapter is selected, append
the command to existing project hooks without overwriting them, and understand
exactly what still requires manual Codex
approval.

## Why This Goal Exists

The current hook proof defines the native `UserPromptSubmit` feedback path and
documents a source-checkout command, but an end user still has to reason through
too much manually:

- how to get stable agent feedback and optional Codex hook JSON from the stable
  Assura CLI,
- how to append the Assura command to `.codex/hooks.json`,
- whether Codex hook support is enabled and approved,
- whether the hook is advisory or blocking,
- whether `assura check` can produce stable agent feedback and valid Codex
  `additionalContext`, and
- how to merge Assura feedback into existing project hooks without overwriting
  other tools.

This next slice should move that uncertainty into explicit `assura check`
commands, docs, and tests.

## Scope

- Add a supported `assura check --format agent` output path in the main Assura
  CLI.
- Add `--agent codex` as an optional delivery adapter for Codex
  `UserPromptSubmit` hook JSON, not as a separate agent-specific format.
- Reuse existing check feedback options for agents:
  - `--min-severity`,
  - `--max-issues`,
  - `--warn` for advisory hook behavior.
- Provide tests that run `assura check --format agent` and `assura check
  --format agent --agent codex` against passing and failing fixture projects
  and confirm stable agent JSON plus valid Codex hook JSON.
- Document an append-only `.codex/hooks.json` merge example that never
  overwrites existing hooks.
- Keep normal `assura check` and developer workflows unaffected unless the user
  explicitly installs or verifies Codex feedback.

## Out of Scope

- Publishing the npm package to the registry.
- Adding new per-agent feedback management binaries.
- Adding one `--format <agent>-hook` value per agent.
- Long-running daemon or editor-session reuse.
- Automatic Codex `/hooks` approval or user-level `features.hooks` mutation.
- Autonomous repair after feedback is injected.
- Replacing repo-local `AGENTS.md` or `.agents/skills/` guidance.

## Acceptance Criteria

- `assura check --format agent` emits `assura.agent-feedback.v1` JSON in
  machine-readable form.
- `assura check --format agent --agent codex --warn` exits `0` for validation
  failures and includes advisory Codex feedback context.
- `assura check --format agent --agent codex` without `--warn` exits `1` for
  validation failures and still emits valid hook JSON.
- Filtering with `--min-severity` and `--max-issues` affects agent and Codex
  delivery output.
- `assura check --format codex-hook` is rejected.
- Docs show the stable `assura check` hook command and do not imply a separate
  per-agent feedback binary or per-agent format is the primary API.
- `.trellis/spec/assura/codex-agent-feedback.md` is updated if command
  signatures or hook contracts change.

## Validation Commands

Run and pass, or document exact blockers:

```bash
cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run
node --run verify:fast
node --run verify:docs
cargo run --quiet -- check --format json .
cargo run --quiet -- check --format agent .
cargo run --quiet -- check --format agent --agent codex . --warn
```

If the implementation touches Rust CLI surfaces, also run:

```bash
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
```

## Reviewer Blocking Criteria

Block the PR if:

- it edits or overwrites existing Codex hooks without an explicit merge path,
- it claims Codex hook approval or user-level hook enablement can be automated
  without proof,
- hook output cannot be consumed by Codex as structured hook JSON,
- it reintroduces `assura-codex-feedback`, `assura check --format codex-hook`,
  or any new per-agent CLI/format shape as the stable user-facing API,
- docs imply a separate per-agent feedback management binary or per-agent format
  is the stable API,
  or
- normal `assura check` workflows become dependent on Codex hook setup.

## Handoff Prompt

```text
/goal docs/goals/assura-codex-feedback-install-status-verify.md
```

## Progress Log

- 2026-05-31: Started execution. Fast-forwarded `master` to the commit that
  added this goal, created branch
  `codex/assura-codex-feedback-install-status-verify`, created Trellis task
  `.trellis/tasks/05-31-codex-feedback-install-status-verify`, seeded PRD and
  context files, and loaded the Assura Codex feedback spec before product
  changes.
- 2026-05-31: Corrected product direction after review: the stable surface is
  `assura check`, not a new per-agent feedback management binary.
- 2026-05-31: Completed corrected validation pass. Context health before
  handoff: active goal tracker not exposed after compaction; relevant prior
  context was the user correction away from per-agent feedback management
  binaries, the PR branch rewrite, and the stable `assura check` API. Passed
  `cargo test --test cli_command_surface_tests --quiet`,
  `cargo test --all-targets --quiet`, `cargo clippy --all-targets
  --all-features -- -D warnings`, `cargo fmt --all -- --check`, `cd
  integrations/agents/codex && npm run lint && npm test && npm run build &&
  npm pack --dry-run`, `node --run verify:fast`, `node --run verify:docs`,
  `cargo run --quiet -- check --format json .`, hook JSON parsing smoke, and
  `git diff --check`.
- 2026-05-31: Addressed review-agent findings before publication: added
  passing-project, severity-filter, max-issue, and status-rejection CLI
  regressions, refreshed help text, and re-ran the full validation set
  successfully.
- 2026-05-31: Addressed the remaining format-design issue: replaced the
  Codex-specific check format with stable `assura check --format agent` output
  and `--agent codex` as the optional delivery adapter. Added regressions for
  generic agent JSON, Codex hook delivery, `codex-hook` rejection, and misuse of
  `--agent` without `--format agent`.
- 2026-05-31: Added a durable direction lock after the user called out context
  drift across sessions. Future work must treat `assura-codex-feedback`,
  `assura check --format codex-hook`, and one-entrypoint/one-format-per-agent
  designs as superseded unless `.trellis/spec/assura/codex-agent-feedback.md`
  is explicitly changed first.
- 2026-05-31: Removed package executable tech debt from the superseded
  direction: the npm package no longer exposes feedback command binaries, docs
  no longer present package commands as fallback CLI surfaces, and the active
  API remains `assura check --format agent` with optional `--agent codex`.
- 2026-05-31: Closed post-merge bookkeeping after PR #15 landed on `master`.
  Archived Trellis task
  `.trellis/tasks/05-31-codex-feedback-install-status-verify` and moved the
  Agent Feedback MVP roadmap owner back to the real-project policy proof task.
