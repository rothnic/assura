---
id: goal-assura-code-doc-reference-validation
type: goal
title: Assura code and documentation reference validation
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ../project-intelligence-facts.md
---

# Assura Code And Documentation Reference Validation

## Objective

Validate repository-internal references across Markdown, source comments,
docstrings, and simple string-like references so docs and code do not silently
rot when files, headings, or line targets move.

## Current Gap

The Markdown reference engine goal includes this requirement, but the beta
program needs a separately reviewable epic for the repository reference graph
itself. Markdown linting can land first, but beta is not complete until
code-to-doc and doc-to-code references are discoverable, validated, and useful
for affected-file feedback.

## Scope

- Discover Markdown links to files, headings, and line ranges.
- Discover cheap repository-relative references from comments, doc comments,
  docstrings, and obvious string literals.
- Validate target files, headings, line numbers, and line ranges.
- Store inbound and outbound reference edges as project facts.
- Explain affected sources when a target is changed, moved, or deleted.
- Explain affected targets when a source reference changes.
- Keep the scanner language-agnostic by default, with optional cheap language
  helpers where they improve precision without requiring an LSP.

## Non-Goals

- No full semantic code analysis.
- No remote link checking in the default path.
- No requirement for LSP, SCIP, or hosted indexers.
- No automatic broad rewrite of ambiguous references.

## Definition Of Done

- Whole-repository checks report broken doc-to-doc, doc-to-code, code-to-doc,
  and code-to-file references where syntax is locally recognizable.
- Reference facts include source path/span, target path/anchor, rule ID, and
  target status.
- Changed-source and changed-target affected-set tests pass.
- Agent and daemon callers can request bounded reference context.
- Docs show examples of stale comments or docstrings being caught.

## Validation Commands

```bash
cargo fmt --check
cargo test --test repository_reference_graph_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm reference discovery is conservative and explains confidence.
- R2: Confirm GitHub-renderable Markdown links are enforced where applicable.
- R3: Confirm inbound references are available for changed or deleted targets.
- R4: Confirm source scanners do not require language-specific services.

## Reviewer Blocking Criteria

Block if code/comment references can rot silently, inbound edges are missing,
diagnostics lack source/target spans, remote services become required, or
ambiguous references are rewritten without explicit user action.
