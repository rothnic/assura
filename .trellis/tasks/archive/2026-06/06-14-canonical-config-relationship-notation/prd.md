# Canonical config relationship notation

## Goal

Make Assura's relationship notation stable, concise, and tree-shaped. The
configuration should keep files, directories, and document sections where they
appear in the project hierarchy while supporting relationships through shared
named captures. The common case should not require detached
`extensions.custom_constraints` declarations or source-local paths that point
into another tree branch.

## What I Already Know

- The current runtime supports `extensions.custom_constraints` with
  `paired_file_exists`, using `source` and `target` templates such as
  `tests/{stem}_test.rs`.
- Historical AST/parser code and archived docs still use `${name}` patterns.
- Older design docs also proposed `{{name}}` templates and source-local
  `require_test` attributes.
- The user prefers the first, tree-shaped pattern: one optional producer entry
  with no `exists` plus one captured counterpart with `exists:1`.
- The user does not want nested attributes to configure target paths in a
  different part of the project tree.
- The best either/or package-doc shape is `needs` on the source and
  `provides` on provider artifacts, with all artifacts configured where they
  live.

## Requirements

- Add a stable, well-known config notation spec for canonical relationship
  notation.
- Use single-brace named captures such as `{component}` and `{package}`.
- Treat a captured entry with no `exists` as an allowed/validated producer.
- Treat a captured counterpart with `exists:1` as required for each matching
  producer capture value in the same relationship set.
- Support either/or relationships with `needs` and `provides`, where one source
  capture can be satisfied by any provider with the same capture value.
- Keep target files and document sections configured where they live in the
  structure tree.
- Keep `extensions.custom_constraints` as an implementation detail only if
  needed internally; do not promote it as the authoring notation for
  relationship checks.
- Remove active docs/tests that promote `${name}`, `{{name}}`,
  source-local cross-tree `requires`, or detached custom-constraint notation as
  the preferred Assura-native relationship syntax.
- Do not preserve backwards compatibility for removed alpha notation.

## Target Notation

Simple same-directory counterpart:

```yaml
structure:
  src/components/:
    "{component}.tsx":
      use: "@react-component"

    "{component}.test.tsx":
      exists: 1
      use: "@component-test"
```

Centralized counterpart:

```yaml
structure:
  src/components/:
    "{component}.tsx":
      use: "@react-component"

  tests/components/:
    "{component}.test.tsx":
      exists: 1
      use: "@component-test"
```

Either/or package documentation:

```yaml
structure:
  packages/:
    "{package}/":
      use: "@package-standard"
      needs: doc

  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
      use: "@package-doc"

  docs/:
    required: false
    packages.md:
      sections:
        "{package}":
          provides: doc
          use: "@package-doc-section"
```

## Acceptance Criteria

- [x] `.trellis/spec/assura/config-notation.md` exists and records the
      canonical relationship notation, examples, semantics, non-goals, and
      rejected alternatives.
- [x] Parser/normalization tests cover single-brace captured structure keys and
      reject or stop documenting `${name}` / `{{name}}` as current notation.
- [x] Runtime or fixture tests cover conditional counterpart existence for
      captured producer/counterpart pairs.
- [x] Runtime or fixture tests cover `needs` / `provides` either/or package
      documentation semantics.
- [x] Active docs no longer present `extensions.custom_constraints` as the
      preferred way to author relationship checks.
- [x] Active docs no longer promote source-local cross-tree `requires` paths.
- [x] `cargo test --test custom_constraints_tests --quiet` passes if the
      internal custom-constraint executor remains touched.
- [x] Focused config/parser tests for the new notation pass.
- [x] `cargo run --quiet -- check --format json .` passes.
- [x] `git diff --check` passes.

## Out Of Scope

- Arbitrary shell-based relationship validators.
- General graph/dependency analysis.
- Preserving backwards compatibility with alpha-only notation.
- Implementing every Markdown outline validation feature in this task unless
  needed for the package-doc provider fixture.

## Technical Notes

- Relevant current implementation:
  - `src/config/loader.rs`
  - `src/config/config.rs`
  - `src/config/config/extensions.rs`
  - `src/config/config/validation.rs`
  - `src/cli/check/custom_constraints.rs`
  - `tests/custom_constraints_tests.rs`
- Relevant historical design inputs:
  - `docs/archive/ls-lint-notation-guide.md`
  - `docs/archive/final-config-design.md`
  - `docs/archive/configuration-spec.md`
  - `docs/analysis/2026-05-15-notation-source-truth.md`
