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
- 2026-06-19: Implemented the local restore candidate by updating `git2` from
  0.18.3 to 0.21.0, refreshing `libgit2-sys` from 0.16.2+1.7.2 to
  0.18.5+1.9.4, preserving explicit SSH/HTTPS features, adapting the changed
  `Signature::email` API, and restoring `windows-latest` to the Rust test
  matrix. Local `cargo fmt --all -- --check`,
  `cargo check --all-targets --all-features`,
  `cargo clippy --all-targets --all-features -- -D warnings`, and
  `cargo test --all-features` passed. Hosted Windows proof still required
  before completion.
- 2026-06-19: PR #93 hosted Windows `Test Suite (windows-latest, stable)` ran
  far enough to prove the `libgit2-sys`/MSVC linker failure is cleared, then
  failed on two Unix-specific performance-report test assertions: a native
  LS-Lint binary path string check that assumed `/` separators and an external
  fixture content check that assumed LF checkout newlines. Updated the tests to
  assert path components and normalize readback newlines while preserving the
  same product coverage. Local `cargo fmt --all -- --check`,
  `cargo test --all-features -p assura --lib`,
  `cargo check --all-targets --all-features`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo run --quiet -- check --format json .`, `cargo xtask evidence`,
  `cargo xtask target-state`, `git diff --check`, and
  `cargo test --all-features` passed. Hosted Windows proof is still required
  before completion.
- 2026-06-19: The next PR #93 hosted Windows run passed the previously failing
  performance-report assertions and failed later in `tests/cli_check_tests.rs`
  because the fail-fast JSON path assertion assumed `/` separators while
  serialized `PathBuf` values use `\` on Windows. Normalized that test
  assertion to compare the logical path without changing the check output
  contract. Hosted Windows proof is still required before completion.
- 2026-06-19: The following PR #93 hosted Windows run passed the prior
  fail-fast assertion and failed later in `tests/docs_lifecycle_tests.rs`
  because the docs lifecycle JSON path assertion also assumed `/` separators.
  Normalized that assertion and the other nested-path JSON/advice assertions in
  downstream integration tests to compare logical paths while preserving native
  serialized `PathBuf` output. Hosted Windows proof is still required before
  completion.
- 2026-06-19: The next PR #93 hosted Windows run passed the downstream
  nested-path assertions and failed later in
  `tests/ls_lint_parity_regression_tests.rs` because an external fixture
  materialization readback asserted LF newlines while Windows Git checkout
  produced CRLF. Normalized that test readback while keeping the pinned
  revision and cache-materialization coverage. Hosted Windows proof is still
  required before completion.
- 2026-06-19: The following PR #93 hosted Windows run passed the external
  fixture readback assertion and failed later in the native LS-Lint golden tests
  because the restored Test Suite matrix did not install Node/npm before tests
  that intentionally install the pinned LS-Lint package. Added `setup-node`
  with Node 24 to the Rust Test Suite job to make the existing test dependency
  explicit across Linux, macOS, and Windows. Hosted Windows proof is still
  required before completion.
