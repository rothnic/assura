# PR CI Clippy Cleanup

## Objective

Resolve the Clippy failures reported by PR #112 without broadening the beta
release-candidate scope.

## Scope

- Fix only warnings from `cargo clippy --all-targets --all-features -- -D warnings`.
- Preserve existing behavior and public beta release metadata.
- Keep validation focused on Clippy, formatting, affected tests, Assura
  self-check, and PR status.

## Definition Of Done

- `cargo clippy --all-targets --all-features -- -D warnings` passes locally.
- Focused tests for touched surfaces pass.
- `cargo run --quiet -- check --format json .` passes.
- The cleanup is committed and pushed to PR #112.
