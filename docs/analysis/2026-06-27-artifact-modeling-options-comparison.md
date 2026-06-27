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
The second-stage proof strengthened that recommendation: the restricted LinkML
profile validates with current LinkML tooling, while the readable TypeSpec
decorator shape needs an Assura-owned JavaScript decorator/emitter bridge before
it can compile.

The important product constraint is not "which tool can validate data." Assura
needs users to install one compiled binary that can validate, transform, and
write repo artifacts across Rust and non-Rust projects without requiring Node,
Python, Go, CUE, or a server in the hot path.

The authoring source and runtime target should be treated as different layers.
JSON Schema or JTD is the best Rust runtime target, but it is not the best
human-facing model syntax. LinkML currently leads the next-prototype score
because it has the strongest cross-language and relationship-modeling story,
the lowest authoring-bridge work after the second proof, and still allows
Assura to validate generated artifacts natively. The evidence does not justify
permanently choosing LinkML yet; TypeSpec is close enough that the next
production slice should keep the model-source interface open.

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
- Second-stage authoring proof:
  `tests/fixtures/artifact_modeling_options/authoring_paths/`

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
- TypeSpec custom decorators are implemented in JavaScript and declared with
  `extern dec`, which is why Assura-specific decorators need a small TypeSpec
  package instead of source-only declarations:
  https://typespec.io/docs/extending-typespec/create-decorators/
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

## Second-Stage Authoring Proof

The second-stage prototype uses a larger repository artifact model with nested
objects, arrays, optional references, backrefs, JSONL records, and explicit
collection adapters:

- `Goal`: Markdown frontmatter plus Markdown body.
- `Spec`: JSON record.
- `Task`: YAML record.
- `Decision`: JSONL record.
- Nested objects: `ArtifactMetadata` and `Evidence`.
- Relations: `Goal.specs`, `Goal.tasks`, `Goal.decisions`, `Task.goal`,
  `Task.spec`, `Decision.supersedes`, `Decision.affects_specs`, and
  `Decision.affects_tasks`.

Prototype files:

- LinkML profile:
  `tests/fixtures/artifact_modeling_options/authoring_paths/models/linkml_profile/project.linkml.yaml`
- TypeSpec decorator sketch:
  `tests/fixtures/artifact_modeling_options/authoring_paths/models/typespec_decorators/project.tsp`
- Manual normalized runtime snapshots:
  `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/*.runtime.schema.json`
- Agent operation contract:
  `tests/fixtures/artifact_modeling_options/authoring_paths/contracts/agent_operations.schema.json`
- Cross-format fixtures:
  `tests/fixtures/artifact_modeling_options/authoring_paths/fixtures/pass/`
- Rust proof:
  `tests/artifact_authoring_paths_proof.rs`

Commands and observed results:

```bash
uvx --from linkml linkml-validate \
  tests/fixtures/artifact_modeling_options/authoring_paths/models/linkml_profile/project.linkml.yaml
```

Observed result: `No issues found`.

```bash
npx --yes --package @typespec/compiler tsp compile \
  tests/fixtures/artifact_modeling_options/authoring_paths/models/typespec_decorators/project.tsp \
  --no-emit
```

Observed result: TypeSpec recognized the decorator declarations but failed with
four `missing-implementation` diagnostics for `id`, `ref`, `collection`, and
`adapter`. This is expected TypeSpec extension behavior: custom decorators need
a JavaScript implementation library. That makes TypeSpec viable, but not
zero-glue.

```bash
cargo test --test artifact_authoring_paths_proof --quiet
```

Observed result: `5 passed; 0 failed`.

```bash
cargo test --test artifact_authoring_paths_proof \
  native_runtime_performance_uses_cached_json_schema_validators -- --nocapture
```

Observed result: 800 file-backed records loaded through Markdown, JSON, YAML,
and JSONL adapters and validated through cached Rust `jsonschema` validators in
`79.845601ms`; the full single-test run finished in `0.32s`.

What this proves:

- The authoring model can be separate from stored representation.
- The same object model can cover frontmatter, JSON, YAML, and JSONL records.
- References and backrefs belong in Assura's collection/reference layer, even
  when the source model describes them.
- Write fidelity is an adapter concern. The proof preserves Markdown body bytes
  and YAML list layout for targeted edits, while JSON and JSONL use valid
  canonical rewrites rather than byte-preserving formatting.
- Agent writes should go through typed operations. The proof validates
  operation payloads against a checked-in JSON Schema contract, creates a
  `Task`, appends it to a `Goal`, validates record shape and references, and
  rejects a task with a missing `goal`.
- Runtime performance is not the limiting factor if validators are compiled and
  cached in native Rust.

What remains open:

- The second-stage runtime schemas are manual normalized snapshots, not proven
  generated output. That is enough to validate the runtime interface and test
  authoring shape, but not enough to claim an end-to-end compiler.
- A production TypeSpec path needs a tiny Assura TypeSpec package or emitter
  that implements `@id`, `@ref`, `@collection`, and `@adapter`.
- A production LinkML path needs a documented Assura profile so users do not
  have to learn the whole LinkML metamodel.
- Neither authoring source should be required in the normal Assura hot path.

## Feature Scorecard

The earlier numeric score was useful for a first pass, but it was too easy to
read as arbitrary. The more meaningful comparison is feature proof against the
actual product requirements.

Legend: `Proven` means covered by checked-in fixture, command output, or Rust
test. `Partial` means plausible but needs Assura glue. `Weak` means the option
fights the requirement.

