# Project Intelligence Context Pack

## Objective

Implement `docs/goals/assura-project-intelligence-context-pack.md` as the next
Project Intelligence Usability successor.

## Requirements

- Add one bounded context-pack command or public operation.
- Compose existing validation, content repository, search, graph, relation,
  agent-query, and safe-fix preview contracts.
- Support diagnostic-oriented and object-oriented requests.
- Prove the output for Beacon CRM invalid and the Assura goal model.
- Document a complete agent editing handoff using the context pack.
- Keep the path local-only, read-only, and transport-agnostic.

## Acceptance Criteria

- [x] Context pack has a versioned schema.
- [x] Diagnostics, modeled records, Markdown sections, relation status, and
  safe-fix preview metadata are present or explicitly omitted with reasons.
- [x] Output is bounded and reports truncation or omission metadata.
- [x] Tests prove agreement with lower-level CLI behavior.
- [x] Website docs show when to use context pack versus lower-level commands.
- [x] Goal log and roadmap are updated.
- [x] Validation commands pass.

## Non-Goals

- No writes.
- No semantic ranking as validation truth.
- No persistent session or transport implementation.
- No safe-fix apply behavior.

## Validation Evidence

- 2026-06-29: `cargo test --test project_intelligence_context_pack --quiet`
- 2026-06-29: `cargo run --quiet -- check --format json .`
- 2026-06-29: `cargo fmt --check`
- 2026-06-29: `cargo test --test content_query_cli --quiet`
- 2026-06-29: `cargo test --test content_runtime_dx_docs --quiet`
- 2026-06-29: `cargo xtask docs`
- 2026-06-29: `cargo xtask evidence`
- 2026-06-29: `git diff --check`
- 2026-06-29:
  `python3 ./.trellis/scripts/workflow_gate.py --platform codex --task .trellis/tasks/06-29-project-intelligence-context-pack`
  confirmed only current task dirty paths remained.
