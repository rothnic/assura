---
title: Content Runtime Inspection
status: active
---

# Content Runtime Inspection

This guide shows how a project can inspect Assura repo-native content models
without reading Assura Rust code or installing LinkML, TypeSpec, CUE, Node,
Python, Go, Deeb, SQLite, or a server for normal validation.

The checked example model is:

- Markdown frontmatter goal:
  `tests/fixtures/artifact_modeling_options/authoring_paths/fixtures/pass/docs/goals/goal_model_runtime.md`
- JSON spec:
  `tests/fixtures/artifact_modeling_options/authoring_paths/fixtures/pass/specs/spec_artifact_runtime.json`
- Checked runtime schema artifact:
  `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/linkml_profile.runtime.schema.json`
- Compile manifest:
  `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/selected_authoring_profile.compile.json`

The compile manifest records that `linkml_profile.runtime.schema.json` is the
selected runtime artifact, that the TypeSpec decorator path is the fallback,
and that runtime validation has an empty authoring-tool dependency list.

## Same Object In Files

The goal object is ordinary Markdown with YAML frontmatter as the typed record
and Markdown body as prose:

```markdown
---
id: goal_model_runtime
title: Model repository artifacts with native runtime validation
status: active
owners:
  - platform
specs:
  - spec_artifact_runtime
tasks:
  - task_profile_review
decisions:
  - decision_runtime_target
metadata:
  summary: Prove repository files can behave like typed content objects.
  risk: medium
  tags:
    - schema
    - runtime
---

The body stays as normal Markdown while the frontmatter carries the model data.
```

The spec object is ordinary JSON:

```json
{
  "id": "spec_artifact_runtime",
  "title": "Artifact runtime contract",
  "status": "active",
  "owner": "platform",
  "metadata": {
    "summary": "Shared JSON Schema contract generated from authoring models.",
    "risk": "low",
    "tags": ["json_schema", "rust"]
  },
  "decisions": ["decision_runtime_target"]
}
```

The runtime schema exposes the same object shape through JSON Schema `$defs`.
For example, `Goal` requires `id`, `title`, `status`, `owners`, and
`metadata`, while `specs`, `tasks`, and `decisions` are arrays of referenced
object IDs. Assura-specific runtime metadata stays under `x-assura` so storage
adapters and references remain inspectable:

```json
{
  "$defs": {
    "Goal": {
      "required": ["id", "title", "status", "owners", "metadata"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "specs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }
      }
    }
  },
  "x-assura": {
    "collections": [
      {
        "class": "Goal",
        "path": "docs/goals/*.md",
        "adapter": "markdown_frontmatter",
        "id": "id"
      },
      {
        "class": "Spec",
        "path": "specs/*.json",
        "adapter": "json_record",
        "id": "id"
      }
    ],
    "relations": [
      {
        "from": "Goal.specs",
        "to": "Spec.id",
        "many": true
      }
    ]
  }
}
```

## TypeScript Inspection

TypeScript users can inspect the checked runtime schema as JSON. They do not
need the TypeSpec fallback or an Assura TypeSpec package for normal validation.

Useful inspection targets:

- `"$defs"."Goal"."properties"` lists frontmatter fields accepted for goal
  records.
- `"$defs"."Spec"."properties"` lists JSON fields accepted for spec records.
- `"x-assura"."collections"` maps each class to the file pattern and adapter.
- `"x-assura"."relations"` maps reference fields such as `Goal.specs` to target
  IDs such as `Spec.id`.

JSON Schema-aware editors can use the checked runtime artifact for completion
and validation of JSON records. Markdown frontmatter records use the same
fields, with the Markdown body remaining outside the schema object.

## Python Inspection

Python and data-oriented projects can inspect the same checked JSON artifact
with ordinary JSON tooling. The LinkML source is useful authoring input, but it
is not needed for normal `assura check`.

The portable boundary is:

```text
checked runtime schema artifact
  -> Assura Rust validator cache
  -> Markdown frontmatter, JSON, YAML, or JSONL records
```

For scripts, read `$defs` to understand field shape and `x-assura` to
understand file placement and references. Do not treat the LinkML YAML source
as the runtime dependency; treat it as a build-time authoring source.

## Rust Inspection

Rust projects do not need derive macros or Cargo build scripts to define a
repo-native content model. Assura reads the same checked runtime artifact that
TypeScript and Python users inspect.

Rust consumers should treat the runtime schema as repository data:

- shape comes from JSON Schema `$defs`;
- placement comes from `x-assura.collections`;
- references come from `x-assura.relations`;
- normal validation is `assura check`, not a Rust compile step.

## Validation Commands

Validate the small content-runtime example repo with existing public Assura
commands:

```bash
assura check --format json tests/fixtures/content_runtime/valid
```

Check the missing-reference diagnostic path:

```bash
assura check --format json tests/fixtures/content_runtime/missing_reference
```

Inside the Assura repository, maintainers can run the same checks through Cargo
before the binary is installed:

```bash
cargo run --quiet -- check --format json tests/fixtures/content_runtime/valid
cargo run --quiet -- check --format json tests/fixtures/content_runtime/missing_reference
```

These commands exercise checked artifacts and ordinary files only. They do not
run LinkML, TypeSpec, CUE, Node, Python, Go, Deeb, SQLite, or a server during
normal validation.

The repository-only developer proof for authoring-path equivalence remains:

```bash
cargo test --test artifact_authoring_paths_proof --quiet
```

## Implementing An Example Repo

An implementation agent building a real example repo should create:

- `.assura/config.yml` with `models.validation_artifact`, `collections`, and
  `relations`;
- a runtime schema artifact checked into the repo, using `.assura/models/**`
  when the artifact lives under `.assura/`;
- `docs/goals/*.md` records with YAML frontmatter and Markdown bodies;
- `specs/*.json` records that use the same IDs referenced from goals;
- one passing case, one invalid-field case, and one missing-reference case;
- a README section that points TypeScript, Python, and Rust users to the same
  checked runtime artifact.

The example should prove that Assura constrains structure, Markdown
frontmatter, JSON records, and cross-file references in a language-agnostic
repo.

For a broader implementation checklist, including YAML, JSONL, create/update
writes, and Markdown body preservation, see `docs/content-runtime.md`.
