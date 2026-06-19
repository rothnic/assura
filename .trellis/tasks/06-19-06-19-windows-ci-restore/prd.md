# Restore Windows CI Test Matrix

## Goal

Execute `docs/goals/assura-windows-ci-restore.md` by restoring the
`windows-latest` Rust test matrix with hosted proof that the full-feature test
path links and passes on Windows.

## Current Evidence

- `docs/goals/assura-windows-ci-restore.md` defines the active proof gates.
- `.github/workflows/ci.yml` currently limits the Rust `Test Suite` matrix to
  Linux and macOS because of a deferred `libgit2-sys` MSVC linker failure.
- `Cargo.lock` currently resolves `git2 0.18.3` and
  `libgit2-sys 0.16.2+1.7.2`.
- `cargo info` reports current crates.io availability of `git2 0.21.0` and
  newer `libgit2-sys` releases.

## Acceptance Criteria

- [x] Dependency or configuration changes address the Windows
  `libgit2-sys`/MSVC link failure without removing `git-signals` or
  weakening `assura-full`.
- [x] `.github/workflows/ci.yml` restores `windows-latest` to the Rust
  `Test Suite` matrix.
- [x] `.trellis/spec/assura/tooling-stabilization.md` no longer lists Windows
  CI as paused baseline debt after hosted proof passes.
- [ ] `docs/goals/assura-windows-ci-restore.md` progress log records the
  implementation, review, and hosted Windows evidence.
- [x] Local gates pass for changed surfaces.
- [ ] Hosted PR checks include a passing Windows `Test Suite` job before merge.

## Review Scope

Ask the reviewer to block if Windows is still absent from the test matrix, if
the fix only hides the dependency from the full-feature path, if hosted Windows
proof is missing, if release Windows smoke jobs are weakened, or if tooling
stabilization still describes Windows CI as paused.
