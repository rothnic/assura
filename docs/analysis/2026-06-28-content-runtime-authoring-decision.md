---
id: analysis-2026-06-28-content-runtime-authoring-decision
type: analysis
title: Content runtime authoring decision
status: active
created: 2026-06-28
owners:
  - assura-maintainers
related:
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/analysis/2026-06-27-repo-native-content-object-decision.md
  - docs/analysis/2026-06-27-artifact-modeling-options-comparison.md
  - tests/artifact_authoring_paths_proof.rs
  - tests/fixtures/artifact_modeling_options/authoring_paths/
---

# Content Runtime Authoring Decision

## Decision

Use a restricted **LinkML profile** as the first production authoring source
for repo-native content models, compiled through an Assura-owned normalizer into
a checked-in JSON Schema-compatible runtime artifact.

Keep **TypeSpec decorators** as the first fallback for DX-heavy software teams,
but defer it until Assura has a small TypeSpec package that implements the
repo-specific decorators and emits the same runtime metadata.

Reject **CUE** and a standalone **Assura-owned DSL** for the current production
path. CUE may remain an authoring-time experiment, and an Assura-owned profile
layer remains useful for documenting the accepted subset, but neither should
become the public model source for this increment.

## Runtime Boundary

Normal `assura check`, validation, and typed write operations must only read
checked-in runtime artifacts and ordinary repository content files. They must
not shell out to LinkML, TypeSpec, CUE, Node, Python, Go, Deeb, SQLite, or a
server.

The selected boundary is:

```text
LinkML profile YAML
  -> authoring-time Assura profile normalizer
  -> checked-in JSON Schema-compatible runtime artifact
  -> native Rust validator cache and Assura-owned file adapters
```

The checked proof manifest is
`tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/selected_authoring_profile.compile.json`.
It selects `linkml_profile`, points at the checked
`linkml_profile.runtime.schema.json` artifact, names
`typespec_decorators` as the fallback, and records an empty runtime hot-path
dependency list. It also records the checked LinkML source path, source hash,
runtime artifact hash, LinkML validation command, and validation-result hash so
the selected path is auditable before the production compiler command exists.
Those hashes are computed after CRLF-to-LF normalization so the proof is stable
across GitHub's Linux, macOS, and Windows checkouts.

## Why LinkML First

LinkML is the strongest current fit because:

- it already models classes, slots, identifiers, required fields, enums,
  arrays, references, and inlined nested objects in a storage-neutral YAML
  source;
- the restricted profile validates with current LinkML tooling;
- LinkML has documented JSON Schema generation, so it can feed the runtime
  artifact path without becoming a runtime dependency;
- its ecosystem is friendlier to Python/data/documentation repositories than a
  TypeScript-centered source of truth;
- it lets Assura keep collection paths, adapters, source locations, safe
  writes, and reference resolution in Assura-owned runtime code.

Primary-source checks used for this decision:

- LinkML documents JSON Schema generation from LinkML schemas:
  https://linkml.io/linkml/generators/json-schema.html
- LinkML generators map schemas to web standards including JSON Schema:
  https://linkml.io/linkml/generators/index.html
- LinkML documents schema-aware YAML editing with JSON Schema support:
  https://linkml.io/linkml/faq/tools.html

## Why TypeSpec Is Fallback

TypeSpec remains the best readability challenger. Its syntax is approachable
for software teams, and the JSON Schema emitter can bundle models under
`$defs`, which matches Assura's runtime artifact shape.

It is not selected first because the second-stage decorator sketch still fails
without an Assura TypeSpec implementation package for `@id`, `@ref`,
`@collection`, and `@adapter`. That package is reasonable future work, but it
is extra bridge code that LinkML does not need for the next production slice.

Primary-source checks:

- TypeSpec JSON Schema emitter options include bundled schema output under
  `$defs`: https://typespec.io/docs/emitters/json-schema/reference/emitter/
- TypeSpec supports custom emitter configuration through library-defined
  options, which is the likely route for Assura-specific output:
  https://typespec.io/docs/extending-typespec/emitters-basics/

