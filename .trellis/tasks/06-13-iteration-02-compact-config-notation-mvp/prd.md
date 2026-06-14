# Iteration 02 native config notation MVP

## Goal

Implement the first Policy Expressiveness native config notation MVP from
`.trellis/spec/assura/config-notation.md` without changing the stable runtime
validation model. The parser should accept concise structure path keys,
quoted `"@rule"` references, `use` tree-fragment merges, and `exists`
cardinality shorthand, then normalize that notation into the structure-first
`Config` model.

## What I Already Know

- The previous config notation spec task is merged and archived in this branch.
- Current public runtime config loads through `ConfigLoader::parse_validated`
  and deserializes YAML directly into `Config`.
- The existing `Config` model already supports the target lower-level fields:
  `children`, `files.exists`, `directories.exists`, `files.naming_patterns`,
  `directories.exists`, `allow_extra`, and direct structure scopes.
- The target spec requires stable examples to keep `"@rule"` values quoted
  because bare `@` values are invalid YAML.
- This MVP must not implement Markdown outlines or code-to-doc relations.

## Requirements

- Accept direct file keys under `structure` nodes, such as
  `README.md: exists:1`.
- Accept exact directory keys under `structure` nodes, such as
  `src/: exists:1`.
- Accept extension and subextension keys under `structure` nodes, such as
  `.md: kebab-case` and `.test.ts: snake_case`.
- Accept directory-scope keys under `structure`, such as `packages/*/`, as
  non-required validation scopes unless explicit existence rules require a
  match.
- Accept quoted rule references, such as `README.md: "@readme-standard"`.
- Accept `use: "@project-docs"` and `use: ["@a", "@b"]` tree-fragment merges.
- Merge `use` fragments left to right, then apply local keys over referenced
  keys.
- Infer node fragments versus tree fragments and reject invalid shape mixing or
  wrong-use references with clear config errors.
- Normalize `exists:1`, `exists:0-1`, `exists:0`, and `exists:N-M` shorthand
  into existing direct file/directory count fields.
- Replace superseded notation and stale docs instead of maintaining parallel
  compatibility paths.

## Acceptance Criteria

- [x] Parser accepts a native root project contract using path keys and
  `exists` shorthand.
- [x] Parser accepts reusable node and tree fragments through quoted
  `"@rule"` references and `use`.
- [x] `use` merge order is covered by tests.
- [x] Wrong fragment kind and unknown rule references produce clear errors.
- [x] CLI `assura check` passes/fails against a native config fixture without
  first converting it through an external migration command.
- [x] Assura self-check still passes with the current native config surface.
- [x] Markdown outline and relation syntax remain explicitly out of scope.

## Out Of Scope

- Markdown `outline:` parsing or validation.
- Code-to-document `relations:` parsing or validation.
- Unquoted `@rule` value preprocessing.
- Migrating Assura's own `.assura/config.yml` to the newest native notation.
- Changing LS-Lint migration behavior beyond shared helper reuse.

## Technical Notes

- Target spec: `.trellis/spec/assura/config-notation.md`.
- Current config model: `src/config/config.rs` and
  `src/config/config/bundles.rs`.
- Loader entrypoint: `src/config/loader.rs`.
- Existing LS-Lint direct-count helpers: `src/config/ls_compat.rs` and
  `src/config/ls_compat/validation.rs`.
- Good candidate implementation shape: parse YAML into `serde_yaml::Value`,
  normalize native structure/rules into a standard `Config`-compatible value,
  then deserialize and run existing semantic validation.
