---
id: analysis-2026-06-27-artifact-modeling-options-comparison
type: analysis
title: Artifact modeling options comparison
status: active
created: 2026-06-27
owners:
  - assura-maintainers
related:
  - docs/goals/assura-artifact-modeling-options-comparison.md
  - docs/analysis/2026-06-27-artifact-modeling-options-review-record.md
  - tests/artifact_modeling_options_comparison.rs
  - tests/fixtures/artifact_modeling_options/
---

# Artifact Modeling Options Comparison

## Summary

The strongest path to prototype next is a standards-based split: **LinkML as
the current leading authoring candidate, TypeSpec as the close DX fallback, and
JSON Schema/JTD-like compiled artifacts as the runtime validation target**.

The important product constraint is not "which tool can validate data." Assura
needs users to install one compiled binary that can validate, transform, and
write repo artifacts across Rust and non-Rust projects without requiring Node,
Python, Go, CUE, or a server in the hot path.

The authoring source and runtime target should be treated as different layers.
JSON Schema or JTD is the best Rust runtime target, but it is not the best
human-facing model syntax. LinkML currently leads the next-prototype score
because it has the strongest cross-language and relationship-modeling story
while still allowing Assura to validate generated artifacts natively. The
evidence does not justify permanently choosing LinkML yet; TypeSpec is close
enough that the next production slice should keep the model-source interface
open.

## Prototype Matrix

The comparison uses one real artifact domain across all candidates:

- `Goal`: `id`, `title`, `status`, `owners`, `specs`
- `Spec`: `id`, `title`, `status`
- `Decision`: `id`, `title`, `status`, `supersedes`
- `Goal.specs` references `Spec.id`

Artifacts and fixtures:

- TypeSpec model: `tests/fixtures/artifact_modeling_options/models/typespec/project.tsp`
- LinkML model: `tests/fixtures/artifact_modeling_options/models/linkml/project.linkml.yaml`
- CUE model: `tests/fixtures/artifact_modeling_options/models/cue/project.cue`
- JSON Schema model: `tests/fixtures/artifact_modeling_options/models/json_schema/artifacts.schema.json`
- Zod model: `tests/fixtures/artifact_modeling_options/models/zod/project.schema.ts`
- Assura control model: `tests/fixtures/artifact_modeling_options/models/assura_control/project.model.yml`
- Collection binding sketch: `tests/fixtures/artifact_modeling_options/bindings/assura_collections.yml`
- Runtime schemas: `tests/fixtures/artifact_modeling_options/schemas/*.artifacts.schema.json`
- Generated-output snapshots: `tests/fixtures/artifact_modeling_options/generated_outputs/`
- Fixtures: `tests/fixtures/artifact_modeling_options/fixtures/`

The runtime proof validates Markdown frontmatter, JSON records, YAML records,
and cross-collection references through one Rust integration test:

```bash
cargo test --test artifact_modeling_options_comparison --quiet
```

Result:

```text
7 passed; 0 failed
```

The checked-in schema files are intentionally identical normalized runtime
artifacts. They prove Assura can validate the same logical object model across
Markdown frontmatter, JSON records, YAML records, and references in native
Rust. The test also validates the generated or compiled artifacts in
`generated_outputs/` and model source directories, normalizing expected schema
dialect differences in native Rust before fixture validation.

## Source Evidence

- TypeSpec provides JSON Schema emitter support, including `emitAllModels` and
  bundle options for a single schema document:
  https://typespec.io/docs/emitters/json-schema/reference/
- LinkML generators transform schemas into JSON Schema and other artifacts:
  https://linkml.io/linkml/generators/index.html
- LinkML JSON Schema generation can be run with `gen-json-schema`:
  https://linkml.io/linkml/generators/json-schema.html
- LinkML documents Rust crate generation with Python bindings:
  https://linkml.io/linkml/generators/rust.html
- CUE has first-class support for JSON Schema conversion:
  https://cuelang.org/docs/concept/how-cue-works-with-json-schema/
- Zod 4 supports native JSON Schema conversion:
  https://zod.dev/json-schema
- TypeSpec documents a VS Code extension for authoring support:
  https://typespec.io/docs/introduction/editor/vscode/
- LinkML documents schema-aware YAML editing via its JSON Schema metamodel:
  https://linkml.io/linkml/faq/tools.html
