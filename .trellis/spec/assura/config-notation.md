# Assura Config Notation

This is the canonical source for Assura's hand-authored structure notation.
Assura is pre-1.0, so removed alpha notation is not preserved for compatibility.

## Purpose

Assura config should look like the project, file, or document it validates. The
common path should be compact enough to compete with LS-Lint, while richer
attributes stay available in place when a rule needs more detail.

The notation keeps `structure:` as the project-tree source of truth, puts policy
where the artifact lives, and avoids detached `contents:`, `required`, and
`allowed` layers. It uses `exists` for direct-child cardinality, `rules:` and
`use:` for reuse, nested Markdown outlines for document shape, and named
relationships for source-to-test or code-to-doc contracts.

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

## Notation Change Gates

Any future change to Assura-authored notation must update the whole user-facing
surface in the same goal:

- Update public docs, website and generated examples, fixtures, and test-case
  configs that teach or exercise the changed notation.
- Remove superseded alpha notation rather than preserving backwards
  compatibility shims. Assura is pre-1.0, so compatibility exceptions require a
  documented support-policy reason and an explicit removal plan.
- Run performance gates for changes affecting parsing, normalization, compiled
  artifacts, traversal, relationships, or fast-path eligibility. Record and
  bound any justified cost with checked evidence.
- Keep LS-Lint compatibility claims separate from Assura-authored notation
  robustness claims.

## Pattern-Scoped File Directives

Reusable file shorthand, directory-scope glob reach, pattern inheritance,
specificity, compiled artifacts, and `assura explain` output are specified in
[Pattern-Scoped File Directives](./pattern-scoped-file-directives.md).

## Scenario: First-Party Extension Contract Changes

### 1. Scope / Trigger

- Trigger: adding, removing, or changing fields under a first-party
  `extensions.*` config family.
- Applies to source config structs, check behavior, generated onboarding
  templates, docs, fixtures, and compiled config artifacts.

### 2. Signatures

- Source config structs live under `src/config/config/extensions/`.
- Check implementation lives under `src/cli/check/`.
- Compiled artifact portability lives under
  `src/cli/check/compiled_artifact_*.rs`.
- Binary artifact compatibility is guarded by
  `COMPILED_CONFIG_SCHEMA_VERSION` in `src/cli/check/compiled_artifact.rs`.

### 3. Contracts

- New optional extension fields must deserialize safely from YAML and preserve
  missing-field behavior with `#[serde(default)]` or `Option<T>`.
- New fields that affect checking must round-trip through compiled artifacts.
- Any portable artifact payload shape change must bump
  `COMPILED_CONFIG_SCHEMA_VERSION` so old artifacts fail as incompatible
  instead of deserialization-invalid.
- Portable artifact payload structs must not reuse source config structs that
  rely on skipped serde fields such as `skip_serializing_if`. Postcard is a
  non-self-describing binary format, so skipped fields can shift later bytes
  into the wrong field and produce invalid-option-discriminant errors.
- Generated configs that opt into the extension must self-check without
  advisory drift unless the generated policy intentionally uses non-blocking
  severity.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| New source YAML field omitted | Preserve previous default behavior. |
| New source YAML field configured | Runtime check observes the field. |
| Compiled artifact created before payload-shape change | Reject as incompatible after schema bump. |
| Compiled artifact created after payload-shape change | Runtime check observes every new portable field. |
| Portable artifact includes optional or defaulted source fields | Serialize every binary field explicitly through a dedicated portable struct. |
| Generated template enables the extension | `assura check --format json` succeeds on generated output. |

### 5. Good/Base/Bad Cases

- Good: `extensions.agent_guidance.skill_routing_section` works in direct
  checks and compiled checks, with a schema bump for the portable payload.
- Base: omitting the new field leaves older projects unaffected.
- Bad: direct checks enforce a new field, but compiled checks silently drop it
  because the portable conversion was not updated.
- Bad: a compiled artifact stores a source config struct with
  `skip_serializing_if`, then `assura-check-compiled` fails with
  `invalid compiled config: Found an Option discriminant that wasn't 0 or 1`.

