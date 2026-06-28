---
title: Code Intelligence
description: Optional future code-symbol enrichment for project intelligence facts.
---

Code intelligence is optional enrichment. Assura must remain useful with
ordinary repository files, modeled content, Markdown validation, and local facts
before any code-symbol provider is required.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Native baseline code facts | Planned | Future work may add lightweight symbols without requiring external services. |
| Imported provider facts | Planned | Provider output may enrich the graph when available. |
| Symbol edges from modeled instances | Planned | Content records may link to code symbols after the fact model exists. |
| Required standalone code service | Unsupported | Core validation must not require CKB, LIP, Codanna, CQS, Glean, SCIP, LSP, or hosted services. |

Code intelligence should explain code context; it should not replace structure,
Markdown, content-model, or relation validation.
