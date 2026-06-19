# Restore Windows CI Test Matrix

## Goal

Execute `docs/goals/assura-windows-ci-restore.md` by restoring the
`windows-latest` Rust test matrix with hosted proof that the full-feature test
path links and passes on Windows.

## Completion Evidence

- `docs/goals/assura-windows-ci-restore.md` defines the active proof gates.
- `.github/workflows/ci.yml` restores `windows-latest` to the Rust `Test Suite`
  matrix.
- `Cargo.lock` resolves `git2 0.21.0` and `libgit2-sys 0.18.5+1.9.4`.
- PR #93 head `0217c1cf18ada9ac5dcad54c3efaffa4c39f97d2` passed hosted
  Rust CI run `27839085592`, including `Test Suite (windows-latest, stable)`
  job `82393863614`, and merged as
  `272f3debc107c6ca29674130d9acbe67e23c7a40`.

## Acceptance Criteria

- [x] Dependency or configuration changes address the Windows
  `libgit2-sys`/MSVC link failure without removing `git-signals` or
  weakening `assura-full`.
- [x] `.github/workflows/ci.yml` restores `windows-latest` to the Rust
  `Test Suite` matrix.
- [x] `.trellis/spec/assura/tooling-stabilization.md` no longer lists Windows
  CI as paused baseline debt after hosted proof passes.
- [x] `docs/goals/assura-windows-ci-restore.md` progress log records the
  implementation, review, and hosted Windows evidence.
- [x] Local gates pass for changed surfaces.
- [x] Hosted PR checks include a passing Windows `Test Suite` job before merge.

## Review Scope

Ask the reviewer to block if Windows is still absent from the test matrix, if
the fix only hides the dependency from the full-feature path, if hosted Windows
proof is missing, if release Windows smoke jobs are weakened, or if tooling
stabilization still describes Windows CI as paused.