### 6. Tests Required

- Unit or integration coverage for direct check behavior.
- Negative coverage for the new diagnostic branch.
- Generated-template coverage when onboarding emits the field.
- Compiled CLI coverage that proves each new portable field still affects
  validation after `assura-check-compile-config`.

### 7. Wrong vs Correct

Wrong:

```rust
const COMPILED_CONFIG_SCHEMA_VERSION: u32 = 18;
```

after adding a new portable extension field.

Correct:

```rust
const COMPILED_CONFIG_SCHEMA_VERSION: u32 = 19;
```

Wrong:

```rust
struct PortableConfig {
    models: Option<ContentModelConfig>,
}
```

where `ContentModelConfig` is a YAML-facing type with skipped serde fields.

Correct:

```rust
struct PortableConfig {
    models: Option<PortableContentModelConfig>,
}
```

with explicit `From` conversions between source and portable structs.

## Notation Coverage Proof

The checked matrix starts with LS-Lint-equivalent naming, extension,
closed-world, ignore, and direct-child presence cases, then covers Assura-native
`exists`, captures, relationships, Markdown outlines, and reusable rules. It
must prove equivalent checks remain comparably concise and efficient, while
Assura-native reuse expands policy breadth without hidden behavior. Independent
review confirms both claims from executable fixtures and measured evidence.

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
| `.md` | Cascading extension rule in this scope and inheriting descendants. |
| `.test.ts` | Cascading subextension rule in this scope and inheriting descendants. |
| `"./*.ts"` | Explicit direct-root file glob. |
| `"./**/*.ts"` | Explicit recursive file glob from the root. |
| `"*.ts"` inside `src/` | Explicit direct-file glob resolved relative to `src/`. |
| `"**/*.ts"` inside `src/` | Explicit recursive file glob resolved relative to `src/`. |
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

Literal hierarchy is concise by default:

- an exact literal file or directory mapping without `exists` means
  `exists:1`;
- an exact literal directory scalar rule such as `web/: "@web-app"` also means
  `exists:1`;
- extension, capture, and glob keys are match-only unless they declare an
  explicit direct-child `exists` count;
- the root `./` always exists and cannot declare its own cardinality.

These are defaults for one cardinality model, not a separate `required`
concept. Public structure notation rejects `required`. Replace
`required: true` with an omitted literal default or `exists:1`, and replace
`required: false` with `exists:0-1`.

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

When concise direct-child `exists` keys and expanded `files:` or `directories:`
bundles appear in the same node, Assura merges them. The expanded bundle must
not erase the count/allow entries generated by the concise child keys.

Explicit file-glob `exists` is a direct-child count, so `"./*.ts": exists:1`
is valid but `"./**/*.ts": exists:1` is rejected. Captured-file `exists:1`
keeps its per-source counterpart meaning. Directory captures such as
`"{package}/": exists:1-20` count direct children; multi-segment counts such
as `packages/*/src/: exists:1` are rejected. Express that hierarchy as nested
`packages/`, `{package}/`, and `src/` nodes so each count has one parent.

Exact literal files and directories only accept `0`, `0-1`, or `1`. A literal
cannot exist more than once. `exists:0` cannot carry child policy because a
forbidden directory cannot also define active descendants. An optional
`exists:0-1` directory may carry child policy, which is applied only when that
directory exists.

Detailed attributes remain available when shorthand is not enough:

```yaml
structure:
  packages/*/:
    README.md:
      exists: 1
      markdown:
        max_lines: 300
```

Reusable node directives keep repeated attribute groups concise. A scalar rule
reference and the expanded in-place mapping must normalize equivalently:

```yaml
rules:
  "@source-file":
    naming: kebab-case
    max_lines: 500

structure:
  src/:
    .ts: "@source-file"
    .tsx: "@source-file"
```

```yaml
structure:
  src/:
    .ts:
      naming: kebab-case
      max_lines: 500
    .tsx:
      naming: kebab-case
      max_lines: 500
```

