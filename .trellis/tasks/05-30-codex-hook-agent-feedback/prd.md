# Codex Hook Agent Feedback

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

- A tested API renders Codex `UserPromptSubmit` hook JSON for Assura feedback.
- A CLI entrypoint can run as the hook command and supports report reuse,
  severity filtering, message count limits, and blocking mode.
- Tests prove advisory mode does not block normal workflows by default.
- Tests prove opt-in blocking can return a nonzero code when configured.
- Documentation gives an explicit optional installation snippet and states that
  existing hooks must be merged rather than overwritten.
- Documentation explains hook timing, reused check state, injection mechanism,
  blocking behavior, and configuration knobs.
- Existing unsupported surfaces remain marked unsupported.

## Definition of Done

- `cd integrations/agents/codex && npm run lint && npm test && npm run build`
  passes.
- Repo-level formatting/tests/checks are run as appropriate for the touched
  Rust and docs surface.
- `cargo run --quiet -- check --format json .` passes or any blocker is
  documented exactly.
- Completion audit maps every explicit requirement to file or command evidence.

## Technical Approach

Add a Codex-hook-specific renderer and CLI in the existing
`@assura/codex-integration` package. The hook command will emit Codex hook JSON
with `hookSpecificOutput.hookEventName = "UserPromptSubmit"` and
`additionalContext` containing a concise Assura feedback block. It will reuse an
existing Assura report when `--report` is supplied; otherwise it will run
`assura check --format json` through the existing `runAssuraCheck` path.

Default behavior is advisory: feedback is injected but the hook exits `0`.
Blocking is opt-in through a CLI option and only blocks when configured
thresholds match the report. Runtime/hook errors stay non-blocking unless the
user explicitly chooses strict error blocking.

## Decision (ADR-lite)

**Context**: The prior Assura agent nudge MVP intentionally did not install
Codex hooks. This task must define a native Codex hook path without changing
normal developer defaults or claiming unsupported automation.

**Decision**: Implement an optional `assura-codex-hook` entrypoint in the Codex
integration package. Document installation as an additive `UserPromptSubmit`
hook command that users merge into their Codex hook config.

**Consequences**: The proof is small and testable, and existing workflows remain
unchanged. Users who want always-on feedback must explicitly wire the hook.
Daemon/editor support remains future work.

## Out of Scope

- Automatically editing `.codex/hooks.json` in this repo or user projects.
- Long-running daemon/watch mode.
- Editor-specific integrations.
- Hosted telemetry or remote orchestration.
- Changing the primary `StructureCheckReport` schema.

## Technical Notes

- Existing nudge implementation: `integrations/agents/codex/src/index.ts`.
- Existing CLI: `integrations/agents/codex/src/cli.ts`.
- Existing tests: `integrations/agents/codex/src/nudge-test.ts`.
- Current repo Codex hook config wires only Trellis:
  `.codex/hooks.json`.
- Current hook protocol example emits JSON with
  `hookSpecificOutput.additionalContext`:
  `.codex/hooks/inject-workflow-state.py`.

## Research References

- [`research/current-codex-nudge-surface.md`](research/current-codex-nudge-surface.md)
  — current repo constraints and the selected proof path.

## Completion Audit

| Requirement | Evidence |
| --- | --- |
| Tested API renders Codex hook JSON | `integrations/agents/codex/src/hook.ts` exposes `renderCodexHookFeedback` and `renderCodexHookOutput`; tests assert `hookSpecificOutput.hookEventName = "UserPromptSubmit"` and `additionalContext` includes `<assura-feedback>`. |
| CLI can run as hook command | `integrations/agents/codex/src/hook-cli.ts` exposes `runHookCli`; `package.json` adds the `assura-codex-hook` binary and package files. |
| Report reuse and direct check state are explicit | `--report` path calls `parseStructureCheckReport`; direct mode calls `runAssuraCheck`; README documents both states. |
| Severity/count/blocking configuration exists | CLI supports `--min-severity`, `--max-messages`, `--block-mode`, and `--block-count`; tests cover filtering, limits, advisory default, violation blocking, and error blocking. |
| Normal developer workflows remain optional/unaffected | `.codex/hooks.json` is unchanged; docs say users append the hook command and the package does not edit hook config. Default hook mode exits `0`. |
| Unsupported surfaces are not claimed | README and website docs state automatic hook mutation/installation, daemon/editor reuse, hosted telemetry, and complete orchestration are not implemented. |
| End-user review findings addressed | Independent read-only Codex review found five user-facing gaps: package install/PATH, Codex hook enablement, report-error source attribution, malformed-argument advisory behavior, and thin hook help. Docs, CLI behavior, and regression tests now cover those cases. |
| Durable code-spec captured | `.trellis/spec/assura/codex-agent-feedback.md` records command signatures, hook JSON contracts, install prerequisites, error matrix, and required tests. |
| Validation passed | `cd integrations/agents/codex && npm run lint && npm test && npm run build`; `npm pack --dry-run`; `git diff --check`; `node --run verify:fast`; `node --run verify:docs`. |