- VS Code documents JSON Schema support for JSON editing, with draft caveats:
  https://code.visualstudio.com/docs/languages/json

## Generation And Runtime Evidence

The generator commands below were run as authoring-time probes only. None are
required in Assura's normal validation or safe-write hot path.

### TypeSpec

Command:

```bash
npm install @typespec/compiler @typespec/json-schema
npx tsp compile main.tsp \
  --emit @typespec/json-schema \
  --option @typespec/json-schema.emitAllModels=true \
  --option @typespec/json-schema.bundleId=artifacts
```

Observed result: compile succeeded and emitted a bundled schema with
`$defs.Goal`, `$defs.Spec`, `$defs.Decision`, `minLength`, `minItems`, and
required fields. The generated status unions use `anyOf` plus `const`, so
Assura would normalize them to the runtime enum shape. TypeSpec still needs
Assura-owned relation metadata for `Goal.specs -> Spec.id`.

Snapshot:
`tests/fixtures/artifact_modeling_options/generated_outputs/typespec_artifacts.schema.yaml`

### LinkML

Command:

```bash
uvx --from linkml gen-json-schema \
  tests/fixtures/artifact_modeling_options/models/linkml/project.linkml.yaml
```

Observed result: generation succeeded. The updated LinkML sketch now exposes
the user-facing `status` field and uses `slot_usage` for class-specific status
enums. Output included `$defs`, enum references, required fields, string
patterns, and `owners.minItems`. LinkML's array item constraints are less exact
than the normalized runtime schema, so Assura would still need a small profile
and normalization pass.

Snapshot:
`tests/fixtures/artifact_modeling_options/generated_outputs/linkml_artifacts.schema.json`

### CUE

Command:

```bash
go run cuelang.org/go/cmd/cue@latest export \
  --out jsonschema -e '#Goal' project.cue
```

Observed result: per-class export succeeded, but the output shape used CUE-ish
JSON Schema details such as `not: { const: "" }` and a list constraint that
needs normalization. A naive bundled export produced only a generic object
schema. CUE remains viable only as an authoring-time source for a strict
JSON-Schema-emittable subset; CUE/Go cannot be in Assura's runtime hot path.

Snapshot:
`tests/fixtures/artifact_modeling_options/generated_outputs/cue_goal.schema.json`

### Zod

Command:

```bash
npm install zod typescript tsx
npx tsx -e 'import { z } from "zod"; import { Goal, Spec, Decision } from "./project.schema.ts"; console.log(JSON.stringify({ Goal: z.toJSONSchema(Goal), Spec: z.toJSONSchema(Spec), Decision: z.toJSONSchema(Decision) }, null, 2));'
```

Observed result: generation succeeded and produced clean JSON Schema for each
class. Once checked in or compiled, the runtime path is native Assura like the
other options. The concern is source-of-truth fit: Zod makes TypeScript central
for mixed Rust, Python, documentation, and data repositories.

Snapshot:
`tests/fixtures/artifact_modeling_options/generated_outputs/zod_artifacts.schema.json`

### JSON Schema/JTD And Assura Control

Direct JSON Schema needs no generator and is the best runtime target, but it is
too noisy as the primary authoring syntax. The Assura-owned control remains a
useful benchmark for minimum notation size, but it has no external generator,
IDE, documentation, or cross-language ecosystem unless Assura builds one.

DX evidence summary:
`tests/fixtures/artifact_modeling_options/generated_outputs/dx_evidence.md`

## Safe Update Evidence

The Rust proof includes a native safe-update test. It copies a Markdown
frontmatter goal fixture, updates `title`, preserves `id`, revalidates the
updated frontmatter against every normalized candidate schema, writes the file
back, and rejects an attempted `id` change.

Command:

```bash
cargo test --test artifact_modeling_options_comparison --quiet
```

Observed result:

```text
7 passed; 0 failed
```

## Native Runtime Evidence

The Rust proof includes a cached native-validation loop. It loads each
candidate's generated or compiled schema artifact once, then validates the
passing fixture 200 times per candidate without invoking Node, Python, Go, CUE,
or a server. The test has a conservative 5 second ceiling and the current
artifact comparison test completed in 0.34 seconds on this machine.

This is not a production benchmark. It is enough for this comparison goal to
prove the selected architecture can keep authoring toolchains out of Assura's
normal validation and safe-write hot path.

## Scoring

