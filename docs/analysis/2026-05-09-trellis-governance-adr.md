---
id: adr-2026-05-09-trellis-governance
type: adr
title: Trellis-first governance for Assura
status: accepted
created: 2026-05-09
updated: 2026-05-09
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-05-09-documentation-cleanup-register.md
  - .trellis/workflow.md
  - .assura/config.yml
---

# Trellis-first governance for Assura

## Context

Assura had overlapping planning systems: Trellis instructions in `AGENTS.md`,
OpenSpec skills and prompts, and historical `specs-bak/` content. The project
also needs a workflow that agents can discover without asking the user where
current work, specs, and execution state live.

Trellis 0.6 beta is different from OpenSpec and Spec Kit for this project
because it covers both workflow orchestration and project knowledge. The beta
docs describe `.trellis/tasks/` for task execution, `.trellis/spec/` for
project specs, `.trellis/workspace/` for session memory, Codex hook support,
and a v0.6 roadmap that includes an auto-runner for trusted task chains.

Local verification on 2026-05-09 found:

- `trellis` was not globally installed.
- `npx -y @mindfoldhq/trellis@beta --version` returned `0.6.0-beta.5`.
- `npx -y @mindfoldhq/trellis@beta init -u nroth --codex --yes --skip-existing`
  completed successfully.
- Trellis warned that Codex hook injection requires user-level
  `features.hooks = true` and one-time `/hooks` approval.

## Decision

Use Trellis as Assura's canonical agent workflow and spec organization system.

- `.trellis/workflow.md` is the workflow source of truth.
- `.trellis/spec/` is the durable project spec library.
- `.trellis/tasks/` is the task and iteration execution ledger.
- `.trellis/workspace/` is the developer/session continuity layer.
- `.codex/` and `.agents/skills/trellis-*` are committed as generated platform
  support files.

OpenSpec and Spec Kit are not canonical for Assura right now. OpenSpec may be
kept temporarily as a historical/proposal reference, but it must not be treated
as a competing active workflow. Spec Kit is not adopted because its current
value is narrower than Trellis for this repo: it is primarily spec/change
planning, while Assura needs orchestration, task state, specs, and agent memory
in one discoverable system.

## Consequences

- Future agents should start from `AGENTS.md`, then use Trellis workflow/spec/task
  files for current project context.
- Assura self-validation should enforce `.trellis/` and `.codex/` structure once
  baseline structure checks are stable.
- `specs-bak/` and `openspec/` need explicit archive/delete treatment so agents
  do not treat them as active systems.
- Trellis beta automation is allowed for trusted repetitive chains, but manual
  `/trellis:continue` remains the controlled default until the auto-runner is
  proven stable in this repo.
