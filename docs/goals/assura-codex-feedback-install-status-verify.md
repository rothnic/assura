---
id: goal-assura-codex-feedback-install-status-verify
type: goal
title: Assura Codex feedback install status verify
status: planned
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

Turn the current optional Codex hook proof into a user-verifiable install,
status, and verify workflow for Assura agent feedback, without making Codex
hooks mandatory for ordinary developer workflows.

The goal is complete only when a reviewer can start from a clean checkout,
build or install the Codex integration package, ask Assura whether the Codex
feedback hook is available and configured, verify the hook output against a
fixture report, and understand exactly what still requires manual Codex
approval.

## Why This Goal Exists

The current hook proof defines the native `UserPromptSubmit` feedback path and
documents a source-checkout command, but an end user still has to reason through
too much manually:

- whether `assura-codex-hook` is installed or only available from source,
- whether `.codex/hooks.json` contains an Assura feedback hook,
- whether Codex hook support is enabled and approved,
- whether the hook is advisory or blocking,
- whether a hook command can produce valid Codex `additionalContext`, and
- how to merge Assura feedback into existing project hooks without overwriting
  other tools.

This next slice should move that uncertainty into explicit commands and tests.

## Scope

- Add a supported command path for Codex feedback install/status/verify in the
  integration package or the main Assura CLI. Choose one owner and document why.
- Provide status output that is useful to agents and humans:
  - package command availability,
  - target hook config path,
  - whether an Assura `UserPromptSubmit` hook entry is present,
  - advisory/blocking mode,
  - Codex hook enablement/approval caveats that cannot be verified locally.
- Provide a verify path that runs the hook against a passing and failing
  fixture report and confirms valid Codex hook JSON.
- Provide an install or merge plan that never overwrites existing hooks without
  explicit user confirmation.
- Keep normal `assura check` and developer workflows unaffected unless the user
  explicitly installs or verifies Codex feedback.

## Out of Scope

- Publishing the npm package to the registry unless it is required to prove the
  install path and can be done with release governance.
- Long-running daemon or editor-session reuse.
- Automatic Codex `/hooks` approval or user-level `features.hooks` mutation.
- Autonomous repair after feedback is injected.
- Replacing repo-local `AGENTS.md` or `.agents/skills/` guidance.

## Acceptance Criteria

- A new or extended CLI command can report Codex feedback hook status in
  machine-readable JSON and human-readable text.
- The status command distinguishes "not installed", "installed but not wired",
  "wired but not locally verifiable because Codex approval is external", and
  "verified hook output succeeds".
- The verify command proves:
  - passing report emits `UserPromptSubmit` hook JSON and exits `0`,
  - failing report with default advisory mode exits `0`,
  - failing report with violation blocking exits `1`,
  - malformed hook arguments or invalid reports follow the documented error
    matrix.
- Install/merge behavior is tested against an existing `.codex/hooks.json` with
  unrelated hooks and must preserve them.
- Docs show both the current source-checkout path and the future published npm
  path without implying the package is already published.
- `.trellis/spec/assura/codex-agent-feedback.md` is updated if command
  signatures or hook contracts change.

## Validation Commands

Run and pass, or document exact blockers:

```bash
cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run
node --run verify:fast
node --run verify:docs
cargo run --quiet -- check --format json .
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
- status output cannot be consumed by an agent without screen-scraping prose,
- verify succeeds without checking real Codex hook JSON shape,
- docs imply the npm package is published before release evidence exists, or
- normal `assura check` workflows become dependent on Codex hook setup.

## Handoff Prompt

```text
/goal docs/goals/assura-codex-feedback-install-status-verify.md
```
