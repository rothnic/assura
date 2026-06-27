---
id: analysis-2026-06-27-repo-native-content-object-decision
type: analysis
title: Repo-native content object decision
status: active
created: 2026-06-27
owners:
  - assura-maintainers
related:
  - docs/goals/assura-artifact-modeling-options-comparison.md
  - docs/goals/assura-repo-native-content-runtime-implementation.md
  - docs/analysis/2026-06-27-artifact-modeling-options-comparison.md
  - tests/artifact_authoring_paths_proof.rs
---

# Repo-Native Content Object Decision

## Decision

Assura should implement repo-native content objects as a layered system:

```text
authoring model     LinkML profile first; TypeSpec fallback
runtime contract    checked-in or compiled JSON Schema-like artifact
file adapters       Assura-owned Markdown/frontmatter, JSON, YAML, JSONL
relations           Assura-owned identity and cross-file reference checks
writes              Assura-owned typed operation contracts and safe adapters
indexes/backends    optional caches, not canonical source of truth
```

LinkML is the first authoring path to carry forward because the restricted
profile validates with current LinkML tooling, models references naturally, and
does not need to be present in Assura's runtime hot path.

TypeSpec stays in the race as the developer-experience fallback. Its source is
more approachable for TypeScript-oriented developers, but the prototype proved
that Assura-specific decorators such as `@id`, `@ref`, `@collection`, and
`@adapter` need a TypeSpec JavaScript implementation package before the source
can compile.

JSON Schema or a JSON-Schema-like normalized artifact is the runtime target.
Assura should load and cache validators natively in Rust. LinkML, TypeSpec,
Node, Python, Go, CUE, Zod, Deeb, SQLite, or a server must not be required for
normal validation or agent update workflows.

## Why Not Deeb As The Core Layer

Deeb remains useful to evaluate as an optional internal storage or query cache,
but it should not be the core model layer.

The reason is product fit. Assura needs repository files to remain the source
of truth across Markdown frontmatter, JSON, YAML, JSONL, CSV, and future MDX
or document-like adapters. Deeb is a Rust-native JSON database over JSON files.
It brings CRUD, queries, indexing, and transactions, but it does not solve the
portable schema authoring problem, Markdown body preservation, non-JSON storage
adapters, or Python/TypeScript-friendly agent contracts.

Use Deeb only if an implementation benchmark shows it is a better internal
index/cache than an Assura-owned file index. It should not define the public
artifact model.

## Evidence

- `tests/fixtures/artifact_modeling_options/authoring_paths/models/linkml_profile/project.linkml.yaml`
  defines the larger LinkML profile and passed `linkml-validate`.
- `tests/fixtures/artifact_modeling_options/authoring_paths/models/typespec_decorators/project.tsp`
  shows the equivalent TypeSpec source shape; `tsp compile --no-emit` stops on
  missing decorator implementations.
- `tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/*.runtime.schema.json`
  are manual normalized runtime snapshots used by the Rust proof.
- `tests/fixtures/artifact_modeling_options/authoring_paths/contracts/agent_operations.schema.json`
  demonstrates the agent operation contract shape.
- `tests/artifact_authoring_paths_proof.rs` validates Markdown frontmatter,
  JSON, YAML, JSONL, nested records, references, safe writes, operation
  payloads, and cached Rust `jsonschema` validation.
- The current performance proof validates 800 file-backed records in roughly
  80ms after validators are compiled, with the focused test finishing around
  0.31s on the local machine.

## Path Forward

1. Land the current prototype and decision branch as a research/proof PR.
2. Start a new implementation branch from the updated base branch.
3. Implement the smallest production vertical slice:
   - `models:` config entry pointing to a runtime schema artifact;
   - collection bindings for `class`, `path`, `adapter`, and `id`;
   - native Rust JSON Schema validator cache;
   - Markdown frontmatter and JSON record adapters first;
   - relation validation across collections;
   - one typed agent operation contract.
4. Keep YAML and JSONL in the implementation goal as extension gates if the
   first slice lands cleanly.
5. Add LinkML generation as an authoring-time tool only after the runtime
   contract and adapter layer are real.
6. Re-score TypeSpec only after a minimal decorator package exists.

## Review Bar

The next PR should be blocked if it:

- requires LinkML, TypeSpec, Node, Python, Go, Deeb, SQLite, or a server during
  normal `assura check`;
- makes Rust structs or derive macros the public model source for non-Rust
  repositories;
- treats generated runtime snapshots as generated proof without a command or
  checked artifact;
- rewrites Markdown bodies during frontmatter-only updates;
- validates record shape but skips cross-file references;
- adds a new schema/model DSL before LinkML profile and TypeSpec decorator
  options are proven inadequate.
