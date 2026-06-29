---
title: Repo-Native Content Runtime
description: Validate Markdown frontmatter, JSON, YAML, JSONL, references, and agent writes from ordinary repo files.
---

Assura can treat ordinary repository files as typed content objects while the
files remain the source of truth. A project checks in a runtime schema artifact,
declares collections in `.assura/config.yml`, and runs `assura check` without a
server, database, Rust build step, or authoring tool in the validation path.

## Model A Portable Project

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
  specs:
    class: Spec
    path: specs/*.json
    adapter: json_record
    id: id

relations:
  goals.specs:
    target: specs
    many: true
    required: true
```

The same runtime also supports `yaml_record` and `jsonl_record` adapters for
structured collections. Relation diagnostics identify the source file, object
type, field, and referenced object when available.

## Store Normal Files

Markdown frontmatter records keep prose as Markdown:

```markdown
---
id: goal-portable-structure
title: Portable structure policy
status: planned
specs:
  - spec-portable-structure
---

# Portable Structure Policy

The body remains ordinary Markdown.
```

JSON records use the same ID space:

```json
{
  "id": "spec-portable-structure",
  "title": "Portable structure and frontmatter",
  "status": "draft"
}
```

## Validate The Example

From the Assura repository, the checked fixtures exercise the public validation
path:

```bash
cargo run --quiet -- check --format json tests/fixtures/content_runtime/valid
cargo run --quiet -- check --format json tests/fixtures/content_runtime/adapters/yaml/valid
cargo run --quiet -- check --format json tests/fixtures/content_runtime/adapters/jsonl/valid
```

Installed users run the same shape against their repo:

```bash
assura check --format json .
```

## Let Agents Write Safely

Agents should use Assura-owned create and update operations instead of editing
frontmatter or JSON by hand. Create validates the payload, path policy, IDs, and
references before writing. Update can dry-run proposed content and preserves
Markdown body bytes when only frontmatter changes.

Repository proof lives in:

- `tests/content_runtime_create.rs`
- `tests/content_runtime_update.rs`
- `tests/content_runtime_adapters.rs`
- `tests/content_runtime_references.rs`

The full guide is in `docs/content-runtime.md`, with cross-language inspection
notes in `docs/content-runtime-inspection.md`.

For a visual walkthrough that connects modeled content to search, graph context,
agent envelopes, and safe-fix previews, see
[Project Intelligence Demo](/examples/project-intelligence-demo/).
