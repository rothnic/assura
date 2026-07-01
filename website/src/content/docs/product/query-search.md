---
title: Query And Search
description: Content query, relation, graph expansion, keyword, semantic, and code-symbol search layers.
---

Query and search are Project Intelligence Runtime layers built on modeled
content collections, normalized facts, and local in-memory graph/search indexes.
The first supported command group is `assura content`, which exposes
deterministic collection, relation, keyword, and bounded graph queries for
agents and humans.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Collection queries | Supported | Lists modeled collections and instances from content runtime facts. |
| Relation queries | Supported | Traverses relation edges and reports missing targets. |
| Repository-reference queries | Supported | Reports inbound references to a target path and outbound references from a changed source path. |
| Keyword search | Supported | Searches indexed model-instance, Markdown-section, and diagnostic chunks with deterministic lexical scores. |
| Graph expansion | Supported | Expands from a model instance into bounded modeled relations, diagnostics, and related facts. Use `references` and object-mode context packs for repository-reference edges. |
| Context packs | Supported | Bundle diagnostics, model relations, search matches, repository-reference context, missing relations, and safe-fix previews for local agents. |
| Local semantic search | Experimental candidate enrichment | Optional local candidate retrieval only; it does not decide validation correctness. |
| Code-symbol queries | Experimental candidate enrichment | Shows configured model-to-symbol refs and symbol-to-model refs; unresolved refs remain visible. |

## Commands

Use JSON output for agent integrations:

```bash
assura content collections tests/fixtures/content_runtime/valid --format json
assura content agent-context tests/fixtures/content_runtime/code_symbols --format json
assura content instances goals tests/fixtures/content_runtime/valid --format json
assura content show goals goal-portable-structure tests/fixtures/content_runtime/valid --format json
assura content search "Portable Structure" tests/fixtures/content_runtime/valid --format json
assura content semantic-search "goal-portable-structure" tests/fixtures/content_runtime/valid --enable-local --format json
assura content symbols components component-config tests/fixtures/content_runtime/code_symbols --format json
assura content symbol-refs crate::sample::Config tests/fixtures/content_runtime/code_symbols --format json
assura content missing-relations tests/fixtures/content_runtime/missing_reference --format json
assura content expand goals goal-portable-structure tests/fixtures/content_runtime/valid --format json
assura content references tests/fixtures/content_runtime/code_symbols --target src/sample.rs --format json
assura content context-pack tests/fixtures/content_runtime/code_symbols --collection components --id component-config --limit 5 --format json
```

Use text output for quick terminal inspection:

```bash
assura content collections tests/fixtures/content_runtime/valid
assura content search "Portable Structure" tests/fixtures/content_runtime/valid
assura content semantic-search "goal-portable-structure" tests/fixtures/content_runtime/valid --enable-local
assura content symbols components component-config tests/fixtures/content_runtime/code_symbols
```

These commands build facts through the content runtime and project-intelligence
fact ingestor. They do not make search results validation truth; `assura check`
remains the validation command.

Repository-reference queries answer affected-path questions without requiring a
daemon or editor plugin. Use `--target <path>` before moving or deleting a file
to find inbound references, or `--source <path>` after editing a file to inspect
the paths it references. Object-mode context packs include bounded inbound and
outbound repository-reference arrays for the modeled object's path.

Keyword search scores are deterministic lexical ranking hints. Semantic search
is disabled unless `--enable-local` is present. The built-in local baseline
returns candidate facts with separate semantic scores, related context, and
diagnostics; no score decides validation correctness.

Code-symbol queries are also optional enrichment. The built-in Rust token
baseline can resolve rough local declarations, and missing providers preserve
unresolved refs instead of failing validation.

See [Project Intelligence Demo](/examples/project-intelligence-demo/) for a
visual walkthrough that connects modeled content, search, graph expansion,
missing-relation diagnostics, and agent-query envelopes.

## Boundaries

The beta-supported query layer is local and deterministic for modeled
collections, keyword search, relation queries, repository-reference queries,
bounded graph expansion, context packs, and JSON-line sessions. Semantic and
code-symbol outputs are candidate enrichment only. They do not provide
validation truth, remote embedding services, mandatory code-provider
enrichment, LSP, MCP, daemon, or long-running editor APIs.
