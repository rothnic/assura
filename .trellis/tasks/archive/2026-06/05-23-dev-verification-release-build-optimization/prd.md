# Optimize development verification and release build size

## Goal

Make Assura's day-to-day verification and release build process substantially
faster and safer by default. The normal local loop should avoid unnecessary
Cargo targets, and the release path should prove that published artifacts are
small installable bundles rather than accidentally exposing build-cache bloat or
extra developer binaries.

## What I Already Know

- The current branch is `codex/ls-lint-realistic-parity-core-performance`.
- `scripts/verify.sh` already introduced validation tiers, but
  `verify:test` still uses `cargo test --workspace --lib --bins --tests`.
- A measured `node --run verify:test` run took `91.44s` wall time.
- A measured `cargo test --workspace --lib --tests --all-features --quiet` run
  took `56.91s` wall time with the same visible Rust test suites passing.
- The repository has 34 Rust test files/modules and 5 Criterion benchmark
  files.
- The local `target/` directory is large (`target/debug` about 15 GB and
  `target/release` about 1.5 GB), so local build-cache size is not release
  artifact size.
- Current macOS release binaries are small individually:
  `target/release/assura` about 1.1 MB and `target/release/assura-full` about
  2.7 MB.
- The release workflow currently bundles only `assura` and `assura-full` for
  public installs, while the `assura-check-cli` crate has additional benchmark
  and diagnostic binaries used during development/performance work.

## Requirements

- Reduce the default local Rust test loop by avoiding unnecessary binary test
  harness targets while preserving integration tests that use
  `CARGO_BIN_EXE_*`.
- Keep `verify:full` available for benchmark-adjacent or final confidence runs.
- Add a release-size guard that builds the public install bundle and fails if
  the produced bundle is unexpectedly large.
- Ensure the size guard checks the installable archive, not the Cargo
  `target/` cache.
- Document the distinction between local build-cache size, release binary size,
  and public archive size.
- Keep release packaging centered on the primary `assura` command users install
  and run.

## Acceptance Criteria

- [x] `node --run verify:test` no longer invokes `cargo test --bins` by default.
- [x] A release-size check command exists in the validation tier scripts.
- [x] The release-size check is wired into a documented command and CI-friendly
      flow.
- [x] Documentation explains why `target/` can be huge without implying a huge
      published artifact.
- [x] Validation passes for the changed scripts and docs.

## Out of Scope

- Publishing a live GitHub Release tag.
- Removing the hidden `assura-full` fallback companion unless measurement shows
  it is the actual source of unacceptable public artifact size.
- Replacing Cargo's build cache or introducing a remote build farm in this task.

## Technical Notes

- Primary files inspected: `scripts/verify.sh`, `docs/validation.md`,
  `.github/workflows/ci.yml`, root `Cargo.toml`, and
  `crates/assura-check-cli/Cargo.toml`.
- Existing release profile already uses `lto = true`, `codegen-units = 1`,
  `strip = true`, and `panic = "abort"`.
- The highest-confidence quick win is removing `--bins` from the normal test
  tier because integration tests still request the required binaries through
  Cargo's `CARGO_BIN_EXE_*` mechanism.
- CI revealed a Windows-only product polish issue: the primary `assura.exe`
  wrapper could not set `argv[0]` for the `assura-full.exe` companion, so
  `assura.exe --help` rendered `Usage: assura-full.exe`. The fix uses an
  explicit `ASSURA_CLI_BIN_NAME=assura` companion environment contract and a
  regression test.
