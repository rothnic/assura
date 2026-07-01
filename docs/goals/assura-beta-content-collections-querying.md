---
id: goal-assura-beta-content-collections-querying
type: goal
title: Assura beta content collections and querying
status: completed
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ./assura-content-model-source-of-truth.md
  - ./assura-content-query-and-search-cli.md
  - ./assura-project-intelligence-context-pack.md
  - ./assura-project-intelligence-persistent-session.md
  - ../project-intelligence-facts.md
---

# Assura Beta Content Collections And Querying

## Objective

Make frontmatter, Assura collections, validation, and querying beta-grade for
code-agnostic repository knowledge. A project should be able to model docs,
goals, ADRs, specs, and other records, validate them locally, and query them
through stable CLI and agent contracts.

## Current Gap

The content runtime and Project Intelligence surfaces are implemented and
locally proven, but beta needs a product contract that separates supported
collection modeling from experimental search, semantic candidates, and editor
or daemon wrappers.

## Scope

- Harden frontmatter model ownership and collection schema validation.
- Preserve `.assura/models/**` as the organized model directory.
- Validate required fields, optional fields, IDs, relations, and path scopes.
- Keep query, search, expand, missing-relation, context-pack, and session
  output stable enough for agents.
- Add docs showing one complete model-change validation workflow.
- Ensure diagnostics connect model errors, markdown sections, and related
  records without semantic search being treated as correctness.

## Non-Goals

- No hosted schema registry.
- No semantic search correctness claims.
- No authoring UI.
- No remote provider requirement.

## Definition Of Done

- A beta user can create, validate, query, and inspect modeled collections with
  copy/paste CLI commands.
- Invalid frontmatter, missing relation targets, and invalid scoped paths
  produce structured diagnostics.
- Context-pack and session outputs remain bounded and stable for agents.
- Docs distinguish supported collection validation from experimental ranking or
  semantic candidate behavior.

## Validation Commands

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test --test content_query_cli --quiet
cargo test project_intelligence --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm frontmatter schema validation has one source of truth.
- R2: Confirm querying is useful without requiring semantic search.
- R3: Confirm outputs are bounded enough for local agents.
- R4: Confirm docs show the model, invalid record, command, and diagnostic.

## Reviewer Blocking Criteria

Block if duplicate frontmatter validators reappear, semantic ranking decides
validation correctness, model artifacts leak into `.assura/` root, or agents
need unbounded context to use collection diagnostics.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Completed the beta closure slice for content collections and querying by validating the completed content-runtime, query/search, context-pack, and session child goals against the current support docs and release manifest. Added a distinct unreleased `content-collections-querying` release surface and tightened docs so model-backed collection validation/querying is the supported contract while semantic and code-symbol outputs remain candidate enrichment, not validation truth. Independent review Hypatia found two blockers: the persistent-session startup proof was timing out and code-symbol docs still marked candidate queries supported. The session proof now uses a bounded non-flaky ready wait, the stale docs are corrected, and target-state guards the support split. | [assura-content-model-source-of-truth.md](./assura-content-model-source-of-truth.md); [assura-content-query-and-search-cli.md](./assura-content-query-and-search-cli.md); [assura-project-intelligence-context-pack.md](./assura-project-intelligence-context-pack.md); [assura-project-intelligence-persistent-session.md](./assura-project-intelligence-persistent-session.md); `docs/data/release-surfaces.json`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `website/src/content/docs/product/content-models.md`; `website/src/content/docs/product/query-search.md`; `website/src/content/docs/product/code-intelligence.md`; `tests/project_intelligence_session.rs`; `cargo test --test project_intelligence_session --quiet`; `cargo xtask target-state`; independent review Hypatia. |
