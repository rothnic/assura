---
id: goal-assura-roadmap-11-markdown-outline-validation
type: goal
title: Assura roadmap 11 markdown outline validation
status: planned
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/2026-06-15-notation-clean-start-roadmap.md
  - docs/analysis/2026-06-18-markdown-tooling-evaluation.md
---

# Goal 11: Markdown Outline Validation

## Objective

Make Markdown heading validation match the concise outline notation in the
config spec without forcing users to maintain heading depth numbers manually.

This is a two-week team chunk for Markdown parsing, config normalization,
fixtures, docs, and diagnostic review.

Start this goal with a Markdown tooling decision record. Assura projects are
expected to need Markdown checks, so it is acceptable to adopt maintained
Markdown linting, frontmatter parsing, and link-validation tooling instead of
building generic document checks from scratch.

## Current Gap

This goal is not complete today because the config spec defines concise
Markdown outline notation, but the runtime proof for the full user-facing
contract is not in place. Existing Markdown checks and examples do not yet prove
optional parents, nested heading order, escaped question-mark headings, skipped
levels, and ambiguous root matching exactly as the spec describes.

## User Certainty Bar

A docs author should be able to encode a document outline once, run Assura, and
know whether the document is structurally correct without manually calculating
heading depths or reverse-engineering parser behavior.

## Scope

- Evaluate existing Markdown tooling before implementing custom checks. The
  evaluation must cover at least a Rust Markdown linter, a Rust link checker,
  parser/AST options, and frontmatter parsers.
- Prefer mature tooling for generic Markdown lint, frontmatter, and
  link-validation behavior when it satisfies Assura's configuration,
  diagnostics, performance, offline, and support-surface needs.
- Keep Assura-owned implementation focused on config-specific outline
  semantics, project-structure scoping, relationship composition, and diagnostic
  normalization that generic Markdown tools do not understand.
- Add parser tests for required headings, `?? ` optional headings, nested
  heading order, headings containing `?`, and object-form escapes.
- Add passing and failing Markdown fixtures for optional parents, missing
  required children, skipped heading levels, and ambiguous root-level matching.
- Update every affected public example, website example, generated example,
  fixture config, and test-case `.assura/config.yml` that teaches or exercises
  Markdown outline notation.
- Keep nested attributes only for custom match behavior such as regex or future
  validators.
- Update docs so Markdown outline examples match runtime behavior exactly.
- Confirm Markdown outline validation composes with package-doc relationship
  providers where relevant.

## Non-Goals

- No full Markdown linter replacement.
- No custom implementation of generic Markdown lint or link-checking behavior
  until the tooling evaluation shows the available options cannot satisfy the
  goal constraints.
- No arbitrary repository-defined command execution.
- No dependency graph validation.
- No implementation of future custom validators unless required by the outline
  contract.
- No backwards compatibility for any superseded outline notation introduced
  before this goal.

## Definition Of Done

- The shorthand outline notation from the config spec has passing and failing
  coverage.
- A Markdown tooling decision record explains what will be adopted, wrapped, or
  deliberately custom-built, with evidence for rejected mature tools.
- Optional parent sections behave predictably when present or absent.
- Ambiguous root matching produces a deterministic error.
- Docs show how to represent headings that contain or start with question
  marks.
- Diagnostics identify the configured outline entry and observed Markdown
  location where practical.
- A first-time docs author can fix every outline fixture failure from the report
  text and docs.
- Outline notation changes have checked performance evidence or a bounded
  inherent-cost record.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test markdown --quiet
cargo test structure_notation --quiet
cargo run --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm outline behavior follows `.trellis/spec/assura/config-notation.md`.
- R1: Confirm the Markdown tooling decision record evaluates maintained
  Markdown linting, frontmatter, parser/AST, and link-checking options before
  custom implementation.
- R2: Review required, optional, nested, escaped, and ambiguous fixtures.
- R3: Review diagnostics against real docs-authoring mistakes.
- R4: Confirm examples do not require users to sync heading level integers.
- R5: Reproduce fixture reports from the CLI.
- R6: Confirm public examples, generated examples, fixtures, and test-case
  configs were migrated consistently.
- R7: Confirm the PR links this goal and a Markdown outline review artifact.

## Reviewer Blocking Criteria

Block the PR if skipped levels pass silently, if optional parents make required
children ambiguous, or if the docs describe outline behavior not backed by
runtime tests. Also block if outline notation changes skip performance evidence
or preserve superseded notation without an explicit support-policy exception.
Block custom generic Markdown linting, frontmatter parsing, or link checking
unless the tooling decision record proves maintained tooling is unsuitable for
Assura's needs.

## Progress Log

- 2026-06-18: Revalidated against current `origin/master` after Goal 10 merged.
  The goal remains valid: the spec defines nested `markdown.outline` shorthand,
  the repo still exposes older frontmatter/required-section/heading-depth
  Markdown checks, and the tooling evaluation is an initial candidate list
  rather than a completed implementation decision.
