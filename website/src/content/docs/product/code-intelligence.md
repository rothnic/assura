---
title: Code Intelligence
description: Optional code-symbol enrichment for project intelligence facts.
---

Code intelligence is optional candidate enrichment. Assura must remain useful
with ordinary repository files, modeled content, Markdown validation, and local
facts; code-symbol providers are not required.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Native baseline code facts | Supported | `rust-token-baseline-v1` scans rough Rust declarations without external services. |
| Imported provider facts | Roadmap | Provider output may enrich the graph when available. |
| Symbol edges from modeled instances | Experimental candidate enrichment | Configured content fields create resolved or unresolved symbol references. |
| Symbol queries | Experimental candidate enrichment | `assura content symbols` and `assura content symbol-refs` expose model-to-symbol and symbol-to-model relationships as inspection hints. |
| Repository reference checks | Experimental | `extensions.repository_references` can report locally provable missing source/comment/docstring file targets, Markdown anchors, and line anchors during `assura check`. |
| Required standalone code service | Unsupported | Core validation must not require CKB, LIP, Codanna, CQS, Glean, SCIP, LSP, or hosted services. |

Code intelligence can explain code context; it does not replace structure,
Markdown, content-model, or relation validation, and it is not required for the
supported content collections/querying contract.

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

Source comments, docstrings, and string-like local paths can also be checked
without a provider:

```yaml
extensions:
  repository_references:
    - id: source_refs
      paths:
        - "src/**"
      severity: high
```
