---
id: goal-assura-compact-project-review-common-issues
type: goal
title: Assura compact project review and common issues
status: planned
created: 2026-07-04
owners:
  - assura-maintainers
related:
  - ./assura-performance-polish-program.md
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-doctor-explain-feedback.md
  - ./assura-llm-wiki-personal-knowledge-base-starters.md
  - ../analysis/2026-07-04-backlog-priority-agent-harness-and-okf-assessment.md
---

# Assura Compact Project Review And Common Issues

## Objective

Give humans and agents one compact first diagnostic flow for the common
question: "Is this repo healthy, and what should I fix or configure next?"

The flow should combine current Assura truth from structure checks, doctor
state, onboarding readiness, next actions, and high-signal content gaps without
dumping noisy reference findings or forcing users to know every lower-level
command first.

## Current Gap

Assura has useful primitives: `assura check`, `assura doctor`, self-check
rules, agent nudges, content agent-query gaps, daemon status, and onboarding
packets. The common user path is still fragmented. A new user or coding agent
can pass one command and still miss that optional references, generated docs,
skills, model files, or onboarding artifacts are incomplete, duplicated, or
inconsistent with the repository's existing structure.

## Scope

- Design a compact review command, recipe, or documented flow over existing
  surfaces before adding new core behavior.
- Prioritize issues by user action: fix now, configure intentionally, inspect
  before changing, or ignore/generated/archive noise.
- Include structure-fit guidance for new files/directories: check the nearby
  project shape, avoid duplication, and only change `.assura/config.yml` when
  the new path is intentionally part of the repository contract.
- Keep output bounded for agent use, with stable JSON fields and concise text.
- Filter or classify low-value reference noise from generated artifacts,
  archives, benchmark history, and logs before surfacing content-gap counts.
- Link from the review output to lower-level commands for detailed evidence.
- Document the first-run workflow and the before-PR review workflow.

## Non-Goals

- No automatic config rewrites.
- No broad new search engine.
- No replacement for `assura check`, `assura doctor`, or content-query
  commands.
- No blocking policy based only on noisy unresolved-reference counts.
- No hosted service, MCP server, or editor-specific requirement.

## Definition Of Done

- One documented compact review path exists for humans and local agents.
- The path reports structure check status, doctor/onboarding status, config
  fit warnings, content/reference gap summary, next recommended commands, and
  whether findings are blocking, advisory, inactive, or informational.
- JSON output is stable enough for agent wrappers and avoids scraping text.
- Text output stays concise enough to paste into an agent turn without
  crowding out the task.
- The flow distinguishes "fix this file" from "decide whether this path should
  exist in the project contract."
- Generated/archive/log/benchmark reference noise is filtered, classified, or
  explicitly reported as omitted.
- Docs show how to use the compact review before creating a new top-level
  directory, before opening a PR, and when onboarding an existing repository.
- Tests cover a clean repo, a structure mismatch, duplicated/unmodeled path
  pressure, inactive optional guidance, noisy reference gaps, and a genuinely
  actionable content gap.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
git diff --check
```

If the implementation is docs-only for the first slice, keep validation scoped
to workflow gate, Assura self-check, docs/evidence/target-state, and
`git diff --check`.

## Review Tasks

- R1: Confirm the compact review uses existing Assura truth surfaces instead
  of inventing a parallel validator.
- R2: Confirm agent JSON is bounded, stable, and points to detailed evidence
  only when needed.
- R3: Confirm structure-fit guidance nudges agents to inspect the existing
  project shape before adding directories or changing config.
- R4: Confirm noisy reference findings are not promoted into blocking guidance
  without filtering or classification.
- R5: Confirm docs make the common first diagnostic path obvious without hiding
  lower-level commands from advanced users.

## Reviewer Blocking Criteria

Block if the implementation changes `.assura/config.yml` automatically, treats
generated/archive/log reference noise as a hard failure, duplicates validation
logic outside existing surfaces, emits unbounded context for agents, or leaves
users without a clear distinction between "fix the repo" and "decide whether
this path belongs in the repo contract."

## Copy/Paste Goal Prompt

```text
Execute docs/goals/assura-compact-project-review-common-issues.md.

Start from live state: run the Trellis workflow gate, inspect git status,
review the current roadmap, and run assura check/doctor/content-gap commands
needed to understand the existing first diagnostic experience.

Design and implement the smallest reviewed slice that gives humans and agents
one compact project review path. It must combine structure check status,
doctor/onboarding readiness, next actions, structure-fit guidance for new
files/directories, and high-signal content/reference gaps without surfacing
noisy generated/archive/log findings as blockers.

Do not auto-edit .assura/config.yml or build a new search engine. Reuse
existing Assura truth surfaces, add tests for clean, mismatch, duplicate-path,
inactive-guidance, noisy-gap, and actionable-gap cases, update docs, run the
scoped validation gates, and record reviewer findings before closure.
```
