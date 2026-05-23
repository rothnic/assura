#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh <fast|check|test|docs|release-smoke|pr|full>

Modes:
  fast           Format, whitespace, focused compile checks, normal tests, and Assura self-check.
  check          Focused compile checks for the primary launcher and full companion.
  test           Rust tests excluding benchmark harness targets.
  docs           Website static build.
  release-smoke  Build and smoke the local Linux installable release archive.
  pr             Fast gate plus clippy and docs build.
  full           PR gate plus cargo test --all-targets for benchmark-adjacent changes.
USAGE
}

mode="${1:-}"
if [ -z "$mode" ]; then
  usage >&2
  exit 2
fi

run_check() {
  cargo check -p assura --bin assura --no-default-features --features json-output,yaml-config
  cargo check -p assura --bin assura-full
}

run_test() {
  cargo test --workspace --lib --bins --tests --all-features --quiet
}

run_self_check() {
  cargo run --quiet -- check --format json .
}

run_docs() {
  if command -v pnpm >/dev/null 2>&1; then
    pnpm --dir website build
  elif command -v npm >/dev/null 2>&1; then
    npm --prefix website run build
  elif [ -x /usr/local/bin/npm ]; then
    PATH="/usr/local/bin:$PATH" /usr/local/bin/npm --prefix website run build
  else
    (cd website && node --run build)
  fi
}

run_release_smoke() {
  cargo build --release --bin assura --no-default-features --features json-output,yaml-config
  cargo rustc --release --bin assura --no-default-features --features json-output,yaml-config -- -C target-feature=+crt-static -C link-arg=-lgcc_eh
  cargo build --release --bin assura-full
  mkdir -p target/release-bundle
  cp target/release/assura target/release/assura-full target/release-bundle/
  tar -C target/release-bundle -czf target/assura-linux-amd64-preview.tar.gz assura assura-full

  tmp="$(mktemp -d)"
  cleanup() {
    rm -rf "$tmp"
  }
  trap cleanup EXIT INT TERM

  tar -xzf target/assura-linux-amd64-preview.tar.gz -C "$tmp"
  "$tmp/assura" check --quiet .
  "$tmp/assura" --version
  "$tmp/assura" --help >"$tmp/assura-help.txt"
  grep -q "Usage: assura" "$tmp/assura-help.txt"
}

case "$mode" in
  fast)
    cargo fmt --all -- --check
    git diff --check
    run_check
    run_test
    run_self_check
    ;;
  check)
    run_check
    ;;
  test)
    run_test
    ;;
  docs)
    run_docs
    ;;
  release-smoke)
    run_release_smoke
    ;;
  pr)
    "$0" fast
    cargo clippy --all-targets --all-features -- -D warnings
    run_docs
    ;;
  full)
    "$0" pr
    cargo test --all-targets --quiet
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