Scores are 1 to 5. Weighted total is out of 5.

Weights:

- Developer readability and authoring: 15%
- IDE/tooling and ecosystem leverage: 15%
- Native Assura runtime performance and single-binary operation: 30%
- Cross-language project fit and storage-neutral artifact modeling: 25%
- References and constraints: 10%
- Agent usability and maintainability: 5%

| Option | Readability and authoring 15% | IDE/tooling and ecosystem 15% | Native Assura runtime 30% | Cross-language and storage-neutral 25% | References and constraints 10% | Agent/maintenance 5% | Weighted total | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| LinkML | 3 | 5 | 4 | 5 | 5 | 3 | 4.30 | Preferred authoring candidate |
| JSON Schema/JTD | 2 | 5 | 5 | 5 | 3 | 3 | 4.25 | Runtime target, poor authoring |
| TypeSpec | 5 | 5 | 4 | 4 | 3 | 4 | 4.20 | DX fallback and strong challenger |
| Assura-owned control | 4 | 2 | 5 | 4 | 3 | 2 | 3.80 | Control only; avoid unless standards fail |
| CUE | 3 | 4 | 3 | 4 | 5 | 3 | 3.60 | Strong constraints, weaker mainstream DX |
| Zod/TypeBox | 5 | 4 | 4 | 2 | 3 | 4 | 3.55 | Great TS DX, too TS-centered |

The scoring gives 55% of the total to native runtime behavior plus
cross-language, storage-neutral modeling. That is deliberate: Assura's product
promise is not a prettier schema file, it is fast native validation and writes
for many repository types after installing one binary.

## Candidate Notes

### LinkML

Best semantic modeling ecosystem. It has the most direct story for linked data,
documentation, diagrams, JSON Schema, Python, TypeScript, Rust, SQL-ish, and
data-dictionary workflows. It is also the closest fit for modeling repository
objects and references independently from how each object is stored on disk.

Risk: authoring UX is YAML-heavy and more specialized. Assura should avoid
exposing full LinkML complexity at first. The likely product shape is a small
documented LinkML profile plus Assura-owned collection bindings, not a new
schema language.

### JSON Schema / JTD

Best runtime and editor validation target. It should be part of the Assura
compiled artifact story because Rust can load, cache, and validate it without
external toolchains.

Risk: authoring directly in JSON Schema is noisy and not the best user-facing
model format for artifact concepts and references.

### TypeSpec

Best developer experience for a general software audience. The syntax is close
to TypeScript interfaces without making TypeScript runtime validation the source
of truth. It has strong editor and emitter positioning, and JSON Schema output
can become Assura's compiled validation artifact. It remains the strongest
fallback if LinkML authoring feels too specialized after a real editor pass.

Risk: references and semantic linked-data concepts are weaker than LinkML.
Assura will still own repo-specific references, collection bindings, source
locations, and adapter behavior.

### CUE

Strongest constraint language among the candidates. It is also compelling for
config-like data validation.

Risk: its unification model is less familiar to mainstream Python/TypeScript
users and may be harder for agents to infer correctly from examples.

### Zod / TypeBox

Excellent TypeScript developer experience and good for website/editor helper
packages.

Risk: it makes TypeScript the center of the model system, which is a poor fit
for Assura's cross-language repository validation goal. Its native runtime
score is still good if JSON Schema artifacts are checked in or compiled.

### Assura-Owned Control

Useful as a control because it shows what the minimal notation could be.

Risk: inventing a schema ecosystem would duplicate existing standards and lose
IDE/generator leverage.

## Independent Review Notes

Three independent candidate review passes plus one final completion-audit pass
were run against the goal, report, tests, and fixtures. The preserved review
record is
`docs/analysis/2026-06-27-artifact-modeling-options-review-record.md`.

- LinkML/TypeSpec review: the recommendation is defensible only as "prototype
  LinkML first, keep TypeSpec close," not as a final production choice. The
  review specifically required actual generator probes and warned that
  TypeSpec decorators for references could change the order.
- JSON Schema/CUE review: JSON Schema/JTD is correctly the runtime target, but
  CUE must be blocked from the runtime path unless restricted to generated
  artifacts. The review also noted that relation validation currently belongs
  to Assura's collection/reference layer, not plain JSON Schema.
