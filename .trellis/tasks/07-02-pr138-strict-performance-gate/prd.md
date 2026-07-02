# PR 138 Strict Performance Gate

## Goal

Fix the PR #138 Performance Report failure without relaxing the strict
accepted-fixture rule that Assura must not be slower than LS-Lint on any
accepted fixture.

## Context

- PR #138 records live `v0.3.0` release evidence for the post-beta parent
  program.
- CI run `28601993430` failed only the Performance Report job.
- The failing accepted fixture is `many_configured_scopes_regression`, where
  `assura-cli` measured `17.511 ms` versus `ls-lint-cli` at `17.412 ms`.
- The same report skipped check-only diagnostic rows because the performance
  job did not build `assura-check-cli` binaries.

## Requirements

- Preserve the strict `cargo xtask performance-no-slower` behavior.
- Reduce real one-shot `assura-cli` cost for many configured LS-Lint-compatible
  scopes instead of hiding the public CLI row behind a diagnostic mode.
- Build the check-only performance binaries in CI so diagnostic rows are
  present and useful.
- Keep the change narrow enough for the release-evidence PR.

## Acceptance Criteria

- [x] `cargo xtask performance-no-slower` passes on a regenerated comparison
      report.
- [ ] CI Performance Report passes for PR #138.
- [x] Accepted LS-Lint-equivalent fixtures remain strictly no-slower.
- [x] Performance report diagnostic rows are not skipped due to missing
      check-only binaries.
- [x] Local validation remains clean.

## Validation Plan

- `cargo fmt --all -- --check`
- `cargo test -p assura ls_fast_plan --quiet`
- `cargo build --release --bin assura --no-default-features --features json-output,yaml-config`
- `cargo build --release --bin assura-full`
- `cargo build --release -p assura-check-cli`
- `target/release/assura performance-report --output target/performance/pr138-fix.json --iterations 5`
- `cargo xtask performance-no-slower target/performance/pr138-fix.json`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`

## Validation Evidence

- `cargo fmt --all -- --check`
- `cargo test -p assura --lib ls_fast_plan --quiet`
- `cargo clippy -p assura --lib --all-features -- -D warnings`
- `cargo build --release --bin assura --no-default-features --features json-output,yaml-config`
- `cargo build --release --bin assura-full`
- `cargo build --release -p assura-check-cli`
- `target/release/assura performance-report --output target/performance/pr138-fix.json --iterations 5`
- `cargo xtask performance-no-slower target/performance/pr138-fix.json`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask evidence`

The regenerated report passed the strict no-slower gate. For
`many_configured_scopes_regression`, local medians were `assura-cli`
`47.997 ms` and `ls-lint-cli` `49.064 ms`; no rows were skipped for missing
check-only binaries.

## Out Of Scope

- Loosening the comparison gate, adding timing tolerance, or changing accepted
  fixture status.
- Reframing compiled, daemon, cached, or check-only rows as the public
  `assura-cli` completion evidence.
