---
name: assura-goal-execution
description: Use when executing long-running Assura goals from docs/goals, especially when the work spans several iterations, context may drift, or repo-local .agents/skills may need to be created or updated for progressive disclosure.
---

# Assura Goal Execution

Use this skill when working from a long-running goal in `docs/goals/`.

## Start

1. Read the target goal file.
2. Read `AGENTS.md` and the Trellis workflow/spec files named by the goal.
3. Record a progress-log entry in the goal before and after major phases.
4. If local build or network failures occur, load
   `.agents/skills/assura-local-build/SKILL.md` before changing product code.

## Iteration Review Hook

Treat one iteration as a meaningful implementation/review loop: planning a
slice, editing files, running validation, and deciding the next slice.

Every third iteration, and before any final handoff:

1. Record the current iteration count in the goal progress log.
2. Record available context-health information. If the platform exposes token
   or context budget, include it; otherwise write `context level: not exposed`
   and summarize the relevant prior messages in 3-6 bullets.
3. Review the current conversation, progress log, failed commands, and repeated
   explanations.
4. Decide whether a reusable project skill should be created or updated under
   `.agents/skills/`.
5. If a skill is created or updated, keep `AGENTS.md` as a lean router: add only
   the skill name, trigger, and one-line purpose. Put operational detail inside
   the skill.
6. Re-run `assura check` after changing `.agents/skills/`, `AGENTS.md`, or
   `.assura/config.yml`.

## Skill Creation Bar

Create or update a skill when any of these are true:

- The agent rediscovered the same repo-specific workflow twice.
- A platform/build workaround is needed for repeatable validation.
- A validation failure needs reusable remediation steps.
- The goal introduces a recurring implementation pattern.
- The agent needed more than one paragraph in `AGENTS.md` to explain a
  procedure.

Do not create a skill for one-off facts, short status notes, or details that
belong only in the current goal's progress log.
