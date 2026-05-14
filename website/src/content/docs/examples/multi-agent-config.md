---
title: Agent Nudge MVP
description: Current Codex nudge MVP and future agent-aware Assura feedback
---

Assura has a small Codex/agent nudge MVP under `integrations/agents/codex`.
It consumes `assura check --format json` output and turns structure violations
into advisory messages for a developer or agent.

The supported validation command remains:

```bash
assura check --format json .
```

## Current Nudge MVP

The MVP supports:

- parsing Assura `StructureCheckReport` JSON
- creating actionable nudge messages with path, rule, severity, guidance, and
  repo-local references
- preserving Assura's nonzero exit behavior when validation fails
- comparing measured runs across:
  - instructions-only workflows
  - `AGENTS.md`/skills workflows
  - Assura runtime nudges

Example:

```bash
assura check --format json . > assura-report.json
assura-codex-nudge --report assura-report.json --format text
```

## Metrics

The MVP measurement model tracks:

- modularity improvement observations
- instruction adherence
- structural violations
- correction loops
- nudge precision
- useful nudges
- noisy nudges
- missed violations

## Feedback Shape

Assura nudges tell the developer or agent:

- what structure rule failed
- what path failed
- what corrective action is likely
- whether the nudge is advisory
- which skill, script, or document should be loaded before fixing it

## Still Future-Only

The MVP does not install Codex hooks automatically, provide hosted telemetry,
or implement complete autonomous agent orchestration. Keep repo-local
`.agents/skills/` as the durable project guidance surface.
