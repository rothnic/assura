---
id: goal-assura-project-intelligence-simple-cli
type: goal
title: Assura project intelligence simple CLI
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-repo-wide-code-content-search.md
  - docs/goals/assura-project-intelligence-content-model-validation-demo.md
---

# Assura Project Intelligence Simple CLI

## Objective

Add one low-ceremony project-intelligence command for normal daily use so a
maintainer or local agent can search a repository without choosing between
`content search`, `semantic-search`, `symbols`, `symbol-refs`,
`context-pack`, and `expand` up front.

## Current Gap

The completed Project Intelligence usability slice exposes useful primitives,
but the common path still requires too much product knowledge. Users have to
know that keyword search is lexical scoring, semantic candidates are a separate
opt-in command, graph expansion requires a collection and ID, and code-symbol
traversal is separate from content search.

## Proposed User Shape

The target shape is a future `assura find` command that can accept natural
search text such as `checkout timeout` or a code symbol such as
`crate::checkout::Timeout` without requiring a collection, object ID, or
provider flag.

The command should default to the current repository, return ranked results,
show whether a result came from content, code, diagnostics, or relations, and
offer the next expansion command when more context is available.

## Scope

- Design a concise CLI name, aliases, defaults, and JSON schema.
- Return ranked results with score explanation fields.
- Include modeled content, Markdown sections, diagnostics, code-symbol facts,
  and relation hints where available.
- Provide a simple text output that is useful without `--format json`.
- Keep lower-level `assura content ...` and `assura agent ...` commands as
  stable building blocks for wrappers.
- Add tests for default path behavior, JSON schema, ranking explanation, and
  no-result output.

## Non-Goals

- No remote search provider.
- No natural-language planner that edits files.
- No claim that ranking decides validation correctness.
- No MCP dependency for the core CLI path.

## Definition Of Done

- A new user can run one command from the repo root and get useful ranked
  project-intelligence results.
- Results explain source kind, score, path, and suggested follow-up.
- JSON output is stable enough for local agents.
- Docs use the simple command first, then disclose lower-level primitives.
- Existing content and agent commands remain covered by compatibility tests.

## Validation Commands

```bash
cargo fmt --check
cargo test --test content_query_cli --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm the command is meaningfully simpler than the existing primitive
  commands.
- R2: Confirm score fields are explained as ranking signals, not correctness.
- R3: Confirm local agents can use JSON output without scraping text.
- R4: Confirm no remote service or MCP server is required.

## Reviewer Blocking Criteria

Block if the command requires complex flags for the common path, hides
validation failures behind ranking, duplicates lower-level query logic instead
of reusing shared facts, or makes MCP/remote access a prerequisite.