Directive-attached `naming`, `max_lines`, and `max_size` remain keyed to the
file pattern. The normalizer stores the limit values in
`files.max_lines_patterns` and `files.max_size_patterns`; the checker selects
the most specific matching pattern before falling back to directory-wide
`files.max_lines` or `files.max_size`. Extension shorthand such as `.ts`
applies through inheriting descendants. Explicit file globs preserve configured
depth: `./*.ts` is direct at the root, `./**/*.ts` is recursive, and globs
authored inside a nested structure scope are relative to that scope. Local
patterns merge and override by key, and `inherit: false` resets inherited
policy. Put attributes under the node's `files:` mapping when they should be
defaults for every file in the scope rather than only files matching one
directive.

Directory scope keys support LS-Lint-style hierarchy globs:

```yaml
structure:
  packages/*/src/:
    .ts: "@source-file"
  packages/**/generated/:
    inherit: false
```

Here `*` matches one path segment and `**` matches any number. The same segment
rules apply to explicit file globs. Use `assura explain <path>` to inspect every
applied scope and the winning normalized naming, line, and size patterns; files
without a matching directive report `matched_file_patterns=none`, and a scope
that discards inherited policy is marked `(reset)` in text output.

## Reusable Rules

Use top-level `rules:` to remove repeated local policy. A rule can be a node
fragment or a tree fragment; `use:` applies it where the structure already is.
Tree fragments may contain ordinary directory-node attributes such as
`inherit`, `files`, or `directories` alongside child path keys, matching inline
`structure:` node behavior. Assura also provides named built-in rules for
opinionated common domains.

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

A scalar rule reference on an exact or pattern directory key is concise tree
reuse:

```yaml
rules:
  "@web-app":
    package.json: exists:1
    src/: exists:1

structure:
  ./:
    apps/:
      web/: "@web-app"
```

This is equivalent to `web/: { use: "@web-app" }`. Use the mapping form when
local keys need to override or extend the reusable tree. A scalar rule used on
a file/extension key must resolve to a node fragment, while a directory key
requires a tree fragment. Unknown references, type mismatches, and cycles are
rejected while loading the config.

Merge order should be deterministic:

1. Expand `use` references first. If `use` is a list, merge left to right.
2. Apply local keys over referenced keys.
3. Prefer exact keys over pattern keys.
4. Prefer more specific scopes over broader scopes.

Built-in agentic project best practices should be used instead of enumerating
root guidance files and every project-local skill:

```yaml
structure:
  ./:
    use: "@agentic-project"
```

The built-in `@agentic-project` requires root `AGENTS.md`, allows an optional
`.agents/` directory, applies `@agents-dir` when that directory exists, and
allows the `.assura/` config directory when root direct-content policies are
closed.

Use the narrower agent-directory rule when the project only needs to validate
a `.agents/` subtree:

```yaml
structure:
  .agents/:
    use: "@agents-dir"
```

The built-in `@agents-dir` allows optional `skills/` content and composes
`@agent-skill-dir` for each `.agents/skills/{skill}/`,
`.agents/skills/built-in/{skill}/`, and `.agents/skills/custom/{skill}/`
directory that exists. Each skill directory is an inheritance boundary, must
be kebab-case, must contain `SKILL.md`, and receives the standard skill file
line/size limits. Optional resource subdirectories such as `agents/`,
`references/`, `scripts/`, and `assets/` are validated when present with
kebab-case naming and the same file size/line limits. Exact file rules can
stay concise through rule references such as `SKILL.md: "@agent-skill-file"`;
project configs should not repeat expanded file-bundle YAML for this common
case.

Global or user-level skills installed outside the checked repository are out of
scope for `@agents-dir`. A third-party skill copied, vendored, or linked into
`.agents/skills/**` is project-local guidance for validation purposes: the
project owns a bounded `SKILL.md` entrypoint there, even if the deeper content
comes from upstream. If an upstream skill is too large or should remain
vendor-owned, keep it global and add a small project-local wrapper skill that
routes agents to the external/global source or to deeper local `references/`,
`scripts/`, and `assets/` content.

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

## Markdown Frontmatter Field Ownership

### 1. Scope / Trigger

