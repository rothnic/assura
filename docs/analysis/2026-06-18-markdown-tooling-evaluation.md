---
id: analysis-2026-06-18-markdown-tooling-evaluation
type: analysis
title: Markdown tooling evaluation
status: active
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-11-markdown-outline-validation.md
  - .trellis/spec/assura/config-notation.md
---

# Markdown Tooling Evaluation

Assura should not build a full Markdown linting stack before evaluating
existing Rust-native tooling. Markdown validation is likely part of the normal
Assura project contract, so bringing in Markdown linting, frontmatter parsing,
and link validation is acceptable when the tool is maintained, performant, and
configurable enough for project-structure checks.

## Current Repo Baseline

Assura already has in-tree Markdown checks for frontmatter presence, required
frontmatter fields, heading depth, and required sections. The implementation is
purpose-built and lightweight, but it does not replace a Markdown linter or
general link checker.

`Cargo.toml` currently lists optional Markdown-related dependencies:

- `pulldown-cmark`
- `frontmatter`

## Tooling Candidates

- `rumdl`: Rust Markdown linter and formatter with markdownlint-compatible rule
  coverage, extra rules, config support, and performance claims. It includes
  relevant rule areas for headings, links, frontmatter spacing/key ordering,
  table of contents validation, and relative-link validation.
  Source: https://rumdl.dev/
- `lychee`: Rust async link checker for Markdown, HTML, reStructuredText, and
  websites with JSON output and CI-friendly behavior.
  Source: https://lychee.cli.rs/overview/
- `comrak`: Rust CommonMark and GitHub Flavored Markdown parser/renderer with
  an AST API. It is a stronger parser candidate than ad hoc heading scanning
  when Assura needs precise CommonMark/GFM behavior.
  Source: https://github.com/kivikakk/comrak
- `markdown` / `markdown-rs`: Rust Markdown parser with mdast output and support
  for common extensions including MDX and frontmatter. It may be a better fit
  if Assura wants a unified-style AST model.
  Source: https://github.com/wooorm/markdown-rs
- `gray_matter` or `markdown-frontmatter`: focused frontmatter parsers that can
  replace hand-rolled frontmatter splitting if the selected Markdown parser does
  not cover Assura's needs.
  Sources: https://docs.rs/gray_matter and
  https://docs.rs/markdown-frontmatter

## Initial Direction

Goal 11 should start with a tooling fit decision before custom implementation.
The expected default is:

- prefer `rumdl` or another mature linter for general Markdown lint rules;
- prefer `lychee` or an equivalent Rust link checker for link validation;
- prefer a real parser/AST for outline semantics if heading behavior needs to
  match CommonMark or GFM precisely;
- keep Assura-owned code for config-specific outline semantics, diagnostics,
  project-structure scoping, and relationship composition that generic tools do
  not understand.

The implementation decision must compare library versus CLI integration,
configuration model, output normalization, offline behavior, internal-anchor
support, performance cost, and support surface before adding dependencies.
