#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci-scope.sh --base <sha> --head <sha> [--merge-base]
  scripts/ci-scope.sh --files-from <path|->
  scripts/ci-scope.sh --all [reason]
  scripts/ci-scope.sh --prefix <output-prefix> ...

Classifies changed paths into CI scopes and writes GitHub Actions outputs when
GITHUB_OUTPUT is set. Unknown diff state falls back to all scopes.
USAGE
}

base_sha=""
head_sha=""
merge_base=false
files_from=""
force_all_reason=""
output_prefix=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --base)
      base_sha="${2:-}"
      shift 2
      ;;
    --head)
      head_sha="${2:-}"
      shift 2
      ;;
    --merge-base)
      merge_base=true
      shift
      ;;
    --files-from)
      files_from="${2:-}"
      shift 2
      ;;
    --all)
      shift
      force_all_reason="${1:-forced all scopes}"
      if [ "$#" -gt 0 ] && [[ "$1" != --* ]]; then
        shift
      fi
      ;;
    --prefix)
      output_prefix="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage >&2
      exit 2
      ;;
  esac
done

rust=false
release=false
performance=false
rustdoc=false
evidence=false
website=false
security=false
reason=""
files=()

set_all() {
  rust=true
  release=true
  performance=true
  rustdoc=true
  evidence=true
  website=true
  security=true
  reason="${1:-all scopes}"
}

append_output() {
  local name="$1"
  local value="$2"
  local output_name="${output_prefix}${name}"
  printf '%s=%s\n' "$output_name" "$value"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf '%s=%s\n' "$output_name" "$value" >> "$GITHUB_OUTPUT"
  fi
}

is_all_zero_sha() {
  [[ "$1" =~ ^0+$ ]]
}

load_files_from_diff() {
  if [ -z "$base_sha" ] || [ -z "$head_sha" ] || is_all_zero_sha "$base_sha"; then
    set_all "diff base unavailable"
    return
  fi

  local range
  if [ "$merge_base" = true ]; then
    range="${base_sha}...${head_sha}"
  else
    range="${base_sha}..${head_sha}"
  fi

  if ! git rev-parse --verify --quiet "$base_sha^{commit}" >/dev/null; then
    set_all "base commit unavailable"
    return
  fi

  if ! git rev-parse --verify --quiet "$head_sha^{commit}" >/dev/null; then
    set_all "head commit unavailable"
    return
  fi

  mapfile -t files < <(git diff --name-only "$range")
}

load_files_from_input() {
  if [ "$files_from" = "-" ]; then
    mapfile -t files
  else
    mapfile -t files < "$files_from"
  fi
}

classify_path() {
  local path="$1"

  case "$path" in
    .github/workflows/*|.cargo/config.toml|xtask/*|xtask/**|scripts/ci-scope.sh|scripts/ci-scope-github.sh|scripts/summarize-rust-cache.sh)
      set_all "workflow, classifier, or validation command changed"
      return
      ;;
    .assura/*|.assura/**|scripts/check-ci-scope.sh)
      evidence=true
      return
      ;;
    benches/history/*|benches/history/**|website/public/data/performance/*|website/public/data/performance/**|website/src/content/docs/reference/performance*|docs/analysis/*performance*|docs/analysis/**performance*)
      performance=true
      evidence=true
      website=true
      return
      ;;
    Cargo.toml|Cargo.lock|rust-toolchain*|.cargo/*|build.rs|src/*|src/**|tests/*|tests/**|benches/*|benches/**|examples/*|examples/**)
      rust=true
      release=true
      performance=true
      rustdoc=true
      website=true
      case "$path" in
        Cargo.toml|Cargo.lock)
          security=true
          ;;
      esac
      return
      ;;
    website/public/install.sh|website/public/install.ps1|scripts/smoke-install-adoption.sh|scripts/smoke-install-adoption.ps1|docs/release-notes.md|docs/release-candidate-checklist.md|docs/support-policy.md|docs/compatibility-and-surface.md)
      release=true
      evidence=true
      return
      ;;
    website/package.json|website/pnpm-lock.yaml|website/astro.config.mjs|website/src/*|website/src/**|website/public/*|website/public/**|website/README.md|website/tsconfig.json)
      website=true
      return
      ;;
    .trellis/*|.trellis/**|.agents/*|.agents/**|AGENTS.md|docs/*|docs/**|.github/PULL_REQUEST_TEMPLATE.md)
      evidence=true
      return
      ;;
  esac
}

if [ -n "$force_all_reason" ]; then
  set_all "$force_all_reason"
elif [ -n "$files_from" ]; then
  load_files_from_input
else
  load_files_from_diff
fi

if [ -z "$reason" ]; then
  for file in "${files[@]}"; do
    classify_path "$file"
    [ -n "$reason" ] && break
  done
fi

changed_count="${#files[@]}"
if [ "$changed_count" -eq 0 ] && [ -z "$reason" ]; then
  reason="no changed files"
fi

append_output rust "$rust"
append_output release "$release"
append_output performance "$performance"
append_output rustdoc "$rustdoc"
append_output evidence "$evidence"
append_output website "$website"
append_output security "$security"
append_output changed_count "$changed_count"
append_output reason "$reason"
