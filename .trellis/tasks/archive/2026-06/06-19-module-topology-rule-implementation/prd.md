# Module Topology Rule Implementation

## Objective

Implement the first reusable module-topology rule slice from
`docs/goals/assura-rule-module-topology.md`.

## User Certainty Bar

A Rust maintainer can configure module-family ownership/status policy and get
actionable `assura check` diagnostics when public modules or exports are
unclassified, conflict with topology status, or reference module roots that no
longer exist.

## Scope

- Add explicit `extensions.module_topologies` config notation.
- Add semantic validation for topology ids, status values, owner/purpose text,
  root files/directories, public export names, and duplicate/conflicting rows.
- Implement a bounded Rust module/export inventory for configured roots,
  including `mod`, `pub mod`, and top-level `pub use` forms needed by
  `src/lib.rs` and fixture roots.
- Report unclassified public module families, public export/status conflicts,
  and missing configured module roots with file/module/policy context.
- Add reusable passing/failing fixtures and CLI integration tests.
- Dogfood the rule on Assura's current public/internal module split without
  deleting, moving, or renaming modules.
- Preserve existing support-matrix, manifest-semantics, test-relationship,
  release-contract, target-state, and evidence behavior.

## Non-Goals

- No broad Rust parser beyond bounded declaration/export forms.
- No public API semver guarantee for pre-1.0 exports.
- No module deletion, movement, or large refactor in this slice.
- No replacement for support-matrix classification.
- No docs lifecycle or stale-claim detector.

## Definition Of Done

- Config model, semantic validation, compiled artifact portability, and runtime
  check support module-topology policies.
- Passing fixture covers supported/current-product modules and explicitly
  internal or experimental modules.
- Failing fixtures cover an unclassified public module, a missing configured
  module root, and a public export/status conflict.
- Assura self-check passes with dogfood module-topology policies configured.
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
cargo xtask docs
git diff --check
```
