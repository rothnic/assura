---
title: Content Runtime And Models
description: Modeled repository collections over Markdown frontmatter, JSON, YAML, and JSONL files.
---

The content runtime makes ordinary repository files addressable as typed
objects. Content models define those objects, and the runtime validates them
while keeping the files as the source of truth.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Runtime schema artifact | Shipped | Assura loads checked-in runtime schema artifacts during `assura check`. |
| Markdown frontmatter adapter | Shipped | `markdown_frontmatter` validates frontmatter and preserves Markdown body bytes for updates. |
| JSON/YAML/JSONL adapters | Shipped | Structured records can share the same model and relation runtime. |
| Cross-collection relations | Shipped | Missing, duplicate, ambiguous, and cyclic references produce content-runtime diagnostics. |
| Agent create/update operations | Experimental | Assura-owned operations validate payloads before writing repository files. |

## Model A Repository Collection

```yaml config-fragment
models:
  source: .assura/models/project/source.linkml.yaml
  validation_artifact: .assura/models/project/runtime.schema.json

collections:
  goals:
    class: Goal
    path: docs/goals/*.md
    adapter: markdown_frontmatter
    data: frontmatter
    body: markdown
    id: id
```

The complete guide is in [Repo-Native Content Runtime](/examples/content-runtime/)
and the deeper repository document is `docs/content-runtime.md`.

When model artifacts live under `.assura/`, keep them under
`.assura/models/**`. Files outside `.assura/`, such as `schemas/**`, are still
valid project paths, but `.assura/` itself keeps a bounded root.

Typed frontmatter validation belongs to these content runtime models and
collections. Markdown rules can require a frontmatter block or heading shape,
but required typed fields, IDs, relations, and scoped object paths are validated
through the model-backed collection contract.