- Trigger: Assura-authored config distinguishes generic Markdown document
  style from typed content records.
- Applies to `structure.<scope>.markdown` bundles loaded from
  `.assura/config.yml`.

### 2. Signatures

- Supported generic Markdown presence field:
  `markdown.require_frontmatter: bool`.
- Supported first-slice generic Markdown lint field:
  `markdown.lint_trailing_spaces: bool`.
- Unsupported legacy typed-field field:
  `markdown.required_fields: string[]`.
- Typed frontmatter fields use top-level `models`, `collections`, and
  `relations` instead.

### 3. Contracts

- `markdown.require_frontmatter: true` only requires a YAML frontmatter block
  to be present in matching Markdown files.
- `markdown.required_fields` is rejected during config semantic validation with
  guidance to content runtime models and collections.
- Structure Markdown validation must not emit `markdown_frontmatter_field` for
  typed fields.
- `markdown.lint_trailing_spaces: true` reports blank Markdown lines that
  contain spaces or tabs as `markdown_trailing_spaces`. The first safe fix
  operation removes only this blank-line whitespace class and must not rewrite
  content-line hard breaks.
- Markdown heading depth, required sections, and `markdown.outline` remain
  Assura-owned Markdown structure behavior.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Markdown file lacks frontmatter and `require_frontmatter: true` | `markdown_frontmatter` violation |
| Config contains `markdown.required_fields` | Config error naming `models` and `collections` |
| Markdown frontmatter record lacks a model-required field | `content_runtime:invalid_object_shape` or model-field finding |
| Blank Markdown line has spaces or tabs and `lint_trailing_spaces: true` | `markdown_trailing_spaces` violation |
| Markdown headings do not satisfy `outline` | `markdown_outline` violation |

### 5. Good/Base/Bad Cases

- Good: a `Goal` Markdown collection defines required `title` in the runtime
  model schema and validates through `collections.goals`.
- Base: an ordinary Markdown style policy uses `require_frontmatter: true`
  without typed field checks.
- Good: a Markdown style policy opts into `lint_trailing_spaces: true` and
  `assura fix markdown` removes spaces from otherwise blank lines only.
- Bad: a structure Markdown bundle declares `required_fields: [title]` and
  duplicates the content model.

### 6. Tests Required

- CLI regression proving `markdown.required_fields` is rejected with model and
  collection guidance.
- Content runtime regression proving a missing model-required Markdown
  frontmatter field is reported through modeled collection validation.
- Markdown regression proving generic `require_frontmatter` still reports
  missing frontmatter.
- Markdown lint/fix regression proving `lint_trailing_spaces` reports and fixes
  blank-line whitespace while preserving frontmatter and body content.
- Outline/heading tests proving Markdown structure behavior remains unchanged.

### 7. Wrong vs Correct

#### Wrong

```yaml
structure:
  docs/:
    markdown:
      require_frontmatter: true
      required_fields:
        - title
```

#### Correct

```yaml
structure:
  docs/:
    markdown:
      require_frontmatter: true

models:
  validation_artifact: schemas/content_runtime.schema.json

collections:
  goals:
    class: Goal
    path: docs/goals/*.md
    adapter: markdown_frontmatter
    data: frontmatter
```

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
Required captured children inside a captured directory remain ordinary
structure requirements for that directory; they are not treated as counterparts
for the directory itself.

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

If the same capture name is reused in separate scopes, Assura pairs same-scope
counterparts first. If there is no same-scope counterpart and multiple
cross-tree counterparts could satisfy one producer, config loading fails as
ambiguous instead of silently choosing one.

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
    exists: 0-1
    "{package}.md":
      provides: doc
      markdown:
        outline:
          - Overview
          - Public API
          - ?? Examples

  docs/:
    exists: 0-1
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

Captured entries that only declare `provides:` are providers, not producers.
Duplicate provider alternatives for the same need, capture set, provider kind,
path, and section are configuration errors. Missing relationship diagnostics
name the producer, source pattern, declaring structure entry, provider kind, and
expanded provider or counterpart path.

## When To Use Nested Attributes