- Zod/control review: Zod should not be penalized inside the runtime category
  once JSON Schema is generated, but it remains weak as a cross-language source
  of truth. The control candidate should remain a benchmark, not be dismissed
  without testing a small generator/profile.

Addressed findings:

- Added actual TypeSpec, LinkML, CUE, and Zod generation probes.
- Added a native safe-update proof to the Rust comparison test.
- Updated LinkML to use the same user-facing `status` field as the other
  candidates.
- Updated TypeSpec, LinkML, CUE, and the Assura control source sketches to
  express the same basic non-empty constraints as the runtime schema where the
  candidate syntax supports them plainly.
- Reworded the recommendation as a next-prototype decision rather than a final
  permanent choice.
- Added generated/compiled artifact validation and a cached native-runtime loop
  after the final reviewer blocked completion.
- Preserved reviewer IDs, per-candidate coverage, findings, and resolutions in
  the review record.

## Requirement Audit

| Requirement | Current evidence | Status |
| --- | --- | --- |
| Comparable TypeSpec, LinkML, CUE, and JSON Schema/JTD MVPs | Candidate source files under `tests/fixtures/artifact_modeling_options/models/` | Satisfied |
| Same artifact classes, fixtures, references, and storage bindings | Shared bindings, pass/fail fixtures, and normalized runtime schemas under `tests/fixtures/artifact_modeling_options/` | Satisfied |
| Rust validation over generated or compiled artifacts | `tests/artifact_modeling_options_comparison.rs` validates normalized runtime artifacts and candidate generated/compiled artifacts | Satisfied |
| Safe update or blocked write support | Native safe-update test updates Markdown frontmatter, preserves `id`, revalidates, writes, and rejects identity changes | Satisfied |
| Command sequence and generated artifacts | Generator commands and snapshots in `generated_outputs/` | Satisfied for authoring-time probes |
| Editor, CLI, or generated-docs evidence | CLI command results, generated snapshots, and `generated_outputs/dx_evidence.md` with source links | Satisfied for comparison-level evidence |
| Weighted score sheet and reviewer notes | Scoring table, Independent Review Notes, and preserved review record | Satisfied |
| Independent review per serious candidate | Review record maps TypeSpec, LinkML, JSON Schema/JTD, CUE, Zod/TypeBox, and Assura control to independent reviewer coverage | Satisfied |
| Final recommendation with winner, fallback, rejected paths, next implementation goal | Recommendation and Next Implementation Goal sections | Satisfied, with explicit provisional language |

## Recommendation

Use this architecture:

```text
Authoring model: LinkML profile first, TypeSpec fallback for DX-heavy projects
Runtime artifact: normalized JSON Schema-like compiled schema
Validation/writes: native Assura binary
Repo binding: Assura collection and adapter config
Storage adapters: Assura-owned Markdown/frontmatter, JSON, YAML, JSONL, CSV
```

Assura should not require TypeSpec, LinkML, CUE, Node, Python, or Go for normal
validation. Those tools may be used to author or regenerate schema artifacts,
but checked-in or compiled artifacts must let installed Assura binaries validate
and update repo content directly.

## Next Implementation Goal

The next implementation goal should prototype **LinkML as an authoring source
feeding Assura's compiled artifact path**, while keeping the model-source
interface open enough to run the same slice with TypeSpec:

1. Add a `models:` config block that points to a model source and compiled
   schema artifact.
2. Add collection bindings for `class`, `path`, `adapter`, and `id`.
3. Compile/load the generated JSON Schema-like artifact into an Assura-native
   validator cache.
4. Validate Markdown frontmatter and JSON records as the same logical class.
5. Keep the existing Assura structure notation responsible for file placement,
   closed-world directories, and path-level policy.

If LinkML's authoring and editor experience is too heavy after that prototype,
run the same implementation slice with TypeSpec before making the production
decision. JSON Schema/JTD remains the runtime artifact either way.

## Manageable Extensions That Could Change Scores

- If LinkML model authoring can be wrapped in a very small Assura-friendly
  profile, LinkML's DX score improves.
- If TypeSpec reference semantics can be extended cleanly with custom
  decorators, TypeSpec's references/constraints score improves enough to
  challenge LinkML.
- If CUE can emit compact JSON Schema artifacts and provide approachable
  examples for TypeScript/Python developers, CUE's DX score improves.
- If Zod/TypeBox can be confined to generated helper packages rather than
  source-of-truth models, it remains valuable without being a core candidate.
