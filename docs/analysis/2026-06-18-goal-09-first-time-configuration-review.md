---
id: analysis-2026-06-18-goal-09-first-time-configuration-review
type: analysis
title: Goal 09 first-time configuration review
status: active
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-09-first-time-configuration-authoring.md
  - .trellis/spec/assura/config-notation.md
  - website/src/content/docs/guides/quickstart.md
  - website/src/content/docs/examples/basic-setup.md
  - tests/structure_config_notation_tests.rs
---

# Goal 09 First-Time Configuration Review

## Revalidation

Status: valid.

The goal is still needed after PR #50 because the public quickstart and basic
setup pages existed, but the first-run path did not yet prove that a new user
could author concise tree-shaped `structure:` config without falling back to
the older `files:` and `directories:` reference model. The current config
notation spec is the source of truth, and Goal 09 should not reopen the
completed canonical relationship notation work.

## Simulated First Run

### Small Rust CLI/library

Starting point:

- `Cargo.toml`
- `README.md`
- `src/main.rs`

First useful policy:

- require root `README.md`, `Cargo.toml`, and `src/`;
- keep root closed with `extra: false`;
- allow an optional lockfile;
- enforce `snake_case` Rust files under `src/`;
- ignore `target/**`.

Result: this path can be expressed with concise `structure:` notation and is
covered by `first_time_rust_project_config_accepts_minimal_useful_shape`.

### Package-Style Project

Starting point:

- `packages/core/README.md`
- `packages/core/package.json`
- `packages/core/src/index.ts`
- `docs/packages/core.md`

First useful policy:

- reuse a package standard with `rules:` and `use:`;
- capture package names from `packages/{package}/`;
- require package documentation through `needs: doc`;
- satisfy documentation with `docs/packages/{package}.md`;
- keep generated output excluded.

Result: this path can be expressed with concise `structure:` notation and is
covered by `first_time_package_project_config_accepts_reusable_rules_and_docs`.

## Notation Matrix

| Use case | First-path notation | Status |
| --- | --- | --- |
| LS-Lint-equivalent naming | `.rs: snake_case`; `.ts: kebab-case` | Covered in docs and tests |
| LS-Lint-equivalent extension policy | `.rs`, `.ts`, and lockfile count rules | Covered in docs and tests |
| Closed-world root | `extra: false` | Covered in docs and tests |
| Ignore/generated output | `exclude: ["target/**", "node_modules/**", "dist/**"]` | Covered in docs |
| Direct-child presence | `README.md: exists:1`; `src/: exists:1` | Covered in docs and tests |
| Assura-native exists | `*.lock: exists:0-1` | Covered in docs and tests |
| Captures | `"{package}/"` and `"{component}.tsx"` | Covered in docs and tests |
| Relationships | `needs: doc`; `provides: doc` | Covered in docs and tests |
| Markdown outlines | `markdown.outline` in the config reference | Documented; runtime proof remains Goal 11 |
| Reusable rules | `rules:` with `use:` | Covered in docs and tests |

## Fixes Made

- Quickstart now shows a first useful hand-authored config instead of stopping
  at `assura init`.
- Basic setup now uses concise tree-shaped notation instead of leading with
  nested `files:` and `directories:`.
- Advanced examples now show reusable rules and relationship notation before
  experimental custom constraints.
- The configuration reference now includes a first-time notation matrix that
  separates LS-Lint-style cases from Assura-native extensions.
- The custom constraints example now leads with concise project-shape notation
  before showing the experimental extension surface.
- The older config-v2 reference is labeled as legacy reference material and
  points first-time users to the current configuration reference.
- The `assura init` generated starter config now uses concise tree-shaped
  notation instead of nested `files:`, `directories:`, and `children:`.
- Executable temp-project tests prove the Rust and package-style examples.

## Deferred Gaps

- Markdown outline runtime proof remains Goal 11. Goal 09 documents the
  notation shape but does not implement generic Markdown outline validation.
- Performance proof for notation changes remains scoped to the existing
  performance gate unless future Goal 09 edits change runtime behavior.
