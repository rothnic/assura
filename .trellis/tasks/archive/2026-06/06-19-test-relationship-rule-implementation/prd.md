# Implement Test Relationship Rule

## Objective

Implement the first reusable test relationship rule slice from
`docs/goals/assura-rule-test-relationship.md`.

## Scope

- Add explicit config notation for one or more test relationship policies.
- Validate configured source globs, required test globs, ignored/manual test
  reason categories, and fixture families.
- Run the rule from `assura check` and report JSON/text diagnostics with path,
  relationship id, and policy context.
- Add independent passing/failing fixtures and Assura self-dogfood config for a
  bounded supported surface.
- Preserve existing support-matrix, manifest-semantics, release-contract, and
  target-state behavior.

## Non-Goals

- No coverage percentage, mutation testing, or semantic test adequacy claim.
- No broad module deletion or source reorganization.
- No replacement for Rust test execution.

## Definition Of Done

- The rule detects missing required test evidence.
- The rule detects ignored/manual tests without accepted reason categories.
- The rule detects undeclared fixture families.
- `assura check --format json` emits actionable diagnostics for all three
  failure classes.
- Independent review confirms the rule is reusable outside Assura and does not
  overclaim coverage.

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
