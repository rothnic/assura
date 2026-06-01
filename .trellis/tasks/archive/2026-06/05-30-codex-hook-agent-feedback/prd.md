# Codex Hook Agent Feedback

## Superseded Direction

This task is historical. Its original package-hook proof was removed from the
current PR because package executable entrypoints are not the stable API. The
stable user-facing direction was corrected on 2026-05-31 and is now owned by
`.trellis/tasks/05-31-codex-feedback-install-status-verify` and
`.trellis/spec/assura/codex-agent-feedback.md`.

Do not use this task to reintroduce `assura-codex-feedback`, one CLI entrypoint
per agent, or per-agent `assura check --format <agent>-hook` values. The stable
surface is `assura check --format agent`; Codex delivery is
`assura check --format agent --agent codex`.

## Goal

Define and prove the native Codex hook installation path for Assura agent
feedback. The proof should show how Codex can receive Assura feedback through a
hook, while keeping ordinary developer workflows optional and unaffected.

## Requirements

- Add a small implementation proof for Codex hook feedback in
  `integrations/agents/codex`.
- Use the existing Assura nudge model as the feedback source rather than adding
  a separate validation model.
- Do not modify the repo's default `.codex/hooks.json` to install Assura
  feedback automatically.
- Clarify when the hook runs and what it does on pass, fail, and execution
  error.
- Clarify what check state the hook reuses and when it invokes
  `assura check --format json` itself.
- Clarify how feedback is injected into Codex through hook `additionalContext`.
- Clarify what can block: Assura validation failures, hook/runtime errors, or
  nothing in advisory mode.
- Add user-facing configuration for severity filtering, message count limits,
  and blocking behavior.
- Avoid claims about daemon support, editor integrations, hosted telemetry, or
  complete autonomous orchestration beyond what is implemented and tested.

## Acceptance Criteria

Historical acceptance criteria below are superseded by the current stable
surface in `.trellis/spec/assura/codex-agent-feedback.md`.

- Stable feedback is provided by `assura check --format agent`.
- Codex `UserPromptSubmit` delivery is provided by
  `assura check --format agent --agent codex`.
- Package CLI entrypoints are not part of the accepted design.
- Existing unsupported surfaces remain marked unsupported.

## Definition of Done

- `cd integrations/agents/codex && npm run lint && npm test && npm run build`
  passes.
- Repo-level formatting/tests/checks are run as appropriate for the touched
  Rust and docs surface.
- `cargo run --quiet -- check --format json .` passes or any blocker is
  documented exactly.
- Completion audit maps every explicit requirement to file or command evidence.

## Superseded Technical Approach

The package-hook CLI approach was superseded. The current supported approach is
to emit Codex hook JSON from the Rust CLI with:

```bash
assura check --format agent --agent codex . --warn
```

Default advisory/blocking behavior is controlled by normal `assura check`
options such as `--warn`, not by package-specific hook flags.

## Out of Scope

- Automatically editing `.codex/hooks.json` in this repo or user projects.
- Long-running daemon/watch mode.
- Editor-specific integrations.
- Hosted telemetry or remote orchestration.
- Changing the primary `StructureCheckReport` schema.

## Technical Notes

- Existing feedback implementation: `integrations/agents/codex/src/index.ts`.
- Package CLI files were removed from the accepted direction.
- Current tests: `integrations/agents/codex/src/agent-feedback-test.ts`.
- Current repo Codex hook config wires only Trellis:
  `.codex/hooks.json`.
- Current hook protocol example emits JSON with
  `hookSpecificOutput.additionalContext`:
  `.codex/hooks/inject-workflow-state.py`.

## Research References

- [`research/current-codex-nudge-surface.md`](research/current-codex-nudge-surface.md)
  — current repo constraints and the selected proof path.

## Completion Audit

This task is superseded. Current completion evidence belongs to
`.trellis/tasks/05-31-codex-feedback-install-status-verify` and PR #15.
