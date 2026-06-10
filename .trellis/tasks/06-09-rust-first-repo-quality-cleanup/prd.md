# Rust-first repo quality cleanup

## Goal

Move Assura's root repository quality workflow away from Node-based command
aliases and abandoned root LS-Lint package dependencies, while adding the first
Rust-native checks that prevent the same drift from returning.

## What I already know

- The root `package.json` only exposes validation aliases and depends on
  `@ls-lint/ls-lint`.
- LS-Lint is a Go utility; the npm package was used as an older distribution
  path and is not a reason to keep root Node validation.
- Website docs use Astro/pnpm and should remain Node-scoped under `website/`.
- Existing CI/docs/reference text still points at `node --run verify:*`.
- Current Clippy gate is clean with `-D warnings`, but stricter Clippy exposed
  high-signal cleanup areas: `too_many_arguments`, unused async, repeated hash
  helpers, unexplained dead-code allowances, and low-level unsafe boundaries.

## Requirements

- Add a Rust-first root quality command surface with `cargo xtask`.
- Preserve existing validation behavior during migration.
- Remove root Node/LS-Lint validation dependencies when no root command needs
  them.
- Keep website and Node agent integrations out of root Node-removal policy.
- Add deterministic checks for root Node dependency drift and unexplained Rust
  lint suppressions.
- Consolidate duplicated stable hash helpers where practical in this slice.
- Keep all changes compatible with the existing Rust MSRV policy.

## Acceptance Criteria

- [ ] `cargo xtask target-state` passes.
- [ ] `cargo xtask pr` passes or any skipped long-running portion is explicitly
      justified in the PR.
- [ ] Root `package.json`/root npm dependency usage is removed or blocked by a
      documented allowlist.
- [ ] CI/docs/PR template prefer `cargo xtask` for root validation.
- [ ] New lint suppressions without reason comments fail a deterministic check.
- [ ] Existing `cargo clippy --all-targets --all-features -- -D warnings`
      remains clean.
- [ ] Workspace ends clean, task archived or advanced, and PR is merged.

## Out of Scope

- Do not change LS-Lint benchmark claims or regenerate performance evidence in
  this PR unless required by tests.
- Do not remove Node from `website/` or Node agent integration packages.
- Do not enable all `clippy::pedantic` or `clippy::nursery`.
- Do not raise MSRV from Rust `1.70.0`.

## Technical Notes

- Relevant specs: `.trellis/spec/assura/index.md`,
  `.trellis/spec/assura/tooling-stabilization.md`, and
  `.trellis/spec/guides/code-reuse-thinking-guide.md`.
- Existing validation shell entrypoint: `scripts/verify.sh`.
- Existing target-state verifier: `scripts/verify-target-state.py`.