Prefer shorthand for ordinary existence, naming, reusable node directives,
optional producers, required counterparts, and optional headings. Use nested
attributes only when the directive needs additional detail, such as severity,
Markdown heading constraints, or future custom validators.

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
2. LS-Lint-equivalent naming and extension rules: apply direct-file naming
   rules such as `snake_case`, exact extension rules such as `.rs`, and ignore
   policy without requiring more ceremony than the LS-Lint baseline.
3. Monorepo package contract: every `packages/*/` directory has `README.md`,
   `AGENTS.md`, `package.json`, and `src/` without duplicating those rules at
   each package path.
4. Optional singleton files: allow but do not require files such as
   `CHANGELOG.md` through `exists:0-1`.
5. Forbidden files: reject root scratch files or generated artifacts through
   `exists:0` or forbidden pattern attributes.
6. Markdown outline contract: validate ordered nested headings with optional
   sections marked by `?? `.
7. Question headings: allow required headings such as `Why Assura?` and
   optional headings such as `?? Why Assura?`.
8. Escaped/custom headings: support object-form nodes for headings that collide
   with shorthand or need custom match attributes.
9. Code-to-test relation: for every matched source capture, require the matching
   test artifact where the test appears in the project tree.
10. Code-to-doc relation: for every matched package directory, require either a
   dedicated docs file or a package section in an aggregate docs file.

## First-Party Extension Policies

First-party extension policies live under `extensions:` when they model
cross-file semantics that do not fit the directory tree itself. They must stay
explicit and deterministic: configured inputs, configured expected values, and
actionable diagnostics.

Cargo manifest semantics use `extensions.manifest_semantics`:

```yaml
extensions:
  manifest_semantics:
    - id: cargo_workspace
      severity: high
      manifests:
        - path: Cargo.toml
          package: assura
          role: public
          version: "0.1.0"
          rust_version: "1.70.0"
          license: "MIT OR Apache-2.0"
          publish: public
          description_required_terms:
            - Structure-first
          description_forbidden_terms:
            - dependency graph validation
          keywords:
            - structure
          binaries:
            - assura
```

Use this rule for Cargo package metadata, publish policy, expected keywords,
description claim checks, and declared `[[bin]]` names. Do not use it as a
replacement for Cargo, dependency usage tools, license/source policy tools, or
semver compatibility checks.

Support matrix policies use `extensions.support_matrices`:

```yaml
extensions:
  support_matrices:
    - id: public_surface
      severity: high
      command_contracts:
        - .assura/command-surface.yml
      rust_exports:
        - src/lib.rs
      docs_claim_sources:
        - path: docs/compatibility-and-surface.md
      manifest_policies:
        - cargo_workspace
      entries:
        - surface: "command:assura check"
          status: supported
        - surface: "rust:cli"
          status: supported
        - surface: "package:assura"
          status: supported
        - surface: "binary:assura"
          status: supported
```

Use this rule for explicit public surface support classification. Command
surfaces come from configured command-surface contracts. Rust export surfaces
come from top-level `pub mod` and `pub use` families in configured files. Docs
claim sources are bounded Markdown files with a table containing `Command` or
`Surface` and `Status`, `Level`, or `Support` columns; only deterministic table
rows are read, and broad prose is ignored. Manifest policy sources reference
configured `extensions.manifest_semantics` policy ids and use their declared
`package` and `binaries` values as `package:<name>` and `binary:<name>`
surfaces. Support statuses are `supported`, `experimental`, `internal`,
`roadmap`, and `unsupported`. Do not use it as a broad semantic-versioning
guarantee, dependency usage analyzer, manifest metadata validator, or docs
prose classifier.

Test relationship policies use `extensions.test_relationships`:

```yaml
extensions:
  test_relationships:
    - id: supported_surface_tests
      severity: high
      relationships:
        - source: src/cli/check/*.rs
          required_tests:
            - tests/custom_constraints_tests.rs
      fixture_roots:
        - tests/fixtures
      fixture_families:
        - path: tests/fixtures/test-relationship
          owner: validation-tests
          purpose: reusable test relationship rule coverage
      allowed_ignore_reasons:
        - manual_performance_audit
      ignored_tests:
        - path: tests/manual_performance.rs
          test: manual_performance_audit
          reason: manual_performance_audit
```

