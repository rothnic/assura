# Current Codex Nudge Surface

## Supersession Note

This research note records the 2026-05-30 proof path for a lower-level package
hook. It is superseded for public CLI/API direction by
`.trellis/spec/assura/codex-agent-feedback.md` and
`.trellis/tasks/05-31-codex-feedback-install-status-verify`.

Do not use the selected proof path below to revive `assura-codex-feedback`, one
CLI entrypoint per agent, or per-agent `assura check --format <agent>-hook`
values. Stable user-facing feedback is `assura check --format agent`; Codex is
only a delivery adapter via `--agent codex`.

## Existing repo facts

- `integrations/agents/codex` is the canonical Codex integration package.
- The package already parses `assura check --format json`, creates advisory
  nudge messages, renders text/JSON, and preserves Assura exit codes when run
  directly.
- `docs/goals/assura-agent-nudge-mvp.md` explicitly kept automatic Codex hook
  installation out of scope.
- `.codex/hooks.json` currently registers a `UserPromptSubmit` command hook for
  Trellis workflow-state injection only.
- `.codex/hooks/inject-workflow-state.py` proves the local native Codex hook
  output shape used in this repo: JSON on stdout with
  `hookSpecificOutput.hookEventName` and `hookSpecificOutput.additionalContext`.

## Constraints

- The proof must not overwrite existing `UserPromptSubmit` hooks.
- Normal developer CLI use must remain opt-in and unaffected.
- Feedback should use the existing Assura nudge model instead of creating a
  second result schema.
- Documentation must not imply daemon, editor, hosted telemetry, or complete
  orchestration support.

## Superseded Proof Path

The package command proof path was superseded. Use the current spec and task
instead:

```bash
assura check --format agent --agent codex . --warn
```

Feedback is still injected through Codex `UserPromptSubmit` additional context,
but the command surface is the Rust `assura check` CLI.
