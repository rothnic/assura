# Project Intelligence Usability Execution

## Goal

Execute the Project Intelligence Usability program from
`docs/goals/assura-project-intelligence-usability-program.md`, beginning with
the adoption blueprint and the requested visual documentation demo.

## What I Already Know

- The persistent objective is to complete all scope in the usability program
  and provide visual documentation examples for how to use the capability.
- The program sequence starts with
  `docs/goals/assura-project-intelligence-adoption-blueprint.md`.
- Existing fixtures under `tests/fixtures/content_runtime/` already exercise
  typed content, missing relations, semantic candidates, code symbols, and
  agent-query envelopes.
- The documentation site uses Starlight with sidebar routing in
  `website/astro.config.mjs`.

## Requirements

- Start with a first executable slice that advances the adoption blueprint.
- Add docs-site examples that show the project-intelligence workflow visually
  and with copyable commands.
- Keep examples on supported local surfaces: `assura check`,
  `assura content`, `assura content agent-context`,
  `assura content agent-query`, and `assura fix markdown --dry-run`.
- Link the new walkthrough from discoverable docs navigation and relevant
  product/example pages.
- Record progress in the relevant goal docs.

## Acceptance Criteria

- [x] A docs page or section provides a visual project-intelligence demo.
- [x] The demo includes modeled content, check output, search/graph queries,
  missing relation diagnosis, agent envelope, and safe-fix preview.
- [x] Sidebar/navigation exposes the demo.
- [x] Adoption-blueprint progress is recorded.
- [x] Validation passes for docs, Assura self-check, and targeted content-query
  commands.

## Definition Of Done

- The first usability slice is committed with validation evidence.
- The full persistent goal remains active until all usability-program successor
  goals are complete and audited.

## Out Of Scope For This Slice

- Implementing persistent sessions, LSP, MCP, or safe-fix apply behavior.
- Claiming completion of the full usability program.
- Adding remote provider requirements.

## Technical Notes

- Program: `docs/goals/assura-project-intelligence-usability-program.md`.
- First successor:
  `docs/goals/assura-project-intelligence-adoption-blueprint.md`.
- Existing docs: `website/src/content/docs/examples/content-runtime.md`,
  `website/src/content/docs/product/query-search.md`, and
  `website/src/content/docs/product/agent-editor-surfaces.md`.
- Existing fixture root: `tests/fixtures/content_runtime/`.

## Validation Evidence

- 2026-06-29: `cargo fmt --check` passed.
- 2026-06-29: `git diff --check` passed.
- 2026-06-29: `cargo test --test content_runtime_dx_docs --quiet` passed.
- 2026-06-29: `cargo run --quiet -- content search "Portable Structure" tests/fixtures/content_runtime/valid --format json` returned modeled and Markdown-section matches.
- 2026-06-29: `cargo run --quiet -- content missing-relations tests/fixtures/content_runtime/missing_reference --format json` returned the missing `missing-spec` relation.
- 2026-06-29: `cargo run --quiet -- content agent-query diagnostics tests/fixtures/content_runtime/missing_reference --format json` returned `assura.project-intelligence.agent-query.v1` with `content_runtime:missing_reference`.
- 2026-06-29: `cargo run --quiet -- check --format json .` passed with 1094 files and 270 directories checked.
- 2026-06-29: `cargo xtask docs` passed and built 37 pages, including `/examples/project-intelligence-demo/`.
- 2026-06-29: `cargo xtask evidence` passed.
- 2026-06-29: Playwright rendered `http://127.0.0.1:4322/examples/project-intelligence-demo/` on 1280x900 and 390x844 viewports; both had 4 workflow steps, 3 demo lanes, required search/agent/safe-fix text, and no detected demo-box overlap.
