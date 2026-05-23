#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh <fast|check|test|docs|release-size|release-smoke|pr|full>

Modes:
  fast           Format, whitespace, focused compile checks, normal tests, and Assura self-check.
  check          Focused compile checks for the primary launcher and full companion.
  test           Rust tests excluding benchmark harness targets.
  docs           Website static build.
  release-size   Build the local installable release archive and enforce its size budget.
  release-smoke  Build and smoke the local installable release archive.
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
  cargo test --workspace --lib --tests --all-features --quiet
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

release_platform() {
  case "$(uname -s)" in
    Linux) echo linux ;;
    Darwin) echo macos ;;
    MINGW*|MSYS*|CYGWIN*) echo windows ;;
    *) echo unknown ;;
  esac
}

release_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo amd64 ;;
    arm64|aarch64) echo arm64 ;;
    *) uname -m ;;
  esac
}

release_archive_path() {
  local platform arch
  platform="$(release_platform)"
  arch="$(release_arch)"
  echo "target/assura-${platform}-${arch}-preview.tar.gz"
}

archive_size_bytes() {
  wc -c <"$1" | tr -d '[:space:]'
}

run_release_bundle() {
  local archive platform
  platform="$(release_platform)"
  if [ "$platform" = "windows" ]; then
    printf 'release-size and release-smoke use the Unix tarball path; Windows archive smoke is covered by CI and website/public/install.ps1.\n' >&2
    return 2
  fi

  archive="$(release_archive_path)"

  if [ "$platform" = "linux" ]; then
    cargo rustc --release --bin assura --no-default-features --features json-output,yaml-config -- -C target-feature=+crt-static -C link-arg=-lgcc_eh
  else
    cargo build --release --bin assura --no-default-features --features json-output,yaml-config
  fi
  cargo build --release --bin assura-full
  mkdir -p target/release-bundle
  cp target/release/assura target/release/assura-full target/release-bundle/
  tar -C target/release-bundle -czf "$archive" assura assura-full
  RELEASE_ARCHIVE="$archive"
}

run_release_size() {
  local archive size max_size
  run_release_bundle
  archive="$RELEASE_ARCHIVE"
  size="$(archive_size_bytes "$archive")"
  max_size="${ASSURA_MAX_RELEASE_ARCHIVE_BYTES:-8388608}"

  printf 'Release archive: %s\n' "$archive"
  printf 'Release archive size: %s bytes (max %s)\n' "$size" "$max_size"

  if [ "$size" -gt "$max_size" ]; then
    printf 'Release archive exceeds size budget.\n' >&2
    return 1
  fi
}

run_release_smoke() {
  local archive
  run_release_bundle
  archive="$RELEASE_ARCHIVE"

  tmp="$(mktemp -d)"
  cleanup() {
    rm -rf "$tmp"
  }
  trap cleanup EXIT INT TERM

  tar -xzf "$archive" -C "$tmp"
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
  release-size)
    run_release_size
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
