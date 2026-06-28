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
- `Resource`, `MarkdownDocument`, `MarkdownSection`, and `ModelInstance` come
  from repository files loaded by content runtime adapters.
- `CodeSymbol` is optional and can be imported later from a native or provider
  source; core validation does not require it.

## Derived Facts

Derived facts are computed from source facts or validation output:

- `RelationshipEdge` connects model instances through configured content
  relations and may remain unresolved when a target is missing.
- `Diagnostic` records structure, Markdown, or content runtime validation
  findings with the most precise resource, field, line, and column Assura has.
- `SafeFix` records deterministic repair operations such as the Markdown
  blank-line trailing-whitespace fix.
- `SearchChunk`, `EmbeddingRecord`, and `SymbolRef` prepare later search,
  semantic, and code-intelligence layers without making those providers
  required.

## Replacement Semantics

Every fact and edge carries a generation label. A caller can replace all facts
from one generation with a new fact set without choosing a storage backend. This
keeps the contract usable for embedded stores, in-memory tests, future daemon
sessions, and one-shot CLI inspection.

Fact and edge IDs are deterministic strings derived from stable source keys such
as collection/class bindings, repository-relative paths, collection IDs,
relation fields, and diagnostic messages. Unchanged inputs should produce the
same IDs across runs.
