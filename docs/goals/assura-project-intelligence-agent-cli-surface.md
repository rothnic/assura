---
id: goal-assura-project-intelligence-agent-cli-surface
type: goal
title: Assura project intelligence agent CLI surface
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - .trellis/spec/assura/codex-agent-feedback.md
  - website/src/content/docs/product/agent-editor-surfaces.md
---

# Assura Project Intelligence Agent CLI Surface

## Post-Merge Revalidation

Completed by the Project Intelligence usability program. Current evidence:
`docs/analysis/2026-06-29-project-intelligence-usability-final-audit.md`
classifies `assura agent ...` as a supported local surface, and
`tests/agent_surface_cli.rs` covers diagnostics, context packs, search/show,
expand, missing-relations, safe-fix preview, and session behavior over shared
content-query contracts. No separate per-agent command family or MCP
dependency is needed for this completed scope.

## Objective

Expose project-intelligence diagnostics, context packs, graph/search queries,
and safe-fix previews through a supported local CLI surface that coding agents
can call directly.

## Current Gap

Agents can call `assura check`, `assura content agent-query`, and
`assura content context-pack`, but the agent workflow is scattered across
human-oriented command groups. A usable agent surface needs a small set of
named local commands over the same behavior without inventing per-agent command
families, per-agent output formats, a daemon, or a remote protocol dependency.

## Scope

- Implement the first supported local agent command group for Project
  Intelligence, with the CLI as the primary product contract.
- Expose diagnostics, context-pack, content search/show/expand, missing
  relations, agent-context, and safe-fix preview operations.
- Reuse persistent-session state when available, while keeping one-shot CLI
  fallback behavior correct.
- Share schema structs and behavior with existing CLI contracts.
- Document command names, arguments, response schemas, support level, and local
  agent usage examples.
- Record MCP stdio as a possible future adapter over the same CLI/library
  contracts only after the CLI surface is stable.

## Non-Goals

- No MCP implementation in this goal.
- No hosted MCP server or remote transport.
- No remote provider requirement.
- No per-agent command family or per-agent output format.
- No daemon requirement.
- No automatic repair.
- No editor protocol in this goal.

## Definition Of Done

- A local agent CLI surface exposes the project-intelligence operations needed
  for a diagnostic-to-context handoff.
- Agent CLI responses match the existing schema families for representative
  diagnostics, context packs, graph/search queries, and safe-fix previews.
- Tests prove the agent CLI and existing CLI agree on Assura-local and Beacon CRM
  examples.
- Docs tell agents when to use `assura agent`, `assura check --format agent`,
  content context packs, and safe-fix previews.
- Support policy accurately classifies the agent CLI as supported,
  experimental, or roadmap-only based on what is implemented, and keeps MCP
  roadmap-only unless separately implemented.

## Validation Commands

```bash
cargo fmt --check
cargo test -p assura --test agent_surface_cli --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo run --quiet -- content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --format json
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm agent CLI commands call shared project-intelligence contracts rather
  than reimplementing validation or query behavior.
- R2: Confirm command output can be correlated with existing CLI evidence by schema,
  diagnostic ID, file path, collection, and object ID.
- R3: Confirm the surface is local and does not require hosted
  infrastructure.
- R4: Confirm no per-agent command or output format is introduced.
- R5: Confirm MCP remains an optional future adapter, not a prerequisite for
  local agent usability.

## Reviewer Blocking Criteria

Block if the agent CLI forks validation/query logic, requires a hosted service,
applies fixes implicitly, hides lower-level evidence, makes MCP required for
local use, or creates Codex-specific behavior outside the shared agent
contract.
