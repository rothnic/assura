---
id: analysis-2026-06-18-goal-11-markdown-outline-review
type: analysis
title: Goal 11 markdown outline validation review
status: active
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/2026-06-18-markdown-tooling-evaluation.md
---

# Goal 11 Markdown Outline Validation Review

## Implementation Summary

- Added `markdown.outline` to the structure config `MarkdownBundle` with
  shorthand text nodes, shorthand parent maps, and object-form nodes.
- Added config-load validation for empty outline titles, malformed shorthand
  parent maps, and unsupported match modes.
- Added runtime `markdown_outline` diagnostics for missing headings, skipped
  heading levels, ambiguous roots, no-heading documents, and invalid regex
  matchers.
- Preserved outline config through inherited rule merging and compiled check
  artifacts.
- Documented the tooling decision: no new generic Markdown lint,
  link-checking, or frontmatter dependency for this slice; keep Assura-owned
  code focused on config-specific outline semantics.

## Covered Behavior

- Required headings and nested heading order.
- `?? ` optional headings and absent optional parents.
- Required children when an optional parent is present.
- Headings containing question marks.
- Required headings that start with literal `?? ` through object form.
- Skipped heading levels as validation failures.
- Ambiguous root matching as deterministic validation failure.
- Package documentation relationship providers composed with outline checks.

## Validation Run

- `cargo fmt --all -- --check`
- `cargo test markdown --quiet`
- `cargo test structure_notation --quiet`
- `cargo test --test markdown_outline_config_notation_tests --quiet`
- `cargo test --test structure_config_notation_tests --quiet`
- `cargo run --quiet -- performance-report --output target/performance/current.json`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `git diff --check`

## Review Status

Independent review requested from Trellis check agent Hume on 2026-06-18.

Finding:

- High: Optional-only outlines failed on headingless documents because
  headingless documents were rejected before checking whether the configured
  outline contained any required entries.

Resolution:

- Moved the headingless-document failure behind required-entry detection, so an
  outline made entirely of optional entries can be absent.
- Added `markdown_outline_allows_headingless_document_when_every_entry_is_optional`
  to cover the regression.

GitHub review:

- Gemini flagged a high-priority false positive where `find_heading` walked
  into child headings of an unmatched sibling section and reported a skipped
  level.
- Gemini flagged repeated regex compilation in root and heading matching.

Resolution:

- `find_heading` now skips unmatched sibling sections with `section_end`.
- Outline entry regexes are compiled once per root search or entry search.
- Added `markdown_outline_skips_unmatched_sibling_sections` to cover the false
  positive.
