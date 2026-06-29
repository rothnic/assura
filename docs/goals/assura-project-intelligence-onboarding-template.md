---
id: goal-assura-project-intelligence-onboarding-template
type: goal
title: Assura project intelligence onboarding template
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-adoption-blueprint.md
  - docs/goals/assura-project-intelligence-real-repo-proof.md
  - docs/analysis/2026-06-29-project-intelligence-usability-gap-evaluation.md
---

# Assura Project Intelligence Onboarding Template

## Objective

Make first-run project-intelligence setup reproducible from a normal repository
without requiring users to hand-author the initial schema, collections, sample
records, and validation commands.

## Current Gap

The adoption demo explains the path and the real-repo proof shows it works, but
both still assume a maintainer can design the first content model and config.
That is too much friction for a user trying to decide whether Assura is useful.

## Scope

- Choose the smallest supported starter surface: an `assura init` profile,
  checked template package, documented copy path, or another repo-native
  mechanism with tests.
- Include a minimal content schema, `.assura/config.yml` collection setup,
  example Markdown/JSON records, and one broken-state example.
- Prove the generated or copied starter with `assura check`, content search,
  graph expansion, missing-relations, and agent-query diagnostics.
- Document how a user replaces the sample model with their own goals, specs,
  ADRs, packages, or release artifacts.
- Keep the path local and source-control friendly.

## Non-Goals

- No hosted setup service.
- No schema generation from prose.
- No migration assistant for arbitrary existing docs.
- No editor or daemon dependency.

## Definition Of Done

- A fresh temporary repo can adopt the starter and reach a clean first
  project-intelligence check with deterministic commands.
- The starter includes at least one modeled object relation and one documented
  invalid-state diagnostic.
- Website docs show the starter path before the lower-level manual setup path.
- Regression tests prove the starter stays in sync with live CLI behavior.
- The result can be reused by later context-pack and transport goals.

## Validation Commands

```bash
cargo fmt --check
cargo test --test project_intelligence_onboarding --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the starter is a real first-run path, not another explanation of
  manual config.
- R2: Confirm generated or copied files are deterministic and repo-local.
- R3: Confirm docs do not imply hosted setup, semantic providers, or editor
  plugins are required.
- R4: Confirm tests fail if the starter drifts from live command behavior.

## Reviewer Blocking Criteria

Block if the path still requires users to invent the first schema by hand, if
the starter relies on untracked local files, if command evidence is missing, or
if docs advertise unsupported automation.

## Progress Log

- 2026-06-29: Started implementation on task
  `.trellis/tasks/06-29-project-intelligence-onboarding-template`. Added the
  `assura init --project-intelligence` starter profile, deterministic
  repo-local schema/goal/spec/ADR/example files, focused onboarding
  regressions, and website/API documentation. Initial focused validation
  passed: `cargo fmt --check`,
  `cargo test --test project_intelligence_onboarding --quiet`,
  `cargo test --test content_runtime_dx_docs --quiet`, and
  `cargo test --test cli_command_surface_tests init --quiet`. Review is in
  progress under agent `019f1391-779f-76e0-bf23-f6c6ebff7de7`.
- 2026-06-29: Addressed review findings from agent
  `019f1391-779f-76e0-bf23-f6c6ebff7de7` by adding
  `--project-intelligence` to `.assura/command-surface.yml`, moving starter
  materialization back under `src/cli/init_support.rs` so edited CLI files
  stay under the 500-line policy, and updating roadmap routing to
  `docs/goals/assura-project-intelligence-context-pack.md`.
