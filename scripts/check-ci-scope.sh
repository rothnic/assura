#!/usr/bin/env bash
set -euo pipefail

check_scope() {
  local label="$1"
  local input="$2"
  local expected="$3"
  local actual

  actual="$(
    printf '%b' "$input" \
      | scripts/ci-scope.sh --files-from - \
      | grep -E '^(rust|release|performance|rustdoc|evidence|website|security)='
  )"

  if [ "$actual" != "$expected" ]; then
    printf 'CI scope policy mismatch: %s\n' "$label" >&2
    printf 'Expected:\n%s\n' "$expected" >&2
    printf 'Actual:\n%s\n' "$actual" >&2
    exit 1
  fi

  printf 'CI scope policy ok: %s\n' "$label"
}

check_scope_with_prefix() {
  local label="$1"
  local input="$2"
  local expected="$3"
  local actual

  actual="$(
    printf '%b' "$input" \
      | scripts/ci-scope.sh --prefix full_ --files-from - \
      | grep -E '^(full_rust|full_release|full_performance|full_rustdoc|full_evidence|full_website|full_security)='
  )"

  if [ "$actual" != "$expected" ]; then
    printf 'CI scope prefix policy mismatch: %s\n' "$label" >&2
    printf 'Expected:\n%s\n' "$expected" >&2
    printf 'Actual:\n%s\n' "$actual" >&2
    exit 1
  fi

  printf 'CI scope prefix policy ok: %s\n' "$label"
}

check_scope "docs and Trellis" \
  'AGENTS.md\n.trellis/workflow.md\n.assura/config.yml\n' \
  'rust=false
release=false
performance=false
rustdoc=false
evidence=true
website=false
security=false'

check_scope "Rust source" \
  'src/main.rs\n' \
  'rust=true
release=true
performance=true
rustdoc=true
evidence=false
website=true
security=false'

check_scope "Cargo metadata" \
  'Cargo.toml\nCargo.lock\n' \
  'rust=true
release=true
performance=true
rustdoc=true
evidence=false
website=true
security=true'

check_scope "installer" \
  'website/public/install.sh\nscripts/smoke-install-adoption.sh\n' \
  'rust=false
release=true
performance=false
rustdoc=false
evidence=true
website=false
security=false'

check_scope "performance evidence" \
  'website/public/data/performance/current.json\nbenches/history/current.json\n' \
  'rust=false
release=false
performance=true
rustdoc=false
evidence=true
website=true
security=false'

check_scope "website" \
  'website/src/pages/index.astro\nwebsite/package.json\n' \
  'rust=false
release=false
performance=false
rustdoc=false
evidence=false
website=true
security=false'

check_scope "workflow" \
  '.github/workflows/ci.yml\n' \
  'rust=true
release=true
performance=true
rustdoc=true
evidence=true
website=true
security=true'

check_scope "validation command" \
  '.cargo/config.toml\nxtask/src/main.rs\nscripts/ci-scope-github.sh\nscripts/summarize-rust-cache.sh\n' \
  'rust=true
release=true
performance=true
rustdoc=true
evidence=true
website=true
security=true'

check_scope_with_prefix "prefixed docs" \
  'docs/validation.md\n' \
  'full_rust=false
full_release=false
full_performance=false
full_rustdoc=false
full_evidence=true
full_website=false
full_security=false'
