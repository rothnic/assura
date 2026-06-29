---
id: goal-assura-project-intelligence-mcp-agent-transport
type: goal
title: Assura project intelligence MCP agent transport
status: planned
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

# Assura Project Intelligence MCP Agent Transport

## Objective

Expose project-intelligence diagnostics, context packs, graph/search queries,
and safe-fix previews through a supported local agent-tool transport that
reuses the same contracts as the CLI.

## Current Gap

Agents can call `assura check`, `assura content agent-query`, and
`assura content context-pack`, but every agent integration still needs custom
shell wrappers and schema handling. A usable agent surface needs named local
tools over the same behavior without inventing per-agent command families or
per-agent output formats.

## Scope

- Select and implement the first supported local agent transport, with MCP as
  the default candidate unless a revalidation record identifies a better fit.
- Expose diagnostics, context-pack, content search/show/expand, missing
  relations, agent-context, and safe-fix preview operations.
- Reuse persistent-session state when available, while keeping one-shot CLI
  fallback behavior correct.
- Share schema structs and behavior with existing CLI contracts.
- Document tool names, arguments, response schemas, support level, and local
  startup path.

## Non-Goals

- No hosted MCP server.
- No remote provider requirement.
- No per-agent command family.
- No automatic repair.
- No editor protocol in this goal.

## Definition Of Done

- A local agent transport exposes the project-intelligence operations needed
  for a diagnostic-to-context handoff.
- Transport responses match the CLI schema families for representative
  diagnostics, context packs, graph/search queries, and safe-fix previews.
- Tests prove the transport and CLI agree on Assura-local and Beacon CRM
  examples.
- Docs tell agents when to use CLI, agent envelopes, context packs, or the
  agent transport.
- Support policy accurately classifies the transport as supported,
  experimental, or roadmap-only based on what is implemented.

## Validation Commands

```bash
cargo fmt --check
cargo test mcp --quiet
cargo test agent_surface --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo run --quiet -- content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --format json
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm transport tools call shared project-intelligence contracts rather
  than reimplementing validation or query behavior.
- R2: Confirm tool output can be correlated with CLI evidence by schema,
  diagnostic ID, file path, collection, and object ID.
- R3: Confirm the transport is local and does not require hosted
  infrastructure.
- R4: Confirm no per-agent command or output format is introduced.

## Reviewer Blocking Criteria

Block if the transport forks validation/query logic, requires a hosted service,
applies fixes implicitly, hides lower-level evidence, or creates Codex-specific
behavior outside the shared agent contract.
