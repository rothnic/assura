# Reference Graph Validation

## Problem

The beta program has completed Markdown Quality, including outbound
`MarkdownLink` facts. Assura still needs a repository-reference layer that can
validate recognizable code/doc references beyond Markdown-authored links and
make inbound/outbound affected context available for later daemon and agent
work.

## Goal

Execute `docs/goals/assura-code-doc-reference-validation.md` as Epic 5 of the
beta program. Start with a conservative, language-agnostic slice that reuses
existing Markdown link facts and adds bounded source/comment reference facts or
diagnostics without requiring LSP, hosted services, or broad semantic parsing.

## Scope

- Revalidate the goal against live Project Intelligence facts and current
  Markdown link behavior before implementation.
- Prefer extending existing fact ingestion/query paths over creating a parallel
  reference subsystem.
- Keep discovery conservative and local: comments, doc comments, docstrings,
  and obvious repository-relative string references only.
- Preserve source path/span, normalized target, anchor/line details, rule ID,
  target status, and confidence or reason where relevant.
- Add changed-source or changed-target affected-set proof when the first slice
  creates enough graph structure to support it.

## Non-Goals

- No full semantic code analysis.
- No remote link checking.
- No LSP, SCIP, or hosted indexer requirement.
- No automatic rewrite of ambiguous references.

## Validation

Use narrow checks while iterating:

```bash
cargo fmt --check
cargo test --test repository_reference_graph_tests --quiet
cargo test --test markdown_link_reference_tests --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
git diff --check
```

Before committing a meaningful implementation slice, also run:

```bash
cargo xtask evidence
cargo xtask target-state
```

## Review

Complex implementation slices require an independent reviewer before commit.
Ask the reviewer to focus on conservative discovery, reusable fact contracts,
span/target quality, and whether daemon/agent callers can consume bounded
reference context without new per-agent logic.
