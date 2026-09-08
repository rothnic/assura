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

policy_review_output="$(
  printf '%b' 'tests/cli_contract.rs\nsrc/constraints/severity.rs\nsrc/cli/performance_report/rows.rs\n.github/workflows/ci.yml\nCONTRIBUTING.md\n' \
    | scripts/ci-scope.sh --files-from - \
    | grep -E '^(policy_review|policy_review_paths)='
)"
expected_policy_review='policy_review=true
policy_review_paths=tests/cli_contract.rs,src/constraints/severity.rs,src/cli/performance_report/rows.rs,.github/workflows/ci.yml'
if [ "$policy_review_output" != "$expected_policy_review" ]; then
  printf 'CI policy-review summary mismatch:\n%s\n' "$policy_review_output" >&2
  exit 1
fi
printf 'CI policy-review summary ok\n'

mixed_order_output="$(
  printf '%b' '.cargo/config.toml\ntests/policy.rs\n' \
    | scripts/ci-scope.sh --files-from - \
    | grep -E '^(rust|release|performance|rustdoc|evidence|website|security|reason|policy_review|policy_review_paths)='
)"
expected_mixed_order='rust=true
release=true
performance=true
rustdoc=true
evidence=true
website=true
security=true
reason=workflow, classifier, or validation command changed
policy_review=true
policy_review_paths=tests/policy.rs'
if [ "$mixed_order_output" != "$expected_mixed_order" ]; then
  printf 'CI mixed-order policy-review mismatch:\nExpected:\n%s\nActual:\n%s\n' "$expected_mixed_order" "$mixed_order_output" >&2
  exit 1
fi
printf 'CI mixed-order policy-review ok\n'

check_github_policy_review_retention() {
  local fixture_dir fixture_repo event_file fake_bin output
  fixture_dir="$(mktemp -d)"
  fixture_repo="$fixture_dir/repo"
  event_file="$fixture_dir/event.json"
  fake_bin="$fixture_dir/bin"
  mkdir -p "$fixture_repo/scripts" "$fixture_repo/tests" "$fixture_repo/docs" "$fake_bin"
  cp scripts/ci-scope.sh scripts/ci-scope-github.sh "$fixture_repo/scripts/"

  git -C "$fixture_repo" init -q
  git -C "$fixture_repo" config user.email ci-scope@example.invalid
  git -C "$fixture_repo" config user.name 'CI scope fixture'
  printf '%s\n' baseline > "$fixture_repo/README.md"
  git -C "$fixture_repo" add README.md
  git -C "$fixture_repo" commit -qm 'fixture: baseline'
  local base_sha
  base_sha="$(git -C "$fixture_repo" rev-parse HEAD)"

  printf '%s\n' 'policy-sensitive test' > "$fixture_repo/tests/policy.rs"
  git -C "$fixture_repo" add tests/policy.rs
  git -C "$fixture_repo" commit -qm 'test: add policy-sensitive test'
  local previous_sha
  previous_sha="$(git -C "$fixture_repo" rev-parse HEAD)"

  printf '%s\n' 'docs follow-up' > "$fixture_repo/docs/follow-up.md"
  git -C "$fixture_repo" add docs/follow-up.md
  git -C "$fixture_repo" commit -qm 'docs: follow up'
  local head_sha
  head_sha="$(git -C "$fixture_repo" rev-parse HEAD)"

  cat > "$event_file" <<EOF
{"action":"synchronize","before":"$previous_sha","pull_request":{"base":{"sha":"$base_sha"},"head":{"sha":"$head_sha"}}}
EOF
  cat > "$fake_bin/gh" <<'EOF'
#!/usr/bin/env bash
cat <<'CHECKS'
Check	success
CHECKS
if [ "${OMIT_MSRV_CHECK:-}" != true ]; then
  printf '%s\n' 'MSRV (Rust 1.86.0)	success'
fi
cat <<'CHECKS'
Rustfmt	success
Clippy	success
Code Coverage	success
Test Suite (ubuntu-latest, stable)	success
Test Suite (macos-latest, stable)	success
Test Suite (windows-latest, stable)	success
Release Bundle Smoke	success
Windows Installer Smoke	success
Installable Adoption Smoke (ubuntu-x86_64)	success
CHECKS
if [ "${OMIT_ALPINE_CHECK:-}" != true ]; then
  printf '%s\n' 'Installable Adoption Smoke (alpine-x86_64)	success'
fi
cat <<'CHECKS'
Installable Adoption Smoke (macos-arm64)	success
Installable Adoption Smoke (macos-x86_64)	success
Installable Adoption Smoke (windows-x86_64)	success
Performance Report	success
Build Documentation	success
Security Audit	success
CHECKS
EOF
  chmod +x "$fake_bin/gh"

  output="$(
    cd "$fixture_repo"
    PATH="$fake_bin:$PATH" \
      GITHUB_EVENT_NAME=pull_request \
      GITHUB_EVENT_PATH="$event_file" \
      GITHUB_REPOSITORY=rothnic/assura \
      GH_TOKEN=fixture-token \
      scripts/ci-scope-github.sh
  )"

  local actual
  actual="$(
    for output_name in scope_mode policy_review policy_review_paths; do
      printf '%s\n' "$output" | grep -E "^${output_name}=" | tail -n 1
    done
  )"
  local expected='scope_mode=delta
policy_review=true
policy_review_paths=tests/policy.rs'
  if [ "$actual" != "$expected" ]; then
    printf 'GitHub full-PR policy-review retention mismatch:\nExpected:\n%s\nActual:\n%s\n' "$expected" "$actual" >&2
    exit 1
  fi
  printf 'GitHub full-PR policy-review retention ok\n'

  local missing_check_output missing_check_mode
  missing_check_output="$(
    cd "$fixture_repo"
    PATH="$fake_bin:$PATH" \
      GITHUB_EVENT_NAME=pull_request \
      GITHUB_EVENT_PATH="$event_file" \
      GITHUB_REPOSITORY=rothnic/assura \
      GH_TOKEN=fixture-token \
      OMIT_ALPINE_CHECK=true \
      scripts/ci-scope-github.sh
  )"
  missing_check_mode="$(printf '%s\n' "$missing_check_output" | grep -E '^scope_mode=' | tail -n 1)"
  if [ "$missing_check_mode" != 'scope_mode=full' ]; then
    printf 'GitHub missing adoption-check fallback mismatch: expected scope_mode=full, got %s\n' "$missing_check_mode" >&2
    exit 1
  fi
  printf 'GitHub missing adoption-check fallback ok\n'

  local missing_msrv_output missing_msrv_mode
  missing_msrv_output="$(
    cd "$fixture_repo"
    PATH="$fake_bin:$PATH" \
      GITHUB_EVENT_NAME=pull_request \
      GITHUB_EVENT_PATH="$event_file" \
      GITHUB_REPOSITORY=rothnic/assura \
      GH_TOKEN=fixture-token \
      OMIT_MSRV_CHECK=true \
      scripts/ci-scope-github.sh
  )"
  missing_msrv_mode="$(printf '%s\n' "$missing_msrv_output" | grep -E '^scope_mode=' | tail -n 1)"
  if [ "$missing_msrv_mode" != 'scope_mode=full' ]; then
    printf 'GitHub missing MSRV-check fallback mismatch: expected scope_mode=full, got %s\n' "$missing_msrv_mode" >&2
    exit 1
  fi
  printf 'GitHub missing MSRV-check fallback ok\n'
  rm -rf "$fixture_dir"
}

check_github_policy_review_retention