| Requirement | LinkML profile | TypeSpec decorators | JSON Schema/JTD | CUE | Zod/TypeBox | Assura DSL |
| --- | --- | --- | --- | --- | --- | --- |
| Source validates with current authoring tool | Proven: `linkml-validate` passed | Partial: syntax is readable, but decorators need JS implementation | Proven | Proven for constrained CUE exports | Proven in TS | Weak until Assura builds validator/tooling |
| Runtime works in one native Assura binary | Proven through manual normalized JSON Schema snapshot | Proven through manual normalized JSON Schema snapshot | Proven directly | Partial if generated artifact is checked in | Partial if generated artifact is checked in | Proven if built |
| Cached Rust validation performance | Proven: 800 file-backed records in 79.845601ms | Proven: same runtime artifact | Proven | Same if normalized to JSON Schema | Same if normalized to JSON Schema | Proven if built |
| Storage-neutral object model | Proven with Markdown, JSON, YAML, JSONL adapters | Proven at runtime; authoring metadata needs emitter bridge | Partial: noisy authoring | Partial: good constraints, less obvious content model | Weak: TS-centered source of truth | Proven if built |
| References and backrefs | Proven in LinkML slots plus Assura relation metadata | Partial: decorators are clear, implementation missing | Partial: needs Assura metadata | Strong constraints, but not mainstream | Partial: custom metadata | Proven if built |
| Developer readability for Python and TS users | Partial: understandable YAML profile, but specialized | Strong: TS-like syntax | Weak: too verbose | Partial: unfamiliar semantics | Strong for TS, weaker for Python/Rust | Strong only if designed well |
| IDE and ecosystem leverage | Strong: generators, docs, validation, Python/data ecosystem | Strong after package setup; VS Code/compiler story is good | Strong editor support | Good but niche | Strong TS ecosystem | Weak initially |
| Agent write contract | Proven by operation schema plus Assura operation semantics test | Proven by same runtime interface | Proven by same runtime interface | Proven only after normalization | Proven only after normalization | Proven if built |
| Assura implementation cost | Lowest now: profile plus normalizer | Medium: needs TypeSpec library/emitter | Low runtime, poor authoring | Medium/high normalization | Medium, but TS-centric | Highest ecosystem cost |

Outcome:

- LinkML is highest now because the second-stage proof reduced uncertainty:
  the profile is valid today, references map naturally, and the runtime remains
  native JSON Schema.
- TypeSpec is still the best readability challenger, but no longer tied with
  LinkML until Assura proves the custom decorator package. The source sketch is
  pleasant; the toolchain bridge is real work.
- JSON Schema/JTD is the runtime target, not the primary authoring format.
- A custom Assura DSL should remain a fallback only if LinkML profile UX and
  TypeSpec decorators both fail in real authoring tests.

## Candidate Notes

### LinkML

Best semantic modeling ecosystem. It has the most direct story for linked data,
documentation, diagrams, JSON Schema, Python, TypeScript, Rust, SQL-ish, and
data-dictionary workflows. It is also the closest fit for modeling repository
objects and references independently from how each object is stored on disk.
The second-stage profile validated with `linkml-validate` without adding
LinkML to Assura's runtime.

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

Risk: references and semantic linked-data concepts are weaker than LinkML, and
the second-stage decorator sketch proved that `@id`, `@ref`, `@collection`,
and `@adapter` need a JavaScript TypeSpec implementation package before the
source compiles. Assura will still own repo-specific references, collection
bindings, source locations, and adapter behavior.

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
| Larger LinkML and TypeSpec authoring paths | Second-stage profiles under `tests/fixtures/artifact_modeling_options/authoring_paths/models/` | Satisfied, with TypeSpec decorator implementation risk |
| Same artifact classes, fixtures, references, and storage bindings | Shared bindings, pass/fail fixtures, and normalized runtime schemas under `tests/fixtures/artifact_modeling_options/` | Satisfied |
| Frontmatter, JSON, YAML, and JSONL as storage adapters for one object model | `tests/artifact_authoring_paths_proof.rs` loads and validates all four representations | Satisfied |
| References, optional refs, arrays, backrefs, and nested objects | Second-stage runtime schemas and fixtures include nested metadata/evidence and cross-record references | Satisfied |
| Rust validation over generated or compiled artifacts | `tests/artifact_modeling_options_comparison.rs` validates normalized runtime artifacts and candidate generated/compiled artifacts | Satisfied |
| Rust validation with a real JSON Schema validator/cache | `jsonschema` validators compile once and validate 800 file-backed records in 79.845601ms | Satisfied for prototype scale |
| Safe update or blocked write support | Native safe-update tests update Markdown frontmatter, YAML, JSON, and JSONL, preserve `id`, revalidate, write, and reject bad references; JSON and JSONL are canonical rewrites, not byte-preserving edits | Satisfied for prototype adapters |
| Agent typed create/update contract | `agent_operations.schema.json` plus `agent_typed_operations_create_update_records_against_contract` validate operation payloads, create a task, append it to a goal, validate, and reject a missing ref | Satisfied for prototype operation contract |
| Command sequence and generated artifacts | Generator commands and snapshots in `generated_outputs/` | Satisfied for authoring-time probes |
| Editor, CLI, or generated-docs evidence | CLI command results, generated snapshots, and `generated_outputs/dx_evidence.md` with source links | Satisfied for comparison-level evidence |
| Meaningful feature comparison and reviewer notes | Feature scorecard, Independent Review Notes, and preserved review record | Satisfied |
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
