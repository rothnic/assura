# Primary CLI Fast Check and Installable Release Path

## Objective

Make the optimized structure-check path available through the primary
`assura check` command, and update release/install documentation so users can
install and run `assura` without building Rust sources or choosing between a
fast and complete binary.

## User Problem

The performance work proved that the focused checker removes a large amount of
startup overhead, but exposing that as a separate user-facing command creates a
product split: one command is fast and another command is complete. New users
should be able to install one `assura` binary and run `assura check` directly.

## Requirements

- `assura check` uses the optimized checker path for supported one-shot
  validation invocations before full CLI startup work.
- The full CLI remains available for `init`, `status`, `migrate`, `watch`,
  `performance-report`, hooks, and fallback check options.
- Users do not need to install Rust for normal usage; install docs lead with
  prebuilt release binaries.
- The internal `assura-check` binary may remain for benchmark attribution, but
  user-facing docs and PR language should position `assura check` as the
  product path.
- The PR explains the before/after: full CLI overhead, focused checker
  correction, Linux static-CRT evidence, macOS dynamic-loader limitations, and
  the current usable install path.

## Validation

- `cargo fmt --all -- --check`
- `cargo test --all-targets --quiet`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run --quiet -- check --format json .`
- Website docs build if installation docs change.
