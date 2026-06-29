# Project Intelligence Agent Surfaces

## Goal

Execute `docs/goals/assura-project-intelligence-agent-surfaces.md` as the
ninth successor in the Project Intelligence Runtime program. The task must
expose agent/editor-facing contracts over the existing local validation and
project-intelligence query core without creating per-agent command families or
requiring hosted infrastructure.

## Revalidation Result

`valid`: live repo state after the completed code-symbol successor has local
project-intelligence facts, collection/query/search/semantic/code-symbol CLI
commands, Markdown safe-fix support, and the stable
`assura check --format agent --agent codex` delivery contract. The goal is not
complete because there is no shared project-intelligence agent surface contract
that lets agents request diagnostics, safe fixes, graph/query/search results,
and code-symbol relationships through one documented API boundary. Existing
docs still mark daemon/editor, LSP, and MCP surfaces as planned.

## Requirements

- Preserve `assura check --format agent --agent codex` as the stable Codex
  delivery path; do not revive per-agent commands or formats.
- Define one shared core API/contract for diagnostics, safe fixes,
  graph expansion, keyword search, semantic candidates, and code-symbol
  relationships.
- Keep CLI behavior as the first public surface and use the shared contract
  rather than duplicating query logic.
- Add agent-facing JSON contracts and tests that can support future daemon,
  LSP, and MCP wrappers.
- Treat daemon/persistent mode as optional until benchmark or reuse evidence
  justifies it.
- Document which surface is for human CLI use, agent automation, editor
  diagnostics, and long-running local workflows.

## Acceptance Criteria

- [ ] A shared agent/query contract exists for diagnostics, safe fixes, graph
  queries, search, semantic candidates, and code-symbol relationships.
- [ ] CLI-facing agent/query output reuses that contract instead of introducing
  a parallel per-agent command family.
- [ ] Contract tests cover deterministic JSON shape, safe-fix dry-run/write
  boundaries, and project-intelligence query cases.
- [ ] Docs explain the supported CLI/agent path and the future daemon/LSP/MCP
  wrapper boundary without promising unsupported infrastructure.
- [ ] Existing `assura check --format agent --agent codex` behavior remains
  valid and tested.
- [ ] Validation commands pass or any blocker is documented with exact output.

## Out Of Scope

- Hosted services, telemetry, or dashboards.
- Plugin marketplaces or remote plugin execution.
- Required daemon, LSP, MCP, or editor integration for normal validation.
- Per-agent CLI entrypoints or per-agent `--format` values.
- Persistent-session implementation unless measured index reuse proves value
  within this task.

## Technical Notes

- Active goal: `docs/goals/assura-project-intelligence-agent-surfaces.md`.
- Master program:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Direction-lock spec: `.trellis/spec/assura/codex-agent-feedback.md`.
- Existing agent output: `src/cli/agent_feedback.rs`,
  `src/cli/agent_output.rs`, `src/cli/codex_output.rs`.
- Existing query core: `src/cli/content_query/` and `src/intelligence/`.
- Existing safe-fix path: `assura fix markdown` and Markdown safe-fix tests.

## Review Tasks

- R1: Confirm the implementation shares core logic across CLI and future
  agent/editor wrappers.
- R2: Confirm agent contracts are stable and documented.
- R3: Confirm no per-agent command family or provider-specific behavior is
  introduced.
- R4: Confirm persistent mode is deferred unless measured reuse evidence is
  added.
- R5: Confirm safe-fix boundaries are explicit and deterministic.

## Progress Evidence

- 2026-06-28: Revalidated as `valid` after completing and archiving code-symbol
  enrichment. Current live state has project-intelligence content/query/search,
  semantic, code-symbol, diagnostics, safe-fix, and agent feedback foundations,
  but no shared project-intelligence agent surface contract for agents/editors
  to request those capabilities through one documented API.
- 2026-06-28: Added first shared project-intelligence agent context slice:
  `assura content agent-context` emits
  `assura.project-intelligence.agent-context.v1` from reusable
  `intelligence::agent_surface` contract structs. The summary covers current
  model instances, diagnostics, safe fixes, graph relationship edges, search
  chunks, semantic embedding records, and code-symbol refs. This is a generic
  wrapper contract, not a per-agent command family. Validation passed:
  `cargo test --test content_query_cli --quiet`,
  `cargo run --quiet -- check --format json .`, `cargo fmt --check`,
  `git diff --check`, `cargo xtask docs`, `cargo xtask evidence`, and
  `cargo check --workspace --all-targets --quiet`.
- 2026-06-28: Added safe-fix dry-run/write contract slice. `assura fix markdown`
  now supports `--dry-run` and `--format json|yaml|text`; JSON reports use
  `assura.safe-fix.markdown.v1`, preview proposed file/line fixes without
  writing, and keep normal write summaries for applied fixes. The exact
  safe-fix dry-run surface is registered in the public support matrix.
  Validation passed: `cargo test --test markdown_lint_fix_tests --quiet`,
  `cargo test --test content_query_cli --quiet`, `cargo fmt --check`,
  `git diff --check`, `cargo run --quiet -- check --format json .`,
  `cargo run --quiet -- check --format agent --agent codex .`,
  `cargo check --workspace --all-targets --quiet`, `cargo xtask docs`, and
  `cargo xtask evidence`.
