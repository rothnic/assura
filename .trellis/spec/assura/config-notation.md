# Assura Native Config Notation

This document defines Assura's canonical native notation. The structure,
cardinality, and reusable-rule sections describe the implemented authoring
surface. The Markdown outline and code-to-document relation sections are
planned extension points until their validators and fixtures land.

## Purpose

Assura config should look like the project or document it validates. The common
path should be concise enough to compete with LS-Lint, while richer object
attributes stay available for custom validation.

This target notation is designed to:

- keep `structure:` as the project tree source of truth;
- avoid a separate `contents:` layer for direct children;
- avoid duplicate `required` and `allowed` declarations for the same item;
- use `exists` cardinality for required, optional, forbidden, and bounded
  direct contents;
- support reusable rule fragments through `rules:` and `@rule` references;
- reserve Markdown outline notation with the same visual hierarchy as the
  document;
- leave room for code-to-doc relationship checks without arbitrary commands.

## Core Shape

The native tree stays under `structure:`. Keys name the thing being validated.
Values are either concise shorthand or detailed attributes.

```yaml
rules:
  readme-standard:
    exists: 1

  agents-standard:
    exists: 1

  project-docs:
    README.md: "@readme-standard"
    AGENTS.md: "@agents-standard"

structure:
  ./:
    use: "@project-docs"
    Cargo.toml: exists:1
    src/: exists:1
    docs/: exists:1

  packages/*/:
    use: "@project-docs"
    package.json: exists:1
    src/: exists:1
```

## Path Keys

Path-like keys are first-class structure entries.

| Key shape | Meaning |
| --- | --- |
| `README.md` | Exact direct file. |
| `src/` | Exact direct directory. |
| `packages/*/` | Directory scope over existing matching directories. |
| `.md` | Direct file extension rule in the current scope. |
| `.test.ts` | Direct file subextension rule in the current scope. |
| `.dir` | Directory rule for the current directory scope. |

Trailing slash is the native way to disambiguate directories from exact files.
Compatibility migrations may accept LS-Lint-style directory keys without the
slash, but native examples should include the slash.

Pattern scopes validate existing matches. They do not create required literal
directories unless an `exists` rule explicitly requires a match.

## Cardinality

`exists` is the common-case presence and count model.

| Shorthand | Meaning |
| --- | --- |
| `exists:1` | Required exactly once and allowed. |
| `exists:0-1` | Optional singleton and allowed when present. |
| `exists:0` | Forbidden. |
| `exists:N-M` | Required count range. |

This replaces most duplicated `required` plus `allowed` declarations. A direct
child with `exists:1` is both required and allowed. A direct child with
`exists:0-1` is allowed but not required. A direct child with `exists:0` is
not allowed.

Nested attributes remain available when shorthand is not enough. The current
implemented node attributes are `exists` and `naming`; future validators may add
more attributes under the same path key shape.

```yaml
structure:
  packages/*/:
    .md:
      exists: 1-4
      naming: kebab-case
```

## Closed-World Direct Contents

Closed-world checks should remain directory-local. Native tree notation maps to
the direct-child file and directory policies described by
`structure-enforcement.md`.

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    AGENTS.md: exists:1
    Cargo.toml: exists:1
    src/: exists:1
    docs/: exists:1
```

`extra: false` means undeclared direct files and directories are rejected in
that scope. It does not recursively make descendants closed-world unless a
descendant scope also says so.

## Reusable Rules

`rules:` defines reusable fragments. `"@name"` references a fragment. References
are quoted in examples because bare `@` values are not valid YAML scalars.
Future parser work may add preprocessing for unquoted `@name`, but the stable
documented form must remain valid YAML after preprocessing.

```yaml
rules:
  package-standard:
    README.md: "@readme-standard"
    AGENTS.md: "@agents-standard"
    package.json: exists:1
    src/: exists:1

