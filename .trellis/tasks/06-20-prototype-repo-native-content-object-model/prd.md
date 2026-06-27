# Prototype Repo-Native Content Object Model

## Goal

Explore a functional prototype for treating repository files as typed content
objects that can be validated, linked, queried, and safely modified through
Assura. The prototype should compare model, adapter, graph/reference, storage,
and write-path options instead of assuming a single library such as Deeb is the
right foundation.

## What I Already Know

* The desired product shape is closer to writable content collections or a
  content repository than a plain JSON database.
* Repo files should remain first-class: directories define which object types
  may live there, and file formats such as JSON, JSONL, YAML, CSV, Markdown,
  Markdown plus frontmatter, and MDX need adapters.
* Markdown documents need both structured metadata and body/outline handling.
* Objects need references to objects in other collections.
* Agents should mutate content through typed commands or APIs with validation,
  not by arbitrary text edits.
* Assura already has compact structure notation, Markdown frontmatter checks,
  Markdown outline validation, and `needs` / `provides` relationship notation.
* Existing config notation is YAML-centered and structure-first; this spike
  should not break current LS-Lint-equivalent structure use cases.

## Assumptions

* Canonical repository state should remain file-backed and Git-reviewable.
* Any SQL, document DB, or embedded object store is a cache/index/backend unless
  the prototype proves a strong reason to make it canonical.
* Runtime validation should stay Rust-native and fast.
* External schema/model tools may be useful if they compile to JSON Schema or a
  small Assura IR.

## Requirements

* Use the file-native prototype path as the first MVP: canonical repository
  files remain the source of truth, and storage/index backends are compared
  after the baseline object/adapters/reference path works.
* Model repository files as typed objects with stable identity, source path,
  optional frontmatter/data, optional body, and source location metadata.
* Define directory/path scopes that allow or reject object types below those
  scopes.
* Support at least one Markdown plus frontmatter collection and one JSON record
  collection in the prototype.
* Support references between collections and report missing or invalid targets.
* Demonstrate read, validate, update, and round-trip write behavior.
* Compare alternatives for each layer:
  * model/schema layer;
  * format adapter layer;
  * repository graph/reference layer;
  * storage/index/query layer;
  * typed agent mutation interface.
* Produce a recommendation with evidence instead of only a code demo.

## Acceptance Criteria

* [x] Prototype loads at least two collection types from a fixture repo.
* [x] Prototype validates type placement by directory/path scope.
* [x] Prototype validates object fields for both Markdown frontmatter and JSON.
* [x] Prototype validates a cross-collection reference.
* [x] Prototype performs at least one safe write and verifies the post-write
      file still parses and validates.
* [x] Prototype records a comparison of at least three solution combinations.
* [x] Prototype documents whether Deeb, SQLite, LinkML, JSON Schema, Keystatic-
      style file collections, or custom Assura glue should be carried forward.

## Definition Of Done

* Task PRD and research notes are complete enough to route future agents.
* Prototype code and fixtures are scoped to a test-only experimental module
  until a later task decides whether to expose a runtime CLI/API surface.
* Tests cover the fixture load, validation, reference resolution, and write path
  chosen for the MVP.
* Review agent inspects the prototype and recommendation before any PR is
  created.
* Validation commands appropriate to changed surfaces pass or are clearly
  documented if skipped.

## Research References

* [research/solution-space.md](research/solution-space.md) - Traditional
  categories and where Assura fits.
* [research/model-layer-options.md](research/model-layer-options.md) - LinkML,
  JSON Schema, custom DSL, and Rust-first options.
* [research/storage-index-options.md](research/storage-index-options.md) -
  Deeb, SQLite, SurrealDB, and file-native indexes.
* [research/file-collection-options.md](research/file-collection-options.md) -
  Keystatic, Astro Content Collections, and adapter implications.
* [research/prototype-baseline-results.md](research/prototype-baseline-results.md)
  - Implemented file-native baseline and recommendation.

## Technical Notes

* Current Markdown surfaces include `src/markdown/*` and
  `src/config/config/bundles/markdown.rs`.
* Current notation and relationship source of truth is
  `.trellis/spec/assura/config-notation.md`.
* Current website docs document `extra: false`, `rules`, `use`, `needs`,
  `provides`, and Markdown `outline`.
* Current dependencies already include `serde`, `serde_json`, `serde_yaml`,
  `toml`, `glob`, `walkdir`, `petgraph`, `pulldown-cmark`, and `frontmatter`.
* Dependency additions must be justified and should be avoided in the first
  prototype unless the comparison specifically needs one.
* The first implementation adds a test-only `src/content_repository/` module
  behind `#[cfg(all(test, feature = "full-cli"))]`. It uses existing
  `serde_json`, `serde_yaml`, `glob`, and `walkdir` dependencies and adds no
  new crate.
* MVP placement validation applies to typed objects discovered by configured
  collection globs. Existing Assura structure rules still own unknown-file and
  closed-world directory enforcement until a later collection-discovery design
  broadens this prototype.

## Decision (ADR-lite)

Context: The prototype could start with a backend comparison, a model-language
comparison, or a file-native end-to-end proof.

Decision: Start with the file-native prototype. Canonical files remain in the
repo; implement minimal Markdown plus frontmatter and JSON record adapters,
object graph/reference validation, directory/type placement validation, and one
safe write path before comparing Deeb/SQLite as optional indexes.

Consequences: This proves the hardest product constraint first: source files
stay human-editable and Git-reviewable while agents get a typed mutation
boundary. It delays deeper backend comparison until the normalized object model
and adapter requirements are concrete.

## Open Questions

* None for the first MVP slice.

## Out Of Scope

* Full CMS UI.
* Running server as a required path.
* Replacing the current Assura config notation in this task.
* Supporting every file format in the first prototype.
* Implementing migrations or long-term persistence guarantees before the
  storage/index comparison is complete.
