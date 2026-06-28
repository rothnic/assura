---
title: Repo-Native Content Runtime
status: active
---

# Repo-Native Content Runtime

Assura can model ordinary repository files as typed content objects while the
files remain the canonical state. The first runtime slice supports Markdown
frontmatter records and JSON records loaded from project config and checked
against a JSON Schema-compatible runtime artifact.

This surface is intentionally portable. A project does not need Rust, a server,
LinkML, TypeSpec, Deeb, SQLite, Node, Python, Go, or CUE at validation time.
Authoring tools may generate the runtime schema artifact, but the runtime
validation path consumes checked-in files. `assura check` report integration is
tracked as the next increment.

## Config Shape

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
```

`models.validation_artifact` points to the runtime schema artifact. Collection
entries bind object classes to paths, adapters, and stable ID fields. Relation
keys use `collection.field` and point at a target collection.

## Runtime Contract

- Markdown collections parse YAML frontmatter as object data and preserve the
  Markdown body separately for future write operations.
- JSON collections parse each matched file as one object.
- Runtime validators are compiled in Rust from the checked-in schema artifact.
- Reference validation resolves configured relation fields across loaded
  collections.
- Diagnostics carry source path, object type, field, and referenced object when
  that context is available.

## Example Repo Objective

An implementation agent can independently create a language-agnostic example
repo with:

- `.assura/config.yml` defining `models`, `collections`, and `relations`;
- `schemas/project.runtime.schema.json` with `Goal` and `Spec` definitions;
- `docs/goals/*.md` files using YAML frontmatter plus Markdown bodies;
- `specs/*.json` files for structured spec records;
- one valid case, one invalid field-value case, and one missing-reference case.

The example should prove that Assura can constrain project structure,
Markdown/frontmatter shape, and cross-file references without depending on the
language used by the project.

## Current Limits

This slice validates read paths only. CLI report integration, typed create and
update operations, YAML and JSONL adapters, authoring-tool generation, indexing,
performance hardening, and release-readiness docs are tracked in
`docs/goals/assura-repo-native-content-runtime-implementation.md`.
