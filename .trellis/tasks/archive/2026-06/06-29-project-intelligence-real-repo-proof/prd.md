# Project Intelligence Real Repo Proof

## Goal

Complete the next Project Intelligence Usability successor by proving the
adoption blueprint on a deterministic realistic non-Assura repo package.

## What I Already Know

- `docs/goals/assura-project-intelligence-real-repo-proof.md` is the next
  successor after the adoption demo.
- Existing `tests/fixtures/content_runtime` fixtures prove individual runtime
  features, but the goal requires a realistic repo-shaped package.
- Existing `tests/fixtures/real-project-agentic-feedback` proves structure and
  agent feedback on a TypeScript-style repo, but not project-intelligence
  content/query/safe-fix composition.

## Requirements

- Add a deterministic non-Assura realistic repo fixture package.
- Include valid and invalid states for typed content, broken relations,
  missing/stale fields, Markdown drift, and safe-fix preview.
- Add tests that exercise `assura check`, content search, missing-relations,
  expand, agent-query, and markdown safe-fix dry-run.
- Add checked evidence in `docs/analysis/`.
- Update docs-site examples so users can find the real-repo proof from the
  visual demo.
- Record progress in the real-repo proof and usability program goals.

## Acceptance Criteria

- [x] Fixture package exists and does not require network access.
- [x] Valid fixture passes and invalid fixture fails for intended reasons.
- [x] Integration tests cover required commands.
- [x] Docs/evidence artifact records exact commands and expected outcomes.
- [x] Docs-site example links to the real-repo proof.
- [x] Validation passes.

## Out Of Scope

- Persistent sessions, LSP/MCP transports, and safe-fix apply behavior.
- External repo checkout during ordinary tests.
- Remote semantic or code-intelligence provider requirements.

## Technical Notes

- Goal: `docs/goals/assura-project-intelligence-real-repo-proof.md`.
- Program: `docs/goals/assura-project-intelligence-usability-program.md`.
- Existing patterns: `tests/content_query_cli.rs`,
  `tests/markdown_lint_fix_tests.rs`, and
  `tests/fixtures/real-project-agentic-feedback`.

## Validation Evidence

- 2026-06-29: `cargo fmt --check` passed.
- 2026-06-29: `git diff --check` passed.
- 2026-06-29: `cargo test --test project_intelligence_real_repo_proof --quiet`
  passed.
- 2026-06-29: `cargo test --test content_query_cli --quiet` passed.
- 2026-06-29: `cargo test --test content_runtime_dx_docs --quiet` passed.
- 2026-06-29: `cargo run --quiet -- check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/valid`
  passed with 11 files and 10 directories checked.
- 2026-06-29: `cargo run --quiet -- check --format json tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid`
  exited nonzero and reported `content_runtime:invalid_object_shape` plus
  `content_runtime:missing_reference`.
- 2026-06-29: `cargo run --quiet -- content search "checkout onboarding" tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --format json`
  returned the `epic-checkout` model instance and Markdown section.
- 2026-06-29: `cargo run --quiet -- content expand epics epic-checkout tests/fixtures/project_intelligence_real_repo/beacon_crm/valid --format json`
  returned related package and ADR facts.
- 2026-06-29: `cargo run --quiet -- content missing-relations tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json`
  returned `adr-missing-payment-risk`.
- 2026-06-29: `cargo run --quiet -- content agent-query diagnostics tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --format json`
  returned `assura.project-intelligence.agent-query.v1` diagnostics.
- 2026-06-29: `cargo run --quiet -- content search "Project Intelligence Usability" . --format json`
  returned the Assura repo's `goal-assura-project-intelligence-usability-program`
  model instance from the `assura_goals` collection.
- 2026-06-29: `cargo run --quiet -- check --format json .` passed with 1100
  files and 271 directories checked.
- 2026-06-29: `cargo xtask docs` passed.
- 2026-06-29: `cargo xtask evidence` passed.
