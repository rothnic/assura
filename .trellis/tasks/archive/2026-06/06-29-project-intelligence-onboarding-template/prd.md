# Project Intelligence Onboarding Template

## Objective

Implement the next Project Intelligence Usability successor:
`docs/goals/assura-project-intelligence-onboarding-template.md`.

## Requirements

- Add a first-run starter path for project-intelligence setup.
- Generate deterministic repo-local starter files.
- Prove the starter with check, search, graph expansion, missing-relations, and
  agent-query diagnostics.
- Put the starter path in website docs before the manual setup path.
- Keep the implementation local-only and avoid editor, daemon, hosted service,
  or semantic-provider dependencies.

## Acceptance Criteria

- [x] A fresh temporary repo can run the starter path and pass
  `assura check --format json`.
- [x] Starter files include a content schema, config collections, modeled
  records, at least one relation, and one broken-state example.
- [x] Regression tests cover valid and invalid starter states with the live CLI.
- [x] Website docs show the starter path before lower-level manual setup.
- [x] The goal progress log and roadmap are updated with evidence.
- [x] Validation commands in the goal pass.

## Non-Goals

- No context-pack implementation.
- No persistent session, daemon, editor, LSP, or MCP support.
- No safe-fix apply behavior.

## Validation Evidence

- 2026-06-29: `cargo fmt --check`
- 2026-06-29: `cargo test --test project_intelligence_onboarding --quiet`
- 2026-06-29: `cargo test --test content_runtime_dx_docs --quiet`
- 2026-06-29: `cargo test --test cli_command_surface_tests init --quiet`
- 2026-06-29: `cargo run --quiet -- check --format json .`
- 2026-06-29: `cargo xtask docs`
- 2026-06-29: `cargo xtask evidence`
- 2026-06-29: `git diff --check`
- 2026-06-29:
  `python3 ./.trellis/scripts/workflow_gate.py --platform codex --task .trellis/tasks/06-29-project-intelligence-onboarding-template`
  confirmed only current task dirty paths remained.
