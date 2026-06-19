---
id: goal-assura-windows-ci-restore
type: goal
title: Assura Windows CI restore
status: planned
created: 2026-06-19
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/tooling-stabilization.md
  - .github/workflows/ci.yml
  - Cargo.toml
  - Cargo.lock
  - src/maturity/git.rs
---

# Assura Windows CI Restore

## Objective

Restore `windows-latest` to the Rust test matrix with hosted proof that the
full-feature test path links and passes on Windows.

## Current Gap

The roadmap still carries one deferred tooling baseline item: Windows CI
Restore. The Rust test matrix currently runs on Linux and macOS only because a
previous `windows-latest` run failed while linking `libgit2-sys` with unresolved
MSVC system-library symbols. The release and installer workflows still define
Windows smoke paths, but they build the lean release feature set and do not
prove `cargo test --all-features` on Windows.

## Scope

- Reproduce or refresh the hosted Windows failure mode using a PR check or a
  temporary diagnostic workflow path.
- Fix the Windows full-feature build/test linkage without weakening the
  supported `assura` and `assura-full` feature contracts.
- Restore `windows-latest` to the Rust `Test Suite` matrix in
  `.github/workflows/ci.yml`.
- Update `.trellis/spec/assura/tooling-stabilization.md` after the fix lands so
  Windows CI is no longer listed as paused baseline debt.
- Keep release Windows smoke checks intact.

## Non-Goals

- No broad release packaging redesign.
- No removal of `git-signals` or `assura-full` just to avoid the linker issue.
- No hosted coverage, Codecov, or unrelated CI policy changes.
- No claim that every Windows release artifact is proven by the test matrix
  alone; release smoke jobs remain the artifact proof.

## Definition Of Done

- `windows-latest` is restored to the Rust `Test Suite` matrix.
- Hosted CI shows `Test Suite (windows-latest, stable)` passing on the PR that
  restores the matrix.
- Linux and macOS test matrix entries continue to pass.
- `cargo check --all-targets --all-features`, `cargo clippy --all-targets
  --all-features -- -D warnings`, and `cargo test --all-features` remain green
  locally or in hosted CI where platform-specific proof is required.
- `.trellis/spec/assura/tooling-stabilization.md` records the closed Windows
  baseline and any remaining deferred tooling issues accurately.
- The PR links this goal and includes independent review.

## Validation Commands

Local validation before PR:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask target-state
git diff --check
```

Hosted validation before merge:

```bash
gh pr checks <pr-number> --watch
```

The hosted proof must include a passing Windows `Test Suite` job after the
matrix is restored.

## Reviewer Blocking Criteria

Block the PR if Windows is still absent from the Rust test matrix, if the fix
only hides `git2`/`libgit2-sys` from `assura-full` without a product decision,
if hosted Windows tests are missing or skipped, if release Windows smoke paths
are weakened, or if the tooling-stabilization spec still describes Windows CI
as paused after the restore.

## Progress Log

- 2026-06-19: Revalidated live roadmap and CI state after Support Matrix
  Surface Expansion completion. All existing Assura goal docs are completed,
  current target-state passes, and Windows CI Restore is the only non-completed
  roadmap item. Created this planned goal as the next bounded candidate.
