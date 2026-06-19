# Cargo Manifest Semantics Rule Implementation

## Objective

Implement the first reusable Cargo manifest-semantics rule slice from
`docs/goals/assura-rule-cargo-manifest-semantics.md`.

## User Certainty Bar

A Rust workspace maintainer can configure public/internal manifest policy and
receive actionable `assura check` diagnostics when package metadata, publish
settings, inherited version/MSRV/license, keywords, descriptions, or configured
binary entries drift from policy.

## Scope

- Add explicit `extensions.manifest_semantics` config notation.
- Add semantic validation for ids, paths, status/publish policy, expected
  metadata fields, required/forbidden terms, and duplicate entries.
- Implement runtime checks through `assura check` with JSON diagnostics that
  identify file, package, field, policy, and manifest id.
- Add reusable passing/failing fixtures and CLI integration tests.
- Dogfood the rule on Assura's root `Cargo.toml` and
  `crates/assura-check-cli/Cargo.toml`.
- Preserve existing support-matrix, release-contract, target-state, and
  evidence behavior.

## Non-Goals

- No optional dependency usage analysis.
- No license/source policy beyond configured manifest fields.
- No semver compatibility checks.
- No replacement for Cargo, `cargo machete`, `cargo audit`, support-matrix, or
  release-contract checks.

## Definition Of Done

- Config model, semantic validation, compiled artifact portability, and runtime
  check support manifest-semantics policies.
- Passing fixture covers public root and internal workspace crates.
- Failing fixtures cover missing required metadata, forbidden claim text,
  incorrect internal publish policy, and mismatched binary metadata.
- Assura self-check passes with dogfood manifest policies configured.
- Independent review is completed before PR merge and valid findings are
  addressed or documented.

## Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask evidence
git diff --check
```
