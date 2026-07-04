---
id: goal-assura-project-intelligence-repo-wide-code-content-search
type: goal
title: Assura project intelligence repo-wide code and content search
status: completed
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-simple-cli.md
  - docs/goals/assura-code-symbol-enrichment.md
  - docs/goals/assura-content-query-and-search-cli.md
---

# Assura Project Intelligence Repo-Wide Code And Content Search

## Post-Merge Revalidation

Closed as superseded on 2026-07-04. The original one-result-set repo-wide
search goal is not the next executable lane. Existing supported surfaces cover
content search, agent-query, context packs, repository references, and code
symbol facts separately; the remaining user need should be revalidated through
`docs/goals/assura-compact-project-review-common-issues.md` for first
diagnostics and
`docs/goals/assura-llm-wiki-personal-knowledge-base-starters.md` for modeled
content/query starter examples before a new unified-search command is proposed.

## Objective

Expand Project Intelligence search from modeled content facts to a repository
view that can return relevant code and content together, then move from a code
result to related content or from a content result to related code.

## Current Gap

Current content search indexes modeled instances, Markdown sections, and
diagnostics. Code-symbol commands can report modeled references, but a user
cannot search the whole repository and see code files, content records,
diagnostics, and relation paths in one result set.

## Scope

- Define the repo-wide index boundary for code, content, docs, diagnostics,
  model artifacts, and ignored/generated files.
- Add code-file search chunks with stable IDs, paths, language/source kind,
  and snippet boundaries.
- Reuse existing code-symbol facts to connect code chunks to modeled content
  where references exist.
- Support bidirectional traversal:
  - content result -> referenced code symbols and files;
  - code result -> content records that mention or model that symbol/file.
- Keep ignored, generated, vendored, and build-output paths out of the default
  index.
- Add fixture coverage for a realistic repo with app code, package code,
  modeled epics, ADRs, markdown sections, and invalid references.

## Non-Goals

- No full language server implementation.
- No remote code intelligence provider.
- No indexing of every binary or generated artifact.
- No semantic correctness claims from search ranking.

## Definition Of Done

- Repo-wide search returns both code and content results from one query path.
- Result JSON identifies source kind, path, score, snippet, and relation hints.
- Expansion can move from code to content and content to code when facts exist.
- Path ignore behavior is documented and tested.
- Performance remains bounded on the existing realistic fixtures.

## Validation Commands

```bash
cargo fmt --check
cargo test project_intelligence --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the default index boundary does not include generated or vendor
  noise.
- R2: Confirm code/content traversal uses facts and symbols, not ad hoc string
  guesses alone.
- R3: Confirm result ordering is deterministic.
- R4: Confirm large-repo performance risk is measured or explicitly bounded.

## Reviewer Blocking Criteria

Block if repo-wide search scans ignored/generated trees by default, cannot
explain why a code result relates to content, introduces remote dependencies,
or returns nondeterministic output.