## Why Not CUE Now

CUE is strong for constraints and configuration-like data, but it is less
obvious for mainstream content model authoring across Python, TypeScript, Rust,
and documentation repositories. CUE can work with JSON Schema and can export
definitions as JSON Schema, but that makes it an authoring-time generator
candidate, not a runtime dependency or primary production source for this
feature.

Primary-source check:

- CUE documents JSON Schema interop and exporting definitions as JSON Schema:
  https://cuelang.org/docs/concept/how-cue-works-with-json-schema/

## Why Not A Standalone Assura DSL

Assura should own the profile, runtime metadata, adapter bindings, relation
resolution, diagnostics, and typed writes. It should not invent a new schema
language before standards-backed options fail in real usage.

The Assura-owned part for this increment is the **profile and normalizer
contract**, not a standalone DSL:

- accepted authoring subset;
- collection/adapters metadata;
- relation metadata;
- normalized runtime artifact shape;
- proof that runtime validation stays Rust-native.

## Compile Path Contract

This increment selects the compile path and records reproducibility evidence,
but it does not claim that Assura already ships a production compiler command.
The next implementation slice should add a real compiler command behind this
contract. The proposed interface is a content-model compiler that accepts a
profile name, a model source path, and an output runtime artifact path. Do not
document a runnable CLI invocation until that command exists in Assura's public
command surface.

Until that command exists, fixture evidence is represented by the checked
compile manifest and runtime artifact:

- `selected_authoring_profile.compile.json`
- `linkml_profile.runtime.schema.json`
- `linkml_profile_validate_result.txt`
- `typespec_decorators_compile_result.txt`

The compile manifest is intentionally stricter than a note: it records the
source model hash, selected runtime artifact hash, LinkML validation command,
and validation output hash. The focused proof test verifies those fields so a
source, artifact, or validation-result change cannot drift silently. Hashes are
CRLF-to-LF normalized because these are text fixtures and the proof should not
depend on the platform checkout line endings.

The runtime artifact includes `x-assura.collections` and `x-assura.relations`
metadata so Assura can keep storage adapters and cross-file references outside
plain JSON Schema validation while still using JSON Schema for object shape.

## DX Implications

TypeScript users can inspect the runtime JSON Schema artifact with ordinary
JSON Schema-aware tooling and can later use the TypeSpec fallback when the
decorator package exists.

Python and data-oriented users get the stronger first path: LinkML YAML source,
schema-aware YAML editing, and generated JSON Schema artifacts.

Rust users do not need a Rust derive macro or Cargo build step to define model
source. They consume the same checked artifact as every other project.

## Validation Evidence

The focused proof is:

```bash
cargo test --test artifact_authoring_paths_proof --quiet
```

It now checks that:

- the selected authoring manifest chooses `linkml_profile`;
- the runtime hot-path dependency list is empty;
- the checked LinkML source, selected runtime artifact, and validation result
  match their recorded SHA-256 hashes;
- the selected checked artifact matches the TypeSpec fallback runtime contract
  after source metadata is removed;
- Markdown frontmatter, JSON, YAML, and JSONL records validate against the
  selected checked artifact;
- references resolve through Assura-owned logic rather than authoring tools.

## Review Bar

Block future PRs if they:

- require LinkML, TypeSpec, CUE, Node, Python, Go, Deeb, SQLite, or a server
  during normal validation;
- make Rust structs, Rust derive macros, or Cargo build steps the model source
  for non-Rust repositories;
- hide collection/adapters/reference metadata inside a source-specific tool in
  a way Assura cannot inspect natively;
- claim generation without a checked artifact, command, or reproducible
  manifest;
- add a standalone Assura schema DSL before the LinkML profile and TypeSpec
  fallback have both been tested with real example repos.

## Next Step

Increment 8 should turn this decision into DX evidence: a small example repo
with LinkML source, the checked runtime artifact, Markdown frontmatter and JSON
records, and inspection guidance for TypeScript, Python, and Rust users.
