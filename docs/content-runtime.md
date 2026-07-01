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
  events:
    class: Event
    path: events/*.jsonl
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

`models.validation_artifact` points to the runtime schema artifact. If
`models.source` or `models.validation_artifact` points inside `.assura/`, the
artifact must live under `.assura/models/**`; projects may still keep artifacts
outside `.assura/`, such as `schemas/**`, when that better fits their layout.
Collection entries bind object classes to paths, adapters, and stable ID
fields. Relation keys use `collection.field`. A relation may point at one
`target`, a bounded set of `targets`, or omit both to infer the target
collection from loaded objects. `required: true` reports absent or empty
reference fields, `many: true` expects an array of target IDs, and
`acyclic: true` rejects directed cycles for that relation.

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

## Supported Document Graph Contract

The supported document graph is the local, deterministic layer built from
checked repository files. It combines:

- modeled content instances from `collections`;
- configured relation edges from `relations`;
- Markdown section facts and keyword-search chunks;
- validation diagnostics from structure, Markdown, and content runtime checks;
- safe-fix preview facts where Assura can propose deterministic local edits;
- repository-reference edges from Markdown links, comments, docstrings, and
  simple string-literal path references.

The supported query path is:

```bash
assura check --format json .
assura content collections . --format json
assura content show <collection> <id> . --format json
assura content search "query text" . --format json
assura content missing-relations . --format json
assura content references . --target docs/guide.md --format json
assura content references . --source docs/guide.md --format json
assura content expand <collection> <id> . --format json
assura content context-pack . --collection <collection> --id <id> --text "query text" --limit 5 --format json
```

`assura content references` answers affected-reference questions in both
directions: inbound references to a target path before moving or deleting it,
and outbound references from a changed source path. Object-mode context packs
include bounded `repository_references.inbound` and
`repository_references.outbound` arrays for the modeled object's path so agents
can read direct doc/code reference context without running a second query.

Semantic search and code-symbol queries remain optional candidate enrichment.
They can add useful inspection hints, but they do not decide validation truth
and are not required for the supported document graph workflow.

## Fixture And Example Matrix

The checked fixture set is the executable example suite for this feature:

| Use case | Fixture or proof |
| --- | --- |
| Markdown frontmatter plus JSON records | `tests/fixtures/content_runtime/valid` |
| Invalid field value | `tests/fixtures/content_runtime/invalid_shape` |
| Missing reference | `tests/fixtures/content_runtime/missing_reference` |
| YAML records | `tests/fixtures/content_runtime/adapters/yaml/valid` |
| JSONL records | `tests/fixtures/content_runtime/adapters/jsonl/valid` |
| Required, optional, many, duplicate, ambiguous, and cyclic references | `tests/fixtures/content_runtime/references/` |
| Typed create operation | `tests/content_runtime_create.rs` |
| Typed update operation and Markdown body preservation | `tests/content_runtime_update.rs` |
| YAML and JSONL deterministic writes | `tests/content_runtime_adapters.rs` |
| Generated runtime schema artifact | `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/linkml_profile.runtime.schema.json` |
| Repository-reference graph facts | `tests/repository_reference_graph_tests.rs` |
| Bounded context packs with repository-reference context | `tests/project_intelligence_context_pack.rs` |

Use these commands to verify the examples from the repository:

```bash
cargo run --quiet -- check --format json tests/fixtures/content_runtime/valid
cargo run --quiet -- check --format json tests/fixtures/content_runtime/adapters/yaml/valid
cargo run --quiet -- check --format json tests/fixtures/content_runtime/adapters/jsonl/valid
cargo test --test content_runtime_create --quiet
cargo test --test content_runtime_update --quiet
cargo test --test content_runtime_adapters --quiet
cargo test --test content_runtime_references --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo test --test project_intelligence_context_pack --quiet
```

## Agent Operation Contract

Agents should call Assura-owned operations instead of editing content files
directly. The operation payload identifies the collection, stable object ID,
target path when creating, data fields, and optional Markdown body:

```json
{
  "operation": "create_record",
  "collection": "goals",
  "id": "goal-new",
  "path": "docs/goals/goal_new.md",
  "data": {
    "title": "New portable goal",
    "status": "planned",
    "specs": ["spec-portable-structure"]
  },
  "body": "# New Portable Goal\n\nThis body remains Markdown.\n"
}
```

Updates use the existing record identity and a field patch. Dry-run mode
returns the proposed content without writing:

```json
{
  "operation": "update_record",
  "collection": "specs",
  "id": "spec-portable-structure",
  "changes": {
    "status": "complete"
  },
  "dry_run": true
}
```

Before writing, Assura validates collection path policy, schema shape,
duplicate IDs, configured references, and write safety. Failed validation leaves
the tree unchanged. Markdown updates preserve the existing body bytes when only
frontmatter changes.

## Example Repo Objective

An implementation agent can independently create a language-agnostic example
repo with:

- `.assura/config.yml` defining `models`, `collections`, and `relations`;
- `.assura/models/project/runtime.schema.json` with `Goal` and `Spec`
  definitions;
- `docs/goals/*.md` files using YAML frontmatter plus Markdown bodies;
- `specs/*.json`, `*.yml`, or `*.jsonl` files for structured records;
- one valid case, one invalid field-value case, and one missing-reference case.

The example should prove that Assura can constrain project structure,
Markdown/frontmatter shape, and cross-file references without depending on the
language used by the project.

Minimum handoff for another agent:

1. Copy the config shape above into `.assura/config.yml`.
2. Check in a JSON Schema-compatible runtime artifact. If it lives under
   `.assura/`, put it under `.assura/models/**`.
3. Add one Markdown frontmatter goal and one JSON/YAML/JSONL spec with matching
   IDs.
4. Add one missing-reference record to prove diagnostics name source path,
   object type, field, and referenced object.
5. Exercise create and update through Assura operations, including a Markdown
   frontmatter update that proves the body bytes stay unchanged.
6. Document the normal validation command as `assura check --format json .`.

## Adoption Path

For an existing repo, adopt the content runtime incrementally:

1. Start with one small collection and one schema definition, such as
   Markdown-frontmatter goals linked to JSON specs.
2. Add `models.validation_artifact`, `collections`, and `relations` to
   `.assura/config.yml`.
3. Run `assura check --format json .` and fix shape/reference diagnostics until
   the initial collection is clean.
4. Add create/update operation tests around the agent workflow before allowing
   agents to write records.
5. Add YAML or JSONL collections only after the simpler Markdown/JSON path is
   stable.

Useful release-readiness diagnostics include:

- `content_runtime:invalid_object_shape` for field shape failures;
- `content_runtime:missing_reference` and
  `content_runtime:missing_reference_field` for relation failures;
- `content_runtime:duplicate_object_id` when a collection has conflicting IDs;
- `content_runtime:ambiguous_reference` when a relation can point at multiple
  target collections;
- `content_runtime:cyclic_reference` for configured acyclic relations;
- `content_runtime:invalid_object_path` and write-specific codes for bounded
  agent mutations.

For concrete inspection paths, see `docs/content-runtime-inspection.md`. It
shows the same model as Markdown frontmatter, JSON, and checked runtime schema
form, with TypeScript, Python, and Rust inspection guidance.

## Current Limits

The authoring decision is recorded in
`docs/analysis/2026-06-28-content-runtime-authoring-decision.md`: LinkML
profile first, TypeSpec fallback, checked JSON Schema-compatible runtime
artifacts in the validation hot path. Indexing, performance hardening, and
release-readiness docs are tracked in
`docs/goals/assura-repo-native-content-runtime-implementation.md`.

The first index/performance decision is recorded in
`docs/analysis/2026-06-28-content-runtime-index-performance.md`: Assura uses an
internal single-walk file index for collection matching and does not add Deeb
or another persistent cache dependency for normal validation.
