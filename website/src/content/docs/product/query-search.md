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
| Keyword search | Supported | Searches indexed model-instance, Markdown-section, and diagnostic chunks. |
| Graph expansion | Supported | Expands from a model instance into bounded related facts. |
| Local semantic search | Supported | Optional local candidate retrieval only; it does not decide validation correctness. |
| Code-symbol queries | Supported | Shows configured model-to-symbol refs and symbol-to-model refs; unresolved refs remain visible. |

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

Semantic search is disabled unless `--enable-local` is present. The built-in
local baseline returns candidate facts with scores, related context, and
diagnostics; scores are ranking hints only.

Code-symbol queries are also optional enrichment. The built-in Rust token
baseline can resolve rough local declarations, and missing providers preserve
unresolved refs instead of failing validation.

## Boundaries

The current query layer is local and deterministic. It does not provide remote
embedding services, mandatory code-provider enrichment, LSP, MCP, daemon, or
long-running editor APIs yet.
