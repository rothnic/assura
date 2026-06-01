# Rustfmt Baseline Cleanup

## Objective

Make repository-wide Rust formatting pass so the `Rustfmt` CI check can become
a trustworthy blocking signal.

## Background

PR #1 intentionally did not sweep repository-wide formatting changes into the
self-enforcement and Trellis governance work. CI now makes the remaining state
clear: build, docs, Linux tests, macOS tests, and coverage are expected to pass,
while `Rustfmt` and `Clippy` expose known baseline debt.

The first tooling cleanup iteration should be mechanical and narrow: run
rustfmt, review the resulting diff, and avoid unrelated behavior or
documentation changes.

## Scope

- Run repository-wide `cargo fmt --all`.
- Review the diff for formatting-only churn.
- Verify `cargo fmt --all -- --check` passes.
- Verify tests still pass after formatting.
- Update the tooling stabilization spec only if the gate policy changes.
- Prepare a focused PR after the bootstrap PR lands.

## Out Of Scope

- Do not fix Clippy warnings in this task.
- Do not change runtime behavior.
- Do not refactor modules except where rustfmt itself changes formatting.
- Do not clean Assura self-check violations unrelated to formatting.
- Do not archive or rewrite stale documentation in this task.

## Acceptance Criteria

- `cargo fmt --all -- --check` passes locally.
- `cargo test --all-targets --quiet` passes locally.
- CI `Rustfmt` passes on the rustfmt cleanup PR.
- The PR diff is formatting-only, except for a small Trellis/spec update if
  required to mark the rustfmt baseline as clean.
- The task records any unexpected non-formatting issue before expanding scope.

## Recommended Execution

1. Create a new branch from the PR #1 merge target after PR #1 lands.
2. Run `cargo fmt --all`.
3. Review `git diff --stat` and spot-check representative files.
4. Run `cargo fmt --all -- --check`.
5. Run `cargo test --all-targets --quiet`.
6. Commit with `style: apply repository rustfmt baseline`.
7. Open a focused PR and make no additional cleanup changes.

## Status

- [ ] Wait for bootstrap PR #1 to be ready or merged.
- [ ] Create the rustfmt cleanup branch.
- [ ] Run repository-wide rustfmt.
- [ ] Verify formatting and tests.
- [ ] Open focused rustfmt cleanup PR.
