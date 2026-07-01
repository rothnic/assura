---
title: Project Intelligence Facts
status: active
---

# Project Intelligence Facts

Project intelligence facts are the storage-independent contract between current
Assura validation and later graph, search, semantic, code-symbol, daemon, LSP,
and MCP surfaces.

## Source Facts

Source facts are projected directly from repository files or Assura
configuration:

- `ModelDefinition`, `FieldDefinition`, `RelationshipDefinition`, and
  `PathScope` come from content runtime configuration and runtime schema
  artifacts.
- `Resource`, `MarkdownDocument`, `MarkdownSection`, `MarkdownLink`, and
  `ModelInstance` come from repository files or Markdown documents loaded by
  Assura ingestion paths.
- `CodeSymbol` and `CodeProviderEvidence` are optional code-intelligence facts
  from a native baseline or provider source. Core validation does not require
  code intelligence.

## Derived Facts

Derived facts are computed from source facts or validation output:

- `RelationshipEdge` connects model instances through configured content
  relations and may remain unresolved when a target is missing.
- `Diagnostic` records structure, Markdown, or content runtime validation
  findings with the most precise resource, field, line, and column Assura has.
- `SafeFix` records deterministic repair operations such as the Markdown
  blank-line trailing-whitespace fix.
- `SearchChunk`, `EmbeddingRecord`, and `SymbolRef` prepare search, semantic,
  and code-intelligence layers without making those providers required.
  `SymbolRef` can remain unresolved when a provider is missing or ambiguous.

`MarkdownLink` records source path, line, column, raw target, normalized
repository-relative target path, optional heading or line anchor details,
target existence at ingest time, and the related `markdown_link_*` validation
rule ID. This gives the later reference graph a stable outbound-edge source
without re-parsing Markdown independently.

`RepositoryReference` edges derive inbound and outbound repository-reference
context from source facts such as `MarkdownLink`. They carry source path/span,
target path, optional target anchor or line range, target existence, related
rule ID, reference kind, and confidence. Resolved edges point at `Resource`
facts for target paths so callers can ask which sources still refer to a file
before moving or deleting it.

## Code Symbol Evidence

Modeled collections can declare code-symbol reference fields in
`.assura/config.yml` using `code_symbols` entries keyed as `collection.field`.
The field value becomes a `SymbolRef` edge from the model instance. When the
configured provider has exactly one local match, the edge records a resolved
`target_id`; otherwise the unresolved edge remains queryable.

The built-in `rust-token-baseline-v1` provider is a no-dependency declaration
scan for rough local Rust context. Its `CodeSymbol` facts carry
`provider = "rust-token-baseline-v1"` and `evidence = "baseline"`, plus a
source location when available. Richer providers can add imported facts later,
but normal `assura check` and content validation must work without them.

## Replacement Semantics

Every fact and edge carries a generation label. A caller can replace all facts
from one generation with a new fact set without choosing a storage backend. This
keeps the contract usable for embedded stores, in-memory tests, future daemon
sessions, and one-shot CLI inspection.

Fact and edge IDs are deterministic strings derived from stable source keys such
as collection/class bindings, repository-relative paths, collection IDs,
relation fields, and diagnostic messages. Unchanged inputs should produce the
same IDs across runs.
