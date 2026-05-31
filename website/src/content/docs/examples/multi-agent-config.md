---
title: Agent Feedback MVP
description: Current agent feedback MVP and future agent-aware Assura feedback
---

Assura exposes agent feedback through the stable `assura check` command. The
lower-level Codex package under `integrations/agents/codex` remains available
for wrappers that already have JSON reports, but the primary user API is not a
separate feedback binary.

The supported validation command remains:

```bash
assura check --format json .
```

## Current Feedback MVP

The MVP supports:

- parsing Assura `StructureCheckReport` JSON
- creating actionable feedback messages with path, rule, severity, guidance, and
  repo-local references
- preserving Assura's nonzero exit behavior when validation fails
- optionally emitting native Codex `UserPromptSubmit` hook JSON that injects
  Assura feedback through `additionalContext`
- comparing measured runs across:
  - instructions-only workflows
  - `AGENTS.md`/skills workflows
  - Assura runtime feedbacks

Primary CLI example:

```bash
assura check --format advice .
assura check --format status .
assura check --format agent . --warn --min-severity medium --max-issues 5
```

## Optional Codex Hook Feedback

Use `assura check --format agent` when a wrapper wants stable Assura feedback
JSON. Add `--agent codex` only when Codex should receive Assura feedback during
the native `UserPromptSubmit` hook event:

```bash
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

The command writes Codex hook JSON with
`hookSpecificOutput.additionalContext`. Use `--warn` for advisory feedback that
exits `0`; omit `--warn` when the hook should block on validation failures.

Add it to `.codex/hooks.json` only if you want this per-prompt feedback. Codex
must have hooks enabled in user config with `features.hooks = true`, and the
project hook command must be approved once with `/hooks`. If the project already
has Codex hooks, append the command instead of replacing the existing
`UserPromptSubmit` list.

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

Default `assura check` behavior blocks on validation failures. Add `--warn` for
advisory Codex feedback.

## Metrics

The MVP measurement model tracks:

- modularity improvement observations
- instruction adherence
- structural violations
- correction loops
- feedback precision
- useful feedback
- noisy feedback
- missed violations
- same-turn observations by violation class, including feedback count, whether the
  class was fixed before a new turn, usefulness, and remaining violations

## Feedback Shape

Assura feedback tells the developer or agent:

- what structure rule failed
- what path failed
- what corrective action is likely
- whether the feedback is advisory
- which skill, script, or document should be loaded before fixing it

## Still Future-Only

Assura can install local Git hooks with `assura hooks install`, `assura hooks
status`, and `assura hooks verify`. The agent feedback MVP does not install
Codex hooks automatically, provide hosted telemetry, reuse a daemon/editor
session, or implement complete autonomous agent orchestration. Keep repo-local
`.agents/skills/` as the durable project guidance surface.
