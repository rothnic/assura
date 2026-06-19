# Docs Lifecycle Rule Implementation

## Goal

Implement the first reusable `extensions.docs_lifecycles` rule slice from
`docs/goals/assura-rule-docs-lifecycle-stale-claims.md`.

## Scope

- Add docs-lifecycle config notation and semantic validation.
- Run the rule through `assura check`.
- Validate configured lifecycle/frontmatter status for active docs.
- Validate active docs do not link to configured historical docs unless an
  exception applies.
- Validate configured claim patterns have declared evidence files.
- Add independent fixtures and CLI integration coverage.
- Dogfood a narrow Assura policy without weakening existing target-state or
  evidence gates.

## Non-Goals

- No broad natural-language stale-prose classifier.
- No automatic archival/deletion.
- No replacement for `cargo xtask target-state` in this slice.
- No remote or GitHub API checks.

## Validation

- `cargo fmt --all -- --check`
- `cargo test --all-targets --quiet`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo xtask target-state`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `git diff --check`
