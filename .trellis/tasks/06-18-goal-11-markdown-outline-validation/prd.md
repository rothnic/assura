# Goal 11 Markdown Outline Validation

## Goal

Make Assura's Markdown outline validation follow the concise nested outline
notation in `.trellis/spec/assura/config-notation.md`, so docs authors can
express required and optional heading structure without manually tracking
heading depths.

## Current Evidence

- `docs/goals/assura-goal-11-markdown-outline-validation.md` remains planned
  and names the missing runtime proof for optional parents, nested heading
  order, escaped question-mark headings, skipped levels, and ambiguous root
  matching.
- `.trellis/spec/assura/config-notation.md` defines nested
  `markdown.outline` shorthand, `?? ` optional headings, object-form escapes,
  relative root matching, skipped-level failures, and ambiguous-root failures.
- `docs/analysis/2026-06-18-markdown-tooling-evaluation.md` records the
  initial tooling candidates but does not yet make an implementation decision.
- Current CLI Markdown checks cover frontmatter, required sections, and maximum
  heading depth; they do not prove the full outline contract.

## Requirements

- Finish the Markdown tooling decision record before adding or extending
  generic Markdown checks.
- Keep Assura-owned code focused on config-specific outline semantics,
  project-structure scoping, relationship composition, and diagnostic
  normalization.
- Support required headings, `?? ` optional headings, headings containing `?`,
  headings starting with literal `?? ` through object form, optional parent
  sections, nested heading order, skipped-level failures, and deterministic
  ambiguous-root failures.
- Update public docs, website reference docs, fixtures, generated examples, and
  test-case `.assura/config.yml` files that teach or exercise Markdown outline
  notation.
- Confirm outline validation composes with package-doc relationship providers
  where relevant.
- Preserve performance evidence or record a bounded inherent-cost decision.

## Non-Goals

- Do not build a full Markdown linter replacement.
- Do not implement generic Markdown lint, frontmatter parsing, or link checking
  unless the decision record shows maintained tooling cannot satisfy Assura's
  needs.
- Do not add arbitrary repository-defined command execution.
- Do not add dependency graph validation or future custom validators unless the
  outline contract requires them.
- Do not preserve superseded alpha outline notation without an explicit support
  exception and removal plan.

## Acceptance Criteria

- Passing and failing tests cover the shorthand outline notation from the
  config spec.
- Optional parent behavior is predictable when parents are present or absent.
- Ambiguous root matching fails deterministically.
- Diagnostics identify the configured outline entry and observed Markdown
  location where practical.
- Docs show how to represent headings that contain or start with question
  marks, and those examples are backed by tests.
- A first-time docs author can fix each outline fixture failure from the report
  text and docs.
- Review artifact links this goal and records fixture, diagnostics, examples,
  tooling, and performance review results.

## Validation

Run at minimum:

```bash
cargo fmt --all -- --check
cargo test markdown --quiet
cargo test structure_notation --quiet
cargo run --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
git diff --check
```

Add `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo xtask evidence`, and `cargo xtask docs` before PR if Rust behavior or
docs/website surfaces change.

## Review Plan

- Use an independent review agent before opening the PR.
- Ask the reviewer to block on skipped-level false passes, optional-parent
  ambiguity, docs/runtime drift, missing performance evidence, unreviewed
  custom generic Markdown behavior, and incomplete example migration.

## Technical Notes

- Goal file: `docs/goals/assura-goal-11-markdown-outline-validation.md`
- Spec: `.trellis/spec/assura/config-notation.md`
- Tooling analysis: `docs/analysis/2026-06-18-markdown-tooling-evaluation.md`
- Current runtime Markdown entrypoint: `src/cli/check/markdown.rs`
- Current config bundle surface: `src/config/config/bundles.rs`
