---
title: Agent Nudge Roadmap
description: Future direction for agent-aware Assura feedback
---

Assura's current release does not include completed runtime agent nudges,
agent profiles, or maturity-specific CLI flags. The supported command remains:

```bash
assura check
```

## Future Direction

The next goal is a Codex/agent nudge MVP that compares:

- instructions-only workflows
- `AGENTS.md` plus repo-local skills
- Assura runtime nudges surfaced during agent work

## Metrics

The nudge MVP should measure:

- modularity improvements
- instruction adherence
- structural violations
- correction loops
- nudge precision

## Feedback Shape

Future Assura failures should tell the developer or agent:

- what structure rule failed
- why the rule exists
- whether the hook is warning or blocking
- which skill, script, or document should be loaded before fixing it
- who can approve a policy change

Until that work is implemented, use `.assura/config.yml`, `AGENTS.md`, and
repo-local `.agents/skills/` as the durable project guidance surface.
