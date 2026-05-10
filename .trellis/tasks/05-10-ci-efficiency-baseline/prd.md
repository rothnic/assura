# CI Efficiency Baseline

## Goal

Reduce duplicated Rust CI work without changing product behavior or folding
Clippy cleanup into this task.

## What I Already Know

- PR #2 merged the repository-wide rustfmt cleanup.
- Current CI repeats overlapping Rust compilation across separate `check`,
  `clippy`, `test`, `coverage`, and documentation jobs.
- `Rustfmt` is cheap and does not build the crate.
- Linux and macOS tests must remain separate because they run on different
  platforms.
- Clippy currently fails from known baseline debt and should remain a tracked
  deferred item until a focused cleanup lands.

## Requirements

- Keep CI behavior equivalent for the current quality gates.
- Improve cache reuse for Cargo registry, git dependencies, installed tools,
  and target artifacts.
- Add caching to coverage, which currently installs and builds independently.
- Keep Clippy visible in CI but advisory until the known warning baseline is
  cleaned in a dedicated follow-up task.
- Avoid fixing Clippy warnings or Assura self-check violations.
- Avoid changing release packaging in this task.

## Acceptance Criteria

- [x] CI workflow uses a consistent Rust cache mechanism for build-heavy Rust
      jobs.
- [x] Coverage no longer runs with no Cargo cache.
- [x] Rustfmt remains simple and formatting-only.
- [x] Local YAML/config review confirms the workflow remains valid.
- [x] PR description calls out that Clippy failure remains known deferred debt.
- [x] Clippy remains visible but no longer makes unrelated PRs red while the
      baseline is deferred.

## Out of Scope

- Clippy warning cleanup.
- Combining jobs into a single serial job.
- Changing branch protection or required checks.
- Release workflow optimization.
- Assura structure self-check cleanup.

## Technical Notes

- Primary file: `.github/workflows/ci.yml`.
- Secondary file: `.github/workflows/docs.yml` if documentation build caching is
  made consistent.
- Existing `actions/cache` keys use only OS and `Cargo.lock`, so independent
  jobs can fight over broad `target/` cache contents and coverage has no cache
  at all.
- `Swatinem/rust-cache` is the standard lightweight replacement for manual
  Cargo cache blocks and handles target-directory caching more safely.
