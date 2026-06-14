# Assura Config Notation

This is the canonical source for Assura's hand-authored structure notation.
Assura is pre-1.0, so removed alpha notation is not preserved for compatibility.

## Purpose

Assura config should look like the project, file, or document it validates. The
common path should be compact enough to compete with LS-Lint, while richer
attributes stay available in place when a rule needs more detail.

This notation is designed to:

- keep `structure:` as the project tree source of truth;
- put policy where the artifact lives;
- avoid a separate `contents:` layer for direct children;
- avoid duplicate `required` and `allowed` declarations for the same item;
- use `exists` cardinality for required, optional, forbidden, and bounded
  direct contents;
- support reusable rule fragments through `rules:` and `use:`;
- validate Markdown outlines with the same visual hierarchy as the document;
- express common source-to-test and code-to-doc relationships without detached
  custom constraints.

## Principles

- `structure:` mirrors the project tree.
- The simple case should be terse. A file, directory, extension, heading, or
  relationship should not require a nested object unless extra attributes are
  needed.
- The detailed case expands in place. Add nested attributes under the same path
  key only when a directive needs more configuration.
- Captures use single braces: `{component}`, `{package}`.
- Removed alpha captures such as `${name}` and `{{name}}` are invalid in
  Assura-authored structure notation.
- `.assura/` is Assura-owned tool state. Users should not need to add it to
  ordinary project-shape excludes just to make closed-world root policies work.

## Direct Structure

Use the tree for normal files and directories. `exists:N` is the concise count
form; a mapping form is available when a node needs more attributes.

```yaml
structure:
  ./:
    extra: false
    README.md: exists:1
    AGENTS.md: exists:1
    Cargo.toml: exists:1
    src/: exists:1
    docs/: exists:1
    .rs: snake_case
```

This expands to the existing internal structure model:

- `extra: false` closes both direct files and direct directories for the scope.
- `README.md: exists:1` requires exactly one direct file named `README.md`.
- `src/: exists:1` requires exactly one direct directory named `src`.
- `.rs: snake_case` applies a naming rule to direct `*.rs` files.

Closed-world checks remain directory-local. `extra: false` does not recursively
make descendants closed-world unless a descendant scope also says so.

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
| `{component}.tsx` | Direct file capture. |
| `{package}/` | Direct directory capture. |

Trailing slash is the authoring convention for directories. Migration from
LS-Lint may accept older directory shapes, but new Assura-authored examples
should include the slash.

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
`exists:0-1` is allowed but not required. A direct child with `exists:0` is not
allowed.

Detailed attributes remain available when shorthand is not enough:

```yaml
structure:
  packages/*/:
    README.md:
      exists: 1
      markdown:
        max_lines: 300
```

## Reusable Rules

Use top-level `rules:` to remove repeated local policy. A rule can be a node
fragment or a tree fragment; `use:` applies it where the structure already is.

```yaml
rules:
  "@readme-standard":
    exists: 1
    markdown:
      outline:
        - Overview
        - Quick Start:
            - Installation
            - ?? Configuration
        - Usage
        - ?? Troubleshooting

  "@agents-standard":
    exists: 1
    markdown:
      outline:
        - Project Guidance
        - Commands
        - Validation
        - ?? Escalation

  "@project-docs":
    README.md: "@readme-standard"
    AGENTS.md: "@agents-standard"

structure:
  ./:
    use: "@project-docs"
    Cargo.toml: exists:1
    src/: exists:1
```

Rules are an authoring convenience. They compile into normal structure and
relationship constraints before validation.

Merge order should be deterministic:

1. Expand `use` references first. If `use` is a list, merge left to right.
2. Apply local keys over referenced keys.
3. Prefer exact keys over pattern keys.
4. Prefer more specific scopes over broader scopes.

## Markdown Outline Notation

Markdown outlines use nested YAML lists and maps. The nesting is the heading
hierarchy; the order is the expected document order.

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
- Nesting derives heading levels. Users should not maintain separate `h2`/`h3`
  depth fields for ordinary documents.
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

## Optional Producers And Required Counterparts

A captured path without `exists` is optional: if it exists, Assura validates any
relationship implied by other paths with the same capture names. A captured path
with `exists:1` becomes the required counterpart for each matching producer.

```yaml
structure:
  src/components/:
    "{component}.tsx":
      use: "@react-component"
    "{component}.test.tsx": exists:1
```

If `src/components/Button.tsx` exists, Assura requires
`src/components/Button.test.tsx`. If no component exists, no test file is
required.

The target still lives where the target artifact appears in the project tree:

```yaml
structure:
  src/components/:
    "{component}.tsx":
      use: "@react-component"

  tests/components/:
    "{component}.test.tsx": exists:1
```

## Named Needs And Providers

Use `needs:` and `provides:` when one source can be satisfied by more than one
kind of artifact. Provider alternatives also live where the provider artifact
appears.

```yaml
rules:
  "@package-standard":
    README.md: exists:1
    AGENTS.md: exists:1
    src/: exists:1

structure:
  packages/:
    "{package}/":
      use: "@package-standard"
      needs: doc

  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
      markdown:
        outline:
          - Overview
          - Public API
          - ?? Examples

  docs/:
    required: false
    packages.md:
      sections:
        "{package}":
          provides: doc
          markdown:
            outline:
              - Overview
              - Public API
              - ?? Examples
```

For each package directory, either `docs/packages/<package>.md` or a heading
named `<package>` in `docs/packages.md` satisfies the `doc` need.

## When To Use Nested Attributes

Prefer shorthand for ordinary existence, naming, optional producers, required
counterparts, and optional headings. Use nested attributes only when the
directive needs additional detail, such as severity, Markdown heading
constraints, or future custom validators.

```yaml
structure:
  docs/:
    "{topic}.md":
      exists: 1
      markdown:
        outline:
          - Summary
          - Status
          - ?? Decisions
```

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
8. Code-to-test relation: for every matched source capture, require the matching
   test artifact where the test appears in the project tree.
9. Code-to-doc relation: for every matched package directory, require either a
   dedicated docs file or a package section in an aggregate docs file.

## Relationship Boundary

`extensions.custom_constraints` remains an experimental first-party execution
surface for specialized constraints. It is not the preferred notation for common
repo relationships. Common source-to-test, package-to-doc, and aggregate-section
relationships belong in `structure:` through captures, `exists:1`, `needs:`,
and `provides:`.

## Non-Goals

- Do not require users to write `children:` for normal structure or heading
  nesting.
- Do not introduce a `contents:` block for ordinary direct files/directories.
- Do not preserve separate `required` and `allowed` declarations as the common
  notation for the same child.
- Do not require users to sync Markdown heading depth numbers with outline
  indentation.
- Do not execute arbitrary repository-defined commands as part of relationship
  or Markdown validation.
- Do not preserve removed alpha notation for backwards compatibility.
