---
title: Agent Feedback MVP
description: Current agent feedback MVP and future agent-aware Assura feedback
---

Assura has a small agent feedback MVP under `integrations/agents/codex`.
It consumes `assura check --format json` output and turns structure violations
into advisory messages for a developer or agent.

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
```

## Optional Codex Hook Feedback

Use the source-checkout hook command when you want Codex to receive Assura
feedback during the native `UserPromptSubmit` hook event:

```bash
node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --min-severity medium --max-messages 5 --block-mode off
```

The Assura release installer installs the `assura` CLI only. Build the separate
agent-feedback package before adding the hook. The current proof path is a
source checkout build:

```bash
cd integrations/agents/codex
npm install
npm run build
node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path .
```

The command writes Codex hook JSON with
`hookSpecificOutput.additionalContext`. It reuses `--report <path>` when you
already have an Assura JSON report; otherwise it runs
`assura check --format json <path>`.

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
            "command": "node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --min-severity medium --max-messages 5 --block-mode off",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

Default hook behavior is advisory and exits `0`. Configure strict behavior with
`--block-mode violations|errors|all` and `--block-count <count>`.

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
`.agents/skills/` as the durable project
guidance surface.
