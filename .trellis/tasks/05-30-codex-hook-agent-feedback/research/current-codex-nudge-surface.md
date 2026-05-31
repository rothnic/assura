# Current Codex Nudge Surface

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

## Selected proof path

Add an optional `assura-codex-hook` command to the existing Codex package. The
command renders native Codex hook JSON for `UserPromptSubmit`. It can reuse a
precomputed report via `--report`, or it can run `assura check --format json`
for the configured path. It injects feedback through `additionalContext`.

The default is advisory exit `0`. Users can opt into blocking based on violation
severity, violation count, or runtime hook errors through explicit CLI options.
