---
id: goal-assura-project-intelligence-content-model-validation-demo
type: goal
title: Assura project intelligence content model validation demo
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/goals/assura-rust-markdown-validation-and-fixing.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
---

# Assura Project Intelligence Content Model Validation Demo

## Objective

Create a clear end-to-end demo that proves Assura catches incomplete or invalid
content when a content model changes, including Markdown frontmatter schema
violations, relation drift, and Markdown lint issues.

## Current Gap

The runtime can validate content models and Markdown, but the public demo is
still hard to follow. It does not yet walk through a tiny model change, show
the invalid files, then show the exact diagnostics and safe-fix preview a user
would act on.

## Scope

- Add a small fixture or starter scenario with:
  - a valid Markdown frontmatter record;
  - a schema change that makes one record invalid;
  - a missing relation target;
  - a Markdown lint issue with a safe-fix preview.
- Document the workflow in the website using copy/paste commands and trimmed
  output.
- Prove `assura check` reports schema/frontmatter failures.
- Prove `assura content search` can find diagnostics.
- Prove `assura content expand` shows related context from the affected record.
- Prove `assura fix markdown --dry-run` previews the lint repair without
  writing.

## Non-Goals

- No automatic semantic repair for invalid frontmatter.
- No authoring UI.
- No remote schema registry.
- No generated code requirement for validation.

## Definition Of Done

- A reader can see the model, the invalid content, the command, and the
  diagnostic in one short guide.
- The demo distinguishes frontmatter/schema validation from Markdown linting.
- The demo shows how diagnostics connect back to search and graph expansion.
- Tests guard the fixture outputs used by the guide.

## Validation Commands

```bash
cargo fmt --check
cargo test --test content_runtime_check_cli --quiet
cargo test --test markdown_lint_fix_tests --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm the demo is understandable without knowing Assura internals.
- R2: Confirm the invalid state is caused by model/content drift, not a toy
  typo unrelated to the schema.
- R3: Confirm lint safe fixes stay opt-in and preview-first.
- R4: Confirm all command examples are copy/pasteable from the repo.

## Reviewer Blocking Criteria

Block if the demo only describes validation without executable evidence, mixes
schema validation with lint fixes so users cannot tell them apart, or implies
Assura can automatically repair semantic content model failures.
