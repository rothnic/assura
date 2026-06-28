---
title: Content Models
description: Modeled repository collections over Markdown frontmatter, JSON, YAML, and JSONL files.
---

Content models make ordinary repository files addressable as typed objects.
They build on structure and Markdown validation while keeping the files as the
source of truth.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Runtime schema artifact | Shipped | Assura loads checked-in runtime schema artifacts during `assura check`. |
| Markdown frontmatter adapter | Shipped | `markdown_frontmatter` validates frontmatter and preserves Markdown body bytes for updates. |
| JSON/YAML/JSONL adapters | Shipped | Structured records can share the same model and relation runtime. |
| Cross-collection relations | Shipped | Missing, duplicate, ambiguous, and cyclic references produce content-runtime diagnostics. |
| Agent create/update operations | Experimental | Assura-owned operations validate payloads before writing repository files. |

## Model A Repository Collection

```yaml
models:
  source: schemas/project.linkml.yaml
  validation_artifact: schemas/project.runtime.schema.json

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
