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
| Repository-reference queries | Supported | Reports inbound references, outbound references, all reference edges, and unresolved reference edges. Configured frontmatter path fields are included. |
| Keyword search | Supported | Searches indexed model-instance, Markdown-section, and diagnostic chunks with deterministic lexical scores. Explicit raw repository fallback is available for day-one discovery before models are active. |
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
assura content search "RawAlphaTerm" . --raw --format json
assura content search "RawAlphaTerm" . --fallback-raw --format json
assura content semantic-search "goal-portable-structure" tests/fixtures/content_runtime/valid --enable-local --format json
assura content symbols components component-config tests/fixtures/content_runtime/code_symbols --format json
assura content symbol-refs crate::sample::Config tests/fixtures/content_runtime/code_symbols --format json
assura content missing-relations tests/fixtures/content_runtime/missing_reference --format json
assura content expand goals goal-portable-structure tests/fixtures/content_runtime/valid --format json
assura content references tests/fixtures/content_runtime/code_symbols --target src/sample.rs --format json
assura content references . --all --format json
assura content references . --unresolved --format json
assura content agent-query capabilities . --format json
assura content agent-query unresolved-references . --format json
assura content agent-query gaps . --format json
assura content agent-query next-actions . --format json
assura content context-pack tests/fixtures/content_runtime/code_symbols --collection components --id component-config --limit 5 --format json
```

Use text output for quick terminal inspection:

```bash
assura content collections tests/fixtures/content_runtime/valid
assura content search "Portable Structure" tests/fixtures/content_runtime/valid
assura content search "RawAlphaTerm" . --fallback-raw
assura content semantic-search "goal-portable-structure" tests/fixtures/content_runtime/valid --enable-local
assura content symbols components component-config tests/fixtures/content_runtime/code_symbols
```

These commands build facts through the content runtime and project-intelligence
fact ingestor. They do not make search results validation truth; `assura check`
remains the validation command.

Repository-reference queries answer affected-path questions without requiring a
daemon or editor plugin. Use `--target <path>` before moving or deleting a file
to find inbound references, `--source <path>` after editing a file to inspect
the paths it references, `--all` to enumerate bounded graph edges, or
`--unresolved` to list edges whose target path is missing. Object-mode context
packs include bounded inbound and outbound repository-reference arrays for the
modeled object's path.

When `extensions.repository_references[].frontmatter_fields` is configured,
Markdown frontmatter string or list fields such as `source_documents`,
`related`, `evidence`, or `requirements` become repository-reference edges.
They appear in `content references`, context packs, and agent-query unresolved
reference output with `reference_kind: frontmatter_reference`.

Raw text search is an explicit local fallback, not a semantic ranking claim.
Use `assura content search <query> --raw` to search repository text directly, or
`--fallback-raw` to search raw text only when modeled fact search has no
matches. Raw results are bounded and deterministic, with path and line
metadata for agent follow-up.

`assura content agent-query capabilities` lists deterministic agent-query
capabilities, required arguments, and suggested follow-up commands. The
`unresolved-references`, `gaps`, and `next-actions` capabilities give agents a
machine-readable way to discover broken references and the next command to run
without inventing a natural-language query parser.

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
collections, keyword search, explicit raw text fallback, relation queries,
repository-reference queries, bounded graph expansion, context packs,
agent-query capability discovery, and JSON-line sessions. Semantic and
code-symbol outputs are candidate enrichment only. They do not provide
validation truth, remote embedding services, mandatory code-provider
enrichment, LSP, MCP, daemon, or long-running editor APIs.
