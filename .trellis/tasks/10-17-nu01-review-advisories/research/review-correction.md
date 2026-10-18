# NU01 independent-review correction

- Reviewed candidate: `8be12d1155dc9c7bbefc2854cf500cdf6c2eac13`; correction commit `13f71b195a02648edd5a9f5f15c127358bad0577`, rebased candidate `0f4c4167b56e137f4b8b0cc73bf359610e032d9a`.
- Finding: `/Users/nroth/workspace/.assura-review-20261017/nu01-review-advisories/nu01-independent-review.md` reported P2: unchanged configured advisories disappeared from finding rows but still generated compact text/agent `next_actions`.
- Correction: next-action assembly now selects only configured Review findings in `new` or `worsened` state after history has been applied. Unchanged and resolved configured findings stay in full JSON/history but do not inject configured fix actions. Independent Doctor actions remain; the blocking Doctor action stays first on Check failure.
- Controls: repeated text and agent output assert that the configured path action is absent; an initial agent report with new configured advisories asserts the action is present; the existing repeated-blocker flow now asserts unchanged blockers remain visible in text and agent next-actions.
- Spec and task acceptance now state the history-sensitive next-action behavior.

## Validation

- Checkout `/Users/nroth/.codex/worktrees/assura-nu01-review-advisories/assura`, branch `fix/nu01-review-advisories`.
- Current `refs/remotes/origin/master` and the rebased branch merge base: `ba4804bb6abdadc1ccc12c41397dd2402a914171`. `git diff --quiet 49d85358bae85de04f1ce7faa746b377687dcaf3..refs/remotes/origin/master -- src crates` succeeded, confirming no Rust source changes between the original Rust base and current integration base. The base adds delivery-contract Python coverage and project docs/config only.
- Formatter/toolchain: `cargo fmt --version` and `rustfmt --version` report `rustfmt 1.8.0-stable (e408947bfd 2026-03-25)`; Cargo `1.94.1`; Rust `1.94.1`; `Cargo.toml` edition `2021`. `cargo fmt --all` and `cargo fmt --all -- --check` passed. All formatter hunks were inspected; changes include the previously flagged import/assert wrapping and new test code only.
- Resource gate before focused compile: 72% system memory free, 127 GiB disk available; no other Cargo/rustc job. Two-job local run used `/Users/nroth/workspace/.assura-review-20261017/nu01-tmp`.
- On rebased candidate `0f4c4167b56e137f4b8b0cc73bf359610e032d9a`, `env TMPDIR=/Users/nroth/workspace/.assura-review-20261017/nu01-tmp CARGO_BUILD_JOBS=2 cargo test --test project_review_cli` — exit 0; 16 passed, 0 failed, 0 ignored.
- On that candidate, `target/debug/assura check --format json .` — exit 0, success true, 15 existing low-severity findings elsewhere; none in the changed files.
- `git diff --check` — exit 0; `src/cli/project_review/report.rs` 489 lines and `tests/project_review_cli.rs` 993 lines.

No VPS, PR, push, or Beads operation was performed. The rebased correction is committed; independent rereview of the corrected commit and coordinator PR-tier validation remain.
