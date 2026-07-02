# Markdown Engine Candidate Fixtures

This package is evidence scaffolding for evaluating a future
markdownlint-compatible Rust engine. It does not adopt an engine.

The fixtures model a documentation-heavy Rust CLI repository:

- `valid/` is the clean baseline Assura must keep passing.
- `invalid/` keeps intentional structure, markdownlint-style, Assura
  link/reference, suppression, and safe-fix findings.
- `frontmatter_link_heavy/`, `large_doc/`, and `fixable_drift/` are
  representative probe profiles for candidate timing and fix-cost evidence.
- `matrix.json` records candidate-agnostic expectations for rule mapping,
  staged ordering, severity ownership, safe-fix behavior, and named probe
  profiles.

Normal tests use only the current Assura binary. External candidate probes must
skip or report unavailable tools instead of making CI depend on third-party
binaries.
