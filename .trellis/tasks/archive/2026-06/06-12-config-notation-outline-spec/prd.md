# Config notation outline spec

## Goal

Define the stable Assura-native config notation target for compact structure
rules, reusable rule fragments, Markdown outline validation, and code-to-doc
relationship checks. The spec should give implementation work a clear product
target while preserving the current structure-first direction and LS-Lint
simplicity.

## What I Already Know

- Assura's current public config surface is `structure:` plus `exclude:`.
- The current format can represent closed-world direct contents, direct
  `exists` counts, LS-Lint-compatible scopes, and Markdown bundle settings, but
  simple cases are verbose.
- The desired notation should visually mirror the project tree rather than
  place direct children under a separate `contents:` block.
- `required` and `allowed` should not duplicate the same intent in two
  different fields for common cases.
- `exists` cardinality can express required, optional, forbidden, and bounded
  direct contents.
- Markdown heading validation should preserve heading order and nesting in the
  config itself.
- Optional heading notation should be concise and on the same line as the
  heading.
- A single bare leading `?` is risky in YAML because YAML treats `? ` as an
  explicit mapping-key indicator.
- Assura should eventually validate relationships such as "every package has a
  dedicated docs page or a section in an aggregate docs page."

## Requirements

- Add a durable Assura spec document for the target compact notation.
- Define key use cases that implementation should prove.
- Make clear which pieces are implemented today versus target notation.
- Preserve current `structure:` as the native tree; do not introduce a parallel
  root language that conflicts with the current product surface.
- Prefer concise shorthand for simple cases and object attributes only for
  customization.
- Define a compact Markdown outline syntax where nesting represents heading
  nesting and `?? ` marks optional headings.
- Explain how headings containing `?` are represented without ambiguity.
- Define how reusable rules reduce duplication across root and monorepo package
  scopes.
- Define how code-to-doc relationship templates can validate package docs.
- Update project source-truth indexes so future implementation can find the
  spec.

## Acceptance Criteria

- [x] `.trellis/spec/assura/` contains a current config-notation spec.
- [x] `.trellis/spec/assura/index.md` links the new spec.
- [x] Existing notation source-truth docs point to the new stable target.
- [x] The constitution no longer points readers only at archived notation docs.
- [x] The task contains a research note with references to YAML, MkDocs,
  markdownlint, and the personal LS-Lint fork examples.
- [x] Docs validation passes for the changed surface.
- [x] A review pass happens before PR creation.
- [ ] A PR is created and merged into `master`.

## Out Of Scope

- Implementing the parser or validator for the new notation.
- Migrating `.assura/config.yml` to the new notation.
- Changing runtime LS-Lint compatibility behavior.
- Adding new Rust validation logic for Markdown outlines or relations.

## Technical Notes

- Current source truth: `docs/analysis/2026-05-15-notation-source-truth.md`.
- Current closed-world contract:
  `.trellis/spec/assura/structure-enforcement.md`.
- Historical design input: `docs/unified-tree-design.md` and
  `docs/archive/final-config-design.md`.
- Research artifact:
  `.trellis/tasks/06-12-config-notation-outline-spec/research/config-notation-references.md`.