structure:
  packages/*/:
    use: "@package-standard"
```

Fragments have inferred kinds:

- A node fragment contains attributes for one file, directory, extension, or
  scope. Current node fragments support `exists` and `naming`.
  It is valid as the value of a path-like key.
- A tree fragment contains path-like child keys and optional `use` entries. It
  is valid through `use` at a structure node or inside another tree fragment.
- A fragment must not mix node attributes and path-like child keys at the same
  map level. Put node attributes under a path key when composing tree fragments.
- Referencing a tree fragment where a node fragment is required, or the reverse,
  is a configuration error.

Merge order should be deterministic:

1. Expand `use` references first. If `use` is a list, merge left to right.
2. Apply local keys over referenced keys.
3. Prefer exact keys over pattern keys.
4. Prefer more specific scopes over broader scopes.

## Planned Markdown Outline Notation

Markdown outline validation is not implemented in the current native parser.
When added, outlines should use nested YAML lists and maps. The nesting is the
heading hierarchy; the order is the expected document order.

```yaml
markdown:
  outline:
    - Overview
    - ?? Prerequisites
    - Quick Start:
        - Installation
        - ?? Configuration
    - Why Assura?
    - ?? Troubleshooting
```

Rules:

- Plain heading text is required.
- A heading string beginning with `?? ` is optional.
- A heading that contains or ends with `?` is still a normal heading.
- A heading that starts with literal `?? ` can use the object escape hatch.
- Nesting derives heading levels. Implementations should not require users to
  maintain separate `h2`/`h3` depth fields for normal documents.
- Matching is relative by default. If the document starts with exactly one H1
  before any lower-level headings, treat that H1 as the document title and match
  root outline entries below it. Otherwise, match the root outline against the
  first heading level that can satisfy the first required root item.
- Once the root level is chosen, every nested outline level must match the next
  deeper Markdown heading level. Skipped levels or multiple possible root
  matches are validation errors unless an expanded object node selects a custom
  match mode.
- Optional parents may be absent. If an optional parent is present, its
  required children are checked within that section.

Examples:

```yaml
markdown:
  outline:
    - Why Assura?
    - ?? Why Assura?
    - title: "?? Debug Mode"
      optional: false
```

The first line is a required heading containing a question mark. The second is
an optional heading containing a question mark. The object form is reserved for
rare cases where the heading text itself would collide with shorthand.

Expanded object nodes are also the extension point for custom matching:

```yaml
markdown:
  outline:
    - title: API Reference
      optional: true
      match: regex
      validate:
        - public-api-links
```

## Planned Code-To-Documentation Relations

Relation validation is not implemented in the current native parser. Some
projects need structure relationships across directories; when added, Assura
should support deterministic relation checks without requiring custom shell
execution.

```yaml
rules:
  package-doc:
    markdown:
      outline:
        - Overview
        - Public API
        - ?? Examples
        - ?? Migration Notes

  package-doc-section:
    markdown:
      outline:
        - Overview
        - Public API
        - ?? Examples

relations:
  package-docs:
    for_each: src/packages/*/
    capture:
      package: basename
    require:
      any:
        - docs/packages/{package}.md:
            use: "@package-doc"
        - docs/packages.md:
            section: "{package}"
            use: "@package-doc-section"
```

This means every package directory must be documented either by a dedicated
document or by a section in an aggregate document. The captured `{package}`
token comes from the matched directory basename.

Relation semantics:

- Relation paths are resolved from the project root unless documented
  otherwise.
- `capture: { package: basename }` captures the matched directory basename.
  Implementations may later add explicit normalization functions, but the
  default value is the literal basename.
- Template placeholders are expanded before path or heading matching.
- `section: "{package}"` selects exactly one heading with the expanded title in
  the target document. Zero matches fail the alternative. Multiple matches are
  ambiguous and fail unless a future selector disambiguates them.
- When `section` is present, the referenced rule validates inside that section;
  its outline starts with child headings under the selected section, not with
  the selected heading itself.

Relation checks should be:

- opt-in;
- deterministic;
- based on paths and parsed document structure;
- independent of network access;
- independent of repository-defined shell commands.

## Proof Use Cases

Implementation should prove these before declaring the notation ready:

1. Root project contract: require `README.md`, `AGENTS.md`, `Cargo.toml`,
   `src/`, and `docs/`, while rejecting undeclared root files when
   `extra: false`.
2. Monorepo package contract: every `packages/*/` directory has `README.md`,
   `AGENTS.md`, `package.json`, and `src/` without duplicating those rules at
   each package path.
3. Optional singleton files: allow but do not require files such as
   `CHANGELOG.md` through `exists:0-1`.
4. Forbidden files: reject root scratch files or generated artifacts through
   `exists:0` or forbidden pattern attributes.
5. Markdown outline contract: validate ordered nested headings with optional
   sections marked by `?? `.
6. Question headings: allow required headings such as `Why Assura?` and
   optional headings such as `?? Why Assura?`.
7. Escaped/custom headings: support object-form nodes for headings that collide
   with shorthand or need custom match attributes.
8. Code-to-doc relation: for every matched package directory, require either a
   dedicated docs file or a package section in an aggregate docs file.

## Non-Goals

- Do not require users to write `children:` for normal structure or heading
  nesting.
- Do not introduce a `contents:` block for ordinary direct files/directories.
- Do not preserve separate `required` and `allowed` declarations as the common
  notation for the same child.
- Do not require users to sync Markdown heading depth numbers with outline
  indentation.
- Do not execute arbitrary repository-defined commands as part of relation or
  Markdown validation.

## Implementation Boundary

Parser and validator work should land in focused implementation tasks with
fixtures that prove the native notation directly, plus normalization coverage
where shorthand maps to lower-level file or directory attributes.
