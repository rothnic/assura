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
- comparing measured runs across:
  - instructions-only workflows
  - `AGENTS.md`/skills workflows
  - Assura runtime feedbacks

Primary CLI example:

```bash
assura check --format advice .
assura check --format status .
```

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
status`, and `assura hooks verify`. The agent feedback MVP does not install Codex
hooks automatically, provide hosted telemetry, or implement complete autonomous
agent orchestration. Keep repo-local `.agents/skills/` as the durable project
guidance surface.
