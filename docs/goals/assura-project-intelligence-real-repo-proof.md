---
id: goal-assura-project-intelligence-real-repo-proof
type: goal
title: Assura project intelligence real repo proof
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-adoption-blueprint.md
  - docs/goals/assura-real-project-policy-proof.md
---

# Assura Project Intelligence Real Repo Proof

## Objective

Prove the project-intelligence adoption blueprint on realistic repository
content, not only purpose-built fixtures.

## Current Gap

The runtime has strong deterministic fixture coverage, and Assura dogfoods many
policy surfaces. A user evaluating usability still needs evidence that modeled
content, Markdown checks, relation queries, graph expansion, safe-fix previews,
and agent envelopes compose on a repo-shaped project with realistic docs and
drift.

## Scope

- Select Assura plus at least one realistic non-Assura repo fixture, generated
  package, or pinned small external project.
- Add deterministic materialization so validation does not depend on untracked
  local checkouts.
- Model project knowledge that resembles real maintainer workflows: goals,
  specs, ADRs, packages, docs, or release artifacts.
- Include valid and invalid states for broken relations, stale/missing fields,
  Markdown drift, and one safe-fix preview.
- Record command evidence and reviewer notes in `docs/analysis/`.

## Non-Goals

- No broad web-scale benchmark.
- No private repository dependency.
- No semantic quality claims beyond candidate retrieval.
- No provider-required code intelligence.

## Definition Of Done

- The real-repo proof package runs through `assura check --format json`,
  `assura content search`, `assura content missing-relations`,
  `assura content expand`, `assura content agent-query`, and
  `assura fix markdown --dry-run --format json`.
- Valid states pass and invalid states fail for the intended reasons.
- Evidence is checked in with exact commands and expected high-level results.
- Docs explain why the scenario is realistic and what is still artificial.
- The proof does not require network access during ordinary test runs.

## Validation Commands

```bash
cargo fmt --check
cargo test --test project_intelligence_real_repo_proof --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the scenario represents realistic repository knowledge, not a
  toy-only fixture.
- R2: Confirm valid/invalid states prove user-visible outcomes.
- R3: Confirm network access, external checkout, and generated artifact
  assumptions are documented.
- R4: Confirm evidence commands are reproducible by a reviewer.

## Reviewer Blocking Criteria

Block if the proof only exercises Assura itself, relies on untracked local
state, needs network access in the default test path, or records screenshots or
manual claims where deterministic command evidence is available.

## Progress Log

- 2026-06-29: Started implementation on task
  `.trellis/tasks/06-29-project-intelligence-real-repo-proof`. Added the
  deterministic non-Assura Beacon CRM fixture under
  `tests/fixtures/project_intelligence_real_repo/beacon_crm`, with valid and
  invalid states for typed epics, ADR decisions, package intelligence records,
  missing owner field drift, missing decision-reference drift, and materialized
  Markdown safe-fix preview. Added
  `tests/project_intelligence_real_repo_proof.rs` and
  `docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md`.
- 2026-06-29: Completed after focused review agent
  `019f1372-0287-7d50-a801-34014fb75a9c`. Review found that the first pass
  proved only Beacon CRM and that the diagnostics regression did not lock the
  missing-owner claim. Fixed both by adding an Assura-local `assura_goals`
  content collection over `docs/goals/*.md`, adding an Assura search/expand
  regression, documenting the Assura-plus-Beacon evidence, and asserting the
  missing-owner diagnostic and severity in
  `tests/project_intelligence_real_repo_proof.rs`. Final validation passed:
  `cargo fmt --check`, `git diff --check`,
  `cargo test --test project_intelligence_real_repo_proof --quiet`,
  `cargo test --test content_query_cli --quiet`,
  `cargo test --test content_runtime_dx_docs --quiet`,
  Beacon valid/invalid CLI probes, Assura goal search,
  `cargo run --quiet -- check --format json .`, `cargo xtask docs`, and
  `cargo xtask evidence`.
