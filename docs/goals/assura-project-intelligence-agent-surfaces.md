---
id: goal-assura-project-intelligence-agent-surfaces
type: goal
title: Assura project intelligence agent surfaces
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-content-query-and-search-cli.md
  - docs/goals/assura-local-semantic-search.md
  - docs/goals/assura-code-symbol-enrichment.md
  - .trellis/spec/assura/codex-agent-feedback.md
---

# Assura Project Intelligence Agent Surfaces

## Objective

Expose the same local validation, query, search, and safe-fix capabilities
through agent-friendly surfaces without creating divergent command families or
provider-specific behavior.

## Current Gap

Assura has stable `assura check --format agent` output, but project
intelligence capabilities will need longer-lived and richer surfaces for
agents, editors, and local workflows: daemon API, CLI, LSP, and MCP. These
surfaces are not yet defined around one shared core.

## Scope

- Keep CLI as the first public surface for validation and query behavior.
- Define a shared internal API for diagnostics, safe fixes, graph queries,
  keyword search, semantic search, and code-symbol relationships.
- Add daemon or persistent-session mode only after the query/search core
  benefits from reused indexes.
- Add LSP diagnostics and commands for editor workflows.
- Add MCP tools for agents only as wrappers over the same core contracts.
- Preserve the stable `assura check --format agent --agent codex` direction.
- Avoid one command family per agent.

## Non-Goals

- No hosted SaaS requirement.
- No plugin marketplace.
- No remote provider requirement for core features.
- No per-agent CLI entrypoints or per-agent `--format` values.
- No daemon until the index/search reuse benefit is demonstrated.

## Definition Of Done

- CLI, LSP, MCP, and any accepted daemon or persistent-session surface share
  one core query and validation API.
- Agent JSON contracts are documented and covered by tests.
- Safe-fix operations include dry-run output and bounded write behavior.
- Persistent mode reuses indexes only when benchmarks show value.
- Existing `assura check --format agent --agent codex` remains the stable
  Codex delivery path.
- Docs explain which surface to use for human CLI, agent automation, editor
  diagnostics, and long-running local workflows.

## Validation Commands

```bash
cargo fmt --check
cargo test agent_surface --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R1: Confirm surfaces share core logic instead of duplicating query behavior.
- R2: Confirm agent contracts are stable and documented.
- R3: Confirm no per-agent command family is introduced.
- R4: Confirm persistent mode is justified by measured index reuse.

## Reviewer Blocking Criteria

Block if implementation forks CLI/LSP/MCP behavior, revives per-agent command
families, requires hosted infrastructure, or adds persistent mode without a
measured reason.

## Progress Log

- 2026-06-28: Revalidated as `valid` after completing and archiving code-symbol
  enrichment. Live repo now has content/query/search, local semantic search,
  code-symbol enrichment, diagnostics, safe-fix, and stable
  `assura check --format agent --agent codex` foundations. The remaining gap
  is a shared project-intelligence agent surface contract for diagnostics,
  safe fixes, graph/search/semantic/code-symbol queries, and future daemon/LSP
  or MCP wrappers without reviving per-agent command families.
