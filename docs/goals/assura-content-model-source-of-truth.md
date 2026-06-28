---
id: goal-assura-content-model-source-of-truth
type: goal
title: Assura content model source of truth
status: planned
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - src/cli/check/markdown.rs
  - src/content_repository/
  - website/src/content/docs/reference/configuration.md
---

# Assura Content Model Source Of Truth

## Objective

Make repo-native content runtime models the only supported source of truth for
typed frontmatter fields in content collections. Remove or reroute legacy
Markdown-frontmatter required-field validation so Assura has one correct model
path instead of parallel convenience surfaces.

## Current Gap

The content runtime can model object shape, required fields, optional fields,
IDs, and relations. Older Markdown config still exposes frontmatter-oriented
checks such as required fields on the Markdown validation surface. That creates
two ways to express the same policy and invites drift.

## User Certainty Bar

A user should know that typed frontmatter belongs in a modeled collection. If a
Markdown file is an instance of a `Goal`, `Requirement`, `Decision`, or another
model, required fields and relation fields should be defined by the model, not
by a separate Markdown rule.

## Scope

- Audit current Markdown frontmatter checks and all docs/examples that teach
  them.
- Decide which checks remain generic Markdown concerns and which become content
  runtime concerns.
- Remove `markdown.required_fields` from the supported Assura-authored config
  path, or make it a hard error that points to modeled collections.
- Decide whether `markdown.require_frontmatter` remains only as a generic
  Markdown/document-style rule or also moves behind modeled collections.
- Route typed frontmatter validation through `models`, `collections`, and
  `relations`.
- Update config docs, website docs, examples, fixtures, and tests to show the
  single model-backed path.
- Add migration diagnostics that are clear without preserving legacy behavior.

## Non-Goals

- No compatibility shim that continues validating typed fields in both places.
- No new schema language beyond the current content runtime authoring path.
- No broad graph/search implementation in this goal.
- No generic Markdown lint/fix integration in this goal.

## Definition Of Done

- There is one supported path for required typed frontmatter fields:
  content runtime models.
- Legacy docs no longer present `markdown.required_fields` as a recommended or
  supported typed-content mechanism.
- Validation diagnostics tell users to model frontmatter through collections
  when they configure typed frontmatter in the wrong place.
- Existing Markdown outline validation still works for required and optional
  heading hierarchy.
- Tests cover a passing modeled-frontmatter case and a failing legacy duplicate
  surface case.
- Public docs and website docs explain the split between Markdown formatting,
  heading hierarchy, and typed frontmatter models.

## Validation Commands

```bash
cargo fmt --check
cargo test markdown --quiet
cargo test --test content_runtime_validation --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm no duplicated typed-frontmatter validation path remains in public
  docs or config examples.
- R2: Confirm the replacement path uses content runtime schema validation and
  relation validation.
- R3: Confirm users still have a clear way to require document headings.
- R4: Confirm any breaking config behavior is intentional under the pre-1.0
  compatibility policy.

## Reviewer Blocking Criteria

Block if typed required fields can still be validated through both Markdown
rules and content runtime models, if docs continue teaching the old path, or if
the change removes heading hierarchy validation by accident.
