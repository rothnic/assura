# Project Intelligence Agent CLI Surface

## Goal

Make Project Intelligence usable by local coding agents through a stable,
scriptable Assura CLI surface. The interface should expose the same practical
capabilities originally targeted for a transport layer, but the primary
contract is local CLI commands that are easy for agents and humans to inspect,
test, and compose.

## What I Already Know

- The active program goal is
  `docs/goals/assura-project-intelligence-usability-program.md`.
- The roadmap previously named
  `docs/goals/assura-project-intelligence-agent-cli-surface.md` as the next
  successor after context packs, persistent sessions, safe-fix workflow, and
  `.assura/` directory organization.
- User direction on 2026-06-29: prefer CLI over MCP; MCP appears heavier than
  needed, and remote access is not a requirement.
- Existing Project Intelligence commands already expose diagnostics,
  context-pack, content search/show/expand, missing relations, agent context,
  and safe-fix preview behavior.
- This task should reuse those contracts rather than duplicating behavior behind
  a separate agent protocol implementation.

## Requirements

- Add a stable local `assura agent ...` CLI surface, or equivalent existing CLI
  grouping if inspection shows a better repo-native pattern.
- Cover the agent workflows that matter for usability:
  diagnostics/status, context pack retrieval, content search/show/expand,
  missing relation inspection, agent context, and safe-fix preview.
- Keep outputs machine-readable and compatible with existing Project
  Intelligence JSON contracts where practical.
- Preserve one-shot CLI behavior; no daemon or remote service is required.
- Document the commands in the docs site with examples showing how agents and
  users can apply the capability.
- Present MCP as an optional future adapter over the same CLI/library contracts,
  not as the source of truth for this slice.

## Acceptance Criteria

- [x] `assura agent --help` exposes the local agent command group.
- [x] Representative `assura agent ... --format json` commands return stable
      JSON for diagnostics/context, search/show/expand, missing relations, and
      safe-fix preview.
- [x] CLI tests prove the agent commands reuse existing Project Intelligence
      behavior rather than drifting into parallel output semantics.
- [x] Docs site examples include a visual, end-to-end demo of using the agent
      CLI to discover context and evaluate a safe fix.
- [x] Roadmap/goal docs no longer imply remote MCP access is required for the
      next usability step.
- [x] Validation commands from the program goal and this task pass, or any
      intentionally deferred gates are documented with evidence.

## Definition of Done

- Tests added or updated for the new agent CLI surface.
- Existing content-query and safe-fix tests remain green.
- Documentation site pages include concrete examples and a visual demo path.
- Assura self-check remains clean.
- Complex-task review agent has reviewed the implementation before final handoff.

## Technical Approach

Use the existing content-query and safe-fix command implementations as the
contract source. Add a thin local agent command grouping that delegates to those
paths and standardizes command naming for coding-agent usage. Favor small
wrapper functions or shared helpers over any independent transport runtime.

## Decision (ADR-lite)

**Context**: The original successor goal named MCP as the default candidate
transport, but the product need is local agent usability, not remote tool
hosting.

**Decision**: Build CLI-first. Treat MCP stdio as a future optional adapter only
after the CLI contract is stable. Remote MCP/HTTP access is out of scope.

**Consequences**: The initial implementation stays lighter, easier to validate,
and aligned with Assura's existing CLI-first agent feedback contract. Future MCP
support can wrap these same commands or shared functions if a host integration
creates enough value.

## Out of Scope

- Remote MCP, Streamable HTTP, hosted service mode, authentication, or network
  access.
- A long-running daemon or background project-intelligence server.
- A separate per-agent binary or one command family per agent host.
- Changing the existing `assura check --format agent` feedback contract.

## Technical Notes

- Relevant specs: `.trellis/spec/assura/roadmap.md`,
  `.trellis/spec/assura/codex-agent-feedback.md`, and shared Trellis thinking
  guides.
- Relevant goals:
  `docs/goals/assura-project-intelligence-usability-program.md` and
  `docs/goals/assura-project-intelligence-agent-cli-surface.md`.
- Inspect current CLI contracts under `src/cli/` before implementing to avoid
  duplicate behavior.
- Validation evidence collected on 2026-06-29:
  `cargo fmt --check`,
  `cargo test -p assura --test agent_surface_cli --quiet`,
  `cargo test --test project_intelligence_context_pack --quiet`,
  `cargo test -p assura --test cli_command_surface_tests --quiet`,
  `cargo run --quiet -- content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --format json`,
  `cargo run --quiet -- check --format json .`,
  `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check`.