Use this rule for explicit source-to-test evidence, accepted ignored/manual
test files, and fixture-family ownership. Do not use it as a coverage
percentage, mutation-test, or semantic test-adequacy claim.

Module topology policies use `extensions.module_topologies`:

```yaml
extensions:
  module_topologies:
    - id: public_rust_modules
      severity: high
      rust_exports:
        - src/lib.rs
      modules:
        - family: cli
          status: supported
          owner: assura-maintainers
          purpose: supported CLI implementation
          roots:
            - src/cli
          public_exports:
            - cli
        - family: experimental_graph
          status: internal
          owner: assura-maintainers
          purpose: contained graph experiment
          roots:
            - src/experimental_graph
          visibility: internal
```

Use this rule for explicit Rust module-family ownership, root existence, and
bounded public export classification. `visibility: internal` means the family
must not appear as a public export in configured `rust_exports` files. Do not
use it as a broad Rust parser, public API semver guarantee, or refactoring
mandate.

Docs lifecycle policies use `extensions.docs_lifecycles`:

```yaml
extensions:
  docs_lifecycles:
    - id: project_docs
      severity: medium
      active:
        - docs/**/*.md
        - website/src/content/docs/**/*.mdx
      historical:
        - docs/archive/**
      require_frontmatter_status:
        - docs/analysis/*.md
        - docs/goals/*.md
      allowed_statuses:
        - active
        - planned
        - completed
        - archived
        - historical
      claim_patterns:
        - id: performance_current
          pattern: "2x"
          evidence_files:
            - benches/history/current.json
            - website/public/data/performance/current.json
      historical_exceptions:
        - docs/archive/**
```

Use this rule for explicit active/historical documentation boundaries, required
frontmatter lifecycle status, historical-reference exceptions, and deterministic
claim tokens that must have current evidence files. Claim patterns are literal
tokens or glob-style token patterns. Do not use it as broad natural-language
stale-prose detection or automatic archival.

Release contract policies use `extensions.release_contracts`:

```yaml
extensions:
  release_contracts:
    - id: release_archives
      severity: high
      artifacts:
        - name: assura-linux-x86_64.tar.gz
          checksum_sidecar: true
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/release-notes.md
      installer_files:
        - website/public/install.sh
      allowed_url_branches:
        - master
```

Use this rule for release artifact, checksum, workflow, docs, installer, and
branch-reference synchronization. Do not use it as a release publisher,
artifact builder, or substitute for release automation.

Repository-reference policies use `extensions.repository_references`:

```yaml
extensions:
  repository_references:
    - id: source_refs
      severity: high
      paths:
        - "src/**"
      frontmatter_fields:
        - source_documents
        - related
```

Use this rule for opt-in diagnostics when supported source, comment, docstring,
string-literal, or configured Markdown frontmatter path references point at
missing files, missing Markdown anchors, or invalid line anchors. Configured
frontmatter fields are also repository-reference graph facts, so they appear in
`assura content references`, context packs, and unresolved-reference
agent-query output. Do not use this rule as the source of truth for
lower-confidence repository-reference candidates; those remain graph context
through `assura content references` and context packs.

## Relationship Boundary

`extensions.custom_constraints` remains an experimental first-party execution
surface for specialized constraints. It is not the preferred notation for common
repo relationships. Common source-to-test, package-to-doc, and aggregate-section
relationships belong in `structure:` through captures, `exists:1`, `needs:`,
and `provides:`.

`extensions.relationships` is an internal generated first-party policy family.
It is normalized from concise `structure` notation and should not be the normal
hand-authored surface. Users should write the structure entries that produce the
relationship instead.

For the full support boundary between first-party `extensions.*` policies,
local CLI contracts, internal Rust APIs, and deferred public plugin APIs, see
[`docs/extension-api-boundaries.md`](../../../docs/extension-api-boundaries.md).

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
