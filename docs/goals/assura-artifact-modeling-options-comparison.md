---
id: goal-assura-artifact-modeling-options-comparison
type: goal
title: Artifact modeling options comparison
status: completed
created: 2026-06-27
owners:
  - assura-maintainers
related:
  - .trellis/tasks/06-20-prototype-repo-native-content-object-model/prd.md
  - .trellis/tasks/06-20-prototype-repo-native-content-object-model/research/prototype-baseline-results.md
  - .trellis/spec/assura/config-notation.md
---

# Artifact Modeling Options Comparison

## Objective

Choose an evidence-backed modeling source for Assura repo artifacts by building
small comparable MVPs instead of judging from docs or taste.

The goal is to decide how developers should define artifact models such as
`Goal`, `Spec`, `Task`, and `Decision` once, then bind those models to stored
representations such as Markdown frontmatter, JSON records, YAML records, or
JSONL rows.

## Current Gap

The current prototype proves that Assura-owned file adapters, placement rules,
reference validation, and safe writes are needed. It does not prove whether the
artifact model source should be LinkML, TypeSpec, CUE, JSON Schema/JTD, a
TypeScript schema tool, or a small Assura-owned format.

Assura should not invent a schema language unless tested alternatives fail
against explicit developer-experience and runtime criteria.

## Success Criteria

- A developer can inspect the model files and understand the core artifact
  types without learning an obscure schema system first.
- The model source can express required fields, optional fields, enums, arrays,
  identifiers, references, and simple constraints for repo artifacts.
- The model source can generate or compile to JSON Schema or another stable
  Rust-validatable artifact.
- Assura can validate Markdown frontmatter and JSON records as the same logical
  artifact class.
- The approach supports useful IDE feedback: syntax highlighting, completion,
  diagnostics, formatting, schema validation, or language-server support.
- Python and TypeScript users can consume generated types or clear generated
  schemas without treating Rust internals as the source of truth.
- Rust runtime validation is fast enough to compile/cache once per config and
  validate changed files without invoking Node or Python in the hot path.
- The end-user workflow works after installing Assura's compiled binary. Node,
  Python, Go, or other toolchains may help author or regenerate artifacts, but
  must not be required for normal validation, transformation, or agent update
  workflows.
- The approach works well for Rust and non-Rust repositories. Assura must not
  make Rust source types, Cargo, or a Rust build step the model source for
  Python, TypeScript, documentation, or mixed-language projects.
- The model can support agents generating or modifying artifacts through a typed
  CLI/API contract.
- The resulting Assura config remains clear about the split between artifact
  model, collection path, storage adapter, and repo placement.

## DX Concerns To Score

Score each option from 1 to 5 for:

- Readability: can a normal Python/TypeScript/Rust developer understand it?
- Authoring friction: how much boilerplate is required for a small model?
- IDE support: completion, diagnostics, formatting, navigation, or schemas.
- Ecosystem leverage: generators for JSON Schema, docs, Python, TypeScript,
  Rust, OpenAPI, SQL, linked data, or diagrams.
- Constraint expressiveness: fields, enums, unions, references, identifiers,
  inheritance/composition, and simple value constraints.
- Runtime fit: can Assura validate in Rust without a server or hot-path
  subprocess?
- Native Assura performance: can schemas/transforms be compiled or cached so
  installed Assura binaries perform validation and writes without external
  toolchains?
- Cross-language project fit: does the model source work naturally for Rust,
  Python, TypeScript, docs, and mixed-language repositories?
- Storage neutrality: does it model artifacts independently from Markdown,
  JSON, YAML, JSONL, or CSV storage?
- Agent usability: can an agent infer the model and produce valid artifacts?
- Maintenance risk: maturity, project velocity, dependency weight, and lock-in.
- Migration path: can Assura adopt it incrementally without breaking current
  structure notation?

## Candidate MVPs

Build the same real-world example with each major option:

- TypeSpec
- LinkML
- CUE
- JSON Schema or JSON Type Definition
- TypeScript schema option: Zod or TypeBox
- Minimal Assura-owned model only as a control, not as the presumed answer

Each MVP must model:

- `Goal`: `id`, `title`, `status`, `owners`, `specs`
- `Spec`: `id`, `title`, `status`
- `Decision`: `id`, `title`, `status`, `supersedes`
- references from `Goal.specs` to `Spec`
- at least one enum and one optional field

Each MVP must bind the same logical artifacts to:

- `docs/goals/*.md` as Markdown body plus frontmatter data
- `specs/*.json` as JSON records
- one JSONL or YAML collection if the option handles it cleanly

## Required Prototype Proof

For each candidate:

