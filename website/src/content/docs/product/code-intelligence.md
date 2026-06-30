---
title: Code Intelligence
description: Optional code-symbol enrichment for project intelligence facts.
---

Code intelligence is optional enrichment. Assura must remain useful with
ordinary repository files, modeled content, Markdown validation, and local facts
before any code-symbol provider is required.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Native baseline code facts | Supported | `rust-token-baseline-v1` scans rough Rust declarations without external services. |
| Imported provider facts | Roadmap | Provider output may enrich the graph when available. |
| Symbol edges from modeled instances | Supported | Configured content fields create resolved or unresolved symbol references. |
| Symbol queries | Supported | `assura content symbols` and `assura content symbol-refs` expose model-to-symbol and symbol-to-model relationships. |
| Required standalone code service | Unsupported | Core validation must not require CKB, LIP, Codanna, CQS, Glean, SCIP, LSP, or hosted services. |

Code intelligence should explain code context; it should not replace structure,
Markdown, content-model, or relation validation.

## Configuration

Declare symbol-bearing fields with `code_symbols` keys:

```yaml
code_symbols:
  components.implementation:
    provider: rust-token-baseline-v1
```

When a provider is unavailable or cannot resolve a unique target, Assura keeps
the unresolved `SymbolRef`. That preserves the relationship for agents without
turning code intelligence into validation truth.
