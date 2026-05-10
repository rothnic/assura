# Clippy Baseline Cleanup

## Goal

Clean the repository-wide Clippy warning baseline so `cargo clippy --all-targets --all-features -- -D warnings` passes locally and can return to being a blocking CI gate.

## What I already know

- PR #1 merged Assura self-enforcement and Trellis governance.
- PR #2 merged repository-wide rustfmt cleanup.
- PR #3 merged CI cache improvements and made Clippy advisory while the known baseline is dirty.
- Current CI is expected to keep Rustfmt, check, tests, docs, and coverage green.
- Clippy currently runs with `-D warnings` but is advisory.
- Assura self-check still reports known advisory baseline violations and must not be cleaned up in this task.
- The active branch is `codex/clippy-baseline-cleanup`, created from latest `origin/master`.

## Requirements

- Run `cargo clippy --all-targets --all-features -- -D warnings` and classify the findings.
- Fix Clippy warnings in focused batches.
- Prefer mechanical fixes first: unused imports or variables, `strip_prefix` / `strip_suffix`, derivable `Default`, collapsible `if`, `map_flatten`, `unnecessary_map_or`, and `useless_vec`.
- Treat semantic-risk lints separately, especially `char_indices_as_byte_indices` in markdown parsing.
- Do not refactor broadly or change runtime behavior unless required by a lint.
- If Clippy becomes clean, make Clippy blocking in CI again by removing advisory `continue-on-error` behavior.
- If Clippy becomes clean, update `.trellis/spec/assura/tooling-stabilization.md` to close the Clippy deferred baseline.

## Acceptance Criteria

- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test --all-targets --quiet` passes.
- [x] `.github/workflows/ci.yml` makes Clippy blocking again if the baseline is clean.
- [x] `.trellis/spec/assura/tooling-stabilization.md` records the Clippy baseline as closed if the baseline is clean.
- [x] Work is committed, pushed, and opened as a focused PR.

## Out of Scope

- Do not fix Assura self-check violations.
- Do not clean legacy docs or source-of-truth issues.
- Do not restore Windows CI.
- Do not address unrelated behavior bugs or broad architecture issues.
- Do not add new dependencies.

## Technical Notes

- Canonical project guidance: `AGENTS.md`, `.trellis/workflow.md`, `.trellis/spec/assura/roadmap.md`, `.trellis/spec/assura/tooling-stabilization.md`.
- The current roadmap puts tooling baseline cleanup next, after Trellis governance and rustfmt cleanup.
- The tooling stabilization spec says the Clippy deferred baseline closes when a dedicated cleanup PR lands and Clippy becomes blocking in CI.
