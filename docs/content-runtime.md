---
title: Repo-Native Content Runtime
status: active
---

# Repo-Native Content Runtime

Assura can model ordinary repository files as typed content objects while the
files remain the canonical state. The runtime supports Markdown frontmatter,
JSON, YAML, and JSONL-backed records loaded from project config and checked
against a JSON Schema-compatible runtime artifact.

This surface is intentionally portable. A project does not need Rust, a server,
LinkML, TypeSpec, Deeb, SQLite, Node, Python, Go, or CUE at validation time.
Authoring tools may generate the runtime schema artifact, but the runtime
validation path consumes checked-in files. `assura check` reports content
runtime diagnostics with source path, object type, field, and referenced object
context when available.

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
  decisions:
    class: Decision
    path: decisions/*.jsonl
    adapter: jsonl_record
    id: id

relations:
  goals.specs:
    target: specs
    many: true
    required: true
  events.related:
    targets:
      - goals
      - specs
  events.parent:
    target: events
    acyclic: true
```

`models.validation_artifact` points to the runtime schema artifact. Collection
entries bind object classes to paths, adapters, and stable ID fields. Relation
keys use `collection.field`. A relation may point at one `target`, a bounded
set of `targets`, or omit both to infer the target collection from loaded
objects. `required: true` reports absent or empty reference fields, `many: true`
expects an array of target IDs, and `acyclic: true` rejects directed cycles for
that relation.

## Runtime Contract

- Markdown collections parse YAML frontmatter as object data and preserve the
  Markdown body separately for write operations.
- JSON collections parse each matched file as one object.
- YAML collections parse each matched file as one object and write normalized
  YAML with deterministic key ordering. Comments and original scalar styling are
  not preserved.
- JSONL collections parse each non-empty line as one object. Create and update
  operations rewrite the JSONL file as compact one-object-per-line JSON sorted
  by object ID, preserving unrelated records as records but not preserving
  original line order or whitespace.
- Runtime validators are compiled in Rust from the checked-in schema artifact.
- Reference validation resolves configured relation fields across loaded
  collections, including required, optional, many, and multi-target relations.
- Diagnostics carry source path, object type, field, and referenced object when
  that context is available.

## Example Repo Objective

An implementation agent can independently create a language-agnostic example
repo with:

- `.assura/config.yml` defining `models`, `collections`, and `relations`;
- `schemas/project.runtime.schema.json` with `Goal` and `Spec` definitions;
- `docs/goals/*.md` files using YAML frontmatter plus Markdown bodies;
- `specs/*.json`, `*.yml`, or `*.jsonl` files for structured records;
- one valid case, one invalid field-value case, and one missing-reference case.

The example should prove that Assura can constrain project structure,
Markdown/frontmatter shape, and cross-file references without depending on the
language used by the project.

## Current Limits

Authoring-tool generation, indexing, performance hardening, and
release-readiness docs are tracked in
`docs/goals/assura-repo-native-content-runtime-implementation.md`.
