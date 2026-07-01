# PR CI Clippy Cleanup

## Objective

Resolve the CI failures reported by PR #112 without broadening the beta
release-candidate scope.

## Scope

- Fix warnings and platform regressions surfaced by PR #112 CI.
- Preserve the hard per-fixture LS-Lint no-slower gate; do not add a tolerance
  or weaken fixture coverage.
- Preserve existing behavior and public beta release metadata.
- Keep validation focused on affected tests, formatting, Clippy, Assura
  self-check, performance no-slower evidence, and PR status.

## Definition Of Done

- `cargo clippy --all-targets --all-features -- -D warnings` passes locally.
- Focused tests for touched surfaces pass.
- `cargo run --quiet -- check --format json .` passes.
- `cargo xtask performance-no-slower <current-report.json>` passes.
- The cleanup is committed and pushed to PR #112.