- Author the model in the candidate format.
- Show the Assura collection binding syntax needed to map files to artifact
  classes and adapters.
- Generate the validation artifact, preferably JSON Schema.
- Validate the same passing and failing fixture set in Rust.
- Demonstrate at least one safe update or explain why write support is blocked.
- Record the generated artifacts and command sequence.
- Capture what the developer sees in an editor, CLI, or generated docs.

## Score Sheet

Create a single comparison table with:

- category scores from the DX concerns above;
- a weighted total;
- clear disqualifiers, if any;
- notes on what small extension could materially improve the score;
- recommendation for primary model source, fallback, and rejected options.

Default weights:

- Developer readability and authoring: 15%
- IDE/tooling and ecosystem leverage: 15%
- Native Assura runtime performance and single-binary operation: 30%
- Cross-language project fit and storage-neutral artifact modeling: 25%
- References and constraints: 10%
- Agent usability and maintainability: 5%

These weights intentionally make runtime performance and cross-language fit
more important than authoring taste. The desired user workflow is installing
Assura's compiled binary and using it for validation, transformation, safe
writes, and agent workflows across Rust, Python, TypeScript, documentation, and
mixed-language repositories without Node, Python, Go, or a server in the
normal hot path. Adjust weights only if the goal records why.

## Independent Review

Use at least one independent review pass per serious candidate. Reviewers must
score the candidate against the same sheet and identify whether a manageable
extra prototype would change the result.

Reviewer blocking criteria:

- Block if candidates are scored from documentation only.
- Block if examples differ enough that scores are not comparable.
- Block if a preferred option cannot validate both Markdown frontmatter and JSON
  records as the same logical artifact class.
- Block if runtime validation requires Node, Python, or a server in the hot
  path without a compiled/cached Rust validation alternative.
- Block if the recommendation ignores IDE/DX evidence.

## Definition Of Done

- Comparable MVPs exist for at least TypeSpec, LinkML, CUE, and one JSON
  Schema/JTD-oriented option.
- Every MVP uses the same artifact classes, fixtures, references, and storage
  bindings.
- Rust validation runs against generated or compiled validation artifacts.
- The score sheet includes category scores, weighted totals, and reviewer notes.
- The final recommendation names the winning path, fallback path, rejected
  paths, and the next implementation goal.
- The final recommendation includes concrete examples of model files,
  collection bindings, generated validation artifacts, and validation output.

## Progress Log

### 2026-06-27

- Created comparable artifact-model sketches for TypeSpec, LinkML, CUE, JSON
  Schema, Zod, and an Assura-owned control format under
  `tests/fixtures/artifact_modeling_options/`.
- Added shared collection binding and pass/fail fixtures covering Markdown
  frontmatter, JSON records, YAML records, enum validation, required fields, and
  `Goal.specs -> Spec.id` reference checks.
- Added Rust proof command
  `cargo test --test artifact_modeling_options_comparison --quiet`; current
  result is `5 passed`.
- Updated scoring weights to prioritize native Assura runtime performance,
  single-binary operation, cross-language fit, and storage-neutral modeling.
- Ran independent review passes. Reviewers agreed the normalized runtime proof
  is useful but the recommendation must remain provisional until a production
  slice proves generated artifacts, editor DX, and relation metadata in a real
  Assura config path.
- Added authoring-time generation probes for TypeSpec, LinkML, CUE, and Zod;
  recorded results in
  `docs/analysis/2026-06-27-artifact-modeling-options-comparison.md`.
- Added native safe-update proof for Markdown frontmatter: preserve identity,
  revalidate, write, and reject an attempted `id` change without Node, Python,
  Go, CUE, or a server in the hot path.
- Added checked-in generated-output snapshots under
  `tests/fixtures/artifact_modeling_options/generated_outputs/` so the
  generator evidence does not depend on conversation memory or transient
  terminal output.
- Added a requirement audit to
  `docs/analysis/2026-06-27-artifact-modeling-options-comparison.md` mapping
  the goal's explicit deliverables to current evidence.
- Addressed final independent-review blockers by validating generated/compiled
  artifacts directly in Rust, adding a cached native-validation loop, and
  preserving reviewer IDs, per-candidate coverage, findings, and resolutions in
  `docs/analysis/2026-06-27-artifact-modeling-options-review-record.md`.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
git diff --check
```

Candidate prototypes should add their own reproducible commands in a
`docs/analysis/` artifact or task research directory.

## Non-Goals

- Do not implement the production Assura artifact-model feature in this goal.
- Do not choose a model source without executable proof.
- Do not build a CMS UI.
- Do not require a running server.
- Do not make Deeb, SQLite, or any storage backend the artifact model source.
