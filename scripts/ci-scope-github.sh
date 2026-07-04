#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/ci-scope-github.sh

Classifies GitHub Actions changed paths into:
  - full_* outputs for the full PR or push range;
  - delta_* outputs for a pull_request synchronize commit delta when available;
  - unprefixed effective outputs used by workflow jobs.

For pull_request synchronize events, effective outputs use the previous PR head
to current PR head delta only when any skipped full-scope jobs already passed
on the previous PR head. All other events use the full PR or push scope.
USAGE
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

append_output() {
  local name="$1"
  local value="$2"
  printf '%s=%s\n' "$name" "$value"
  if [ -n "${GITHUB_OUTPUT:-}" ]; then
    printf '%s=%s\n' "$name" "$value" >> "$GITHUB_OUTPUT"
  fi
}

is_all_zero_sha() {
  [[ "$1" =~ ^0+$ ]]
}

scope_file_value() {
  local file="$1"
  local name="$2"
  grep -E "^${name}=" "$file" | tail -n 1 | cut -d= -f2-
}

run_scope_to_file() {
  local output_file="$1"
  shift
  scripts/ci-scope.sh "$@" | tee "$output_file"
}

event_value() {
  local key="$1"
  python3 - "$key" <<'PY'
import json
import os
import sys

key = sys.argv[1]
path = os.environ.get("GITHUB_EVENT_PATH", "")
data = {}
if path:
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except FileNotFoundError:
        data = {}

pull_request = data.get("pull_request") or {}
values = {
    "action": data.get("action", ""),
    "before": data.get("before", ""),
    "after": data.get("after", ""),
    "pull_request_base_sha": ((pull_request.get("base") or {}).get("sha") or ""),
    "pull_request_head_sha": ((pull_request.get("head") or {}).get("sha") or ""),
}
print(values.get(key, ""))
PY
}

run_full_scope() {
  local output_file="$1"
  local event_name="${GITHUB_EVENT_NAME:-}"
  case "$event_name" in
    pull_request)
      local base_sha head_sha
      base_sha="$(event_value pull_request_base_sha)"
      head_sha="$(event_value pull_request_head_sha)"
      run_scope_to_file "$output_file" --prefix full_ --base "$base_sha" --head "$head_sha" --merge-base
      ;;
    push)
      local before_sha head_sha
      before_sha="$(event_value before)"
      head_sha="${GITHUB_SHA:-$(event_value after)}"
      run_scope_to_file "$output_file" --prefix full_ --base "$before_sha" --head "$head_sha"
      ;;
    schedule)
      run_scope_to_file "$output_file" --prefix full_ --all "scheduled audit"
      ;;
    *)
      run_scope_to_file "$output_file" --prefix full_ --all "unsupported event"
      ;;
  esac
}

run_unprefixed_full_scope() {
  local event_name="${GITHUB_EVENT_NAME:-}"
  case "$event_name" in
    pull_request)
      local base_sha head_sha
      head_sha="$(event_value pull_request_head_sha)"
      base_sha="$(event_value pull_request_base_sha)"
      scripts/ci-scope.sh --base "$base_sha" --head "$head_sha" --merge-base
      ;;
    push)
      local before_sha head_sha
      before_sha="$(event_value before)"
      head_sha="${GITHUB_SHA:-$(event_value after)}"
      scripts/ci-scope.sh --base "$before_sha" --head "$head_sha"
      ;;
    schedule)
      scripts/ci-scope.sh --all "scheduled audit"
      ;;
    *)
      scripts/ci-scope.sh --all "unsupported event"
      ;;
  esac
}

run_delta_scope() {
  local output_file="$1"
  local before_sha="$2"
  local head_sha="$3"
  run_scope_to_file "$output_file" --prefix delta_ --base "$before_sha" --head "$head_sha"
}

run_unprefixed_delta_scope() {
  local before_sha="$1"
  local head_sha="$2"
  scripts/ci-scope.sh --base "$before_sha" --head "$head_sha"
}

required_checks_for_group() {
  local group="$1"
  case "$group" in
    rust)
      printf '%s\n' \
        "Check" \
        "Rustfmt" \
        "Clippy" \
        "Code Coverage" \
        "Test Suite (ubuntu-latest, stable)" \
        "Test Suite (macos-latest, stable)" \
        "Test Suite (windows-latest, stable)"
      ;;
    release)
      printf '%s\n' \
        "Release Bundle Smoke" \
        "Windows Installer Smoke" \
        "Installable Adoption Smoke (ubuntu-x86_64)" \
        "Installable Adoption Smoke (macos-arm64)" \
        "Installable Adoption Smoke (macos-x86_64)" \
        "Installable Adoption Smoke (windows-x86_64)"
      ;;
    performance)
      printf '%s\n' "Performance Report"
      ;;
    rustdoc)
      printf '%s\n' "Build Documentation"
      ;;
    security)
      printf '%s\n' "Security Audit"
      ;;
  esac
}

skipped_full_scope_groups() {
  local full_file="$1"
  local delta_file="$2"
  for group in rust release performance rustdoc security; do
    local full_value delta_value
    full_value="$(scope_file_value "$full_file" "full_${group}")"
    delta_value="$(scope_file_value "$delta_file" "delta_${group}")"
    if [ "$full_value" = "true" ] && [ "$delta_value" != "true" ]; then
      printf '%s\n' "$group"
    fi
  done
}

previous_head_passed_checks() {
  local previous_head="$1"
  shift
  local groups=("$@")
  if [ "${#groups[@]}" -eq 0 ]; then
    return 0
  fi
  if [ -z "${GITHUB_REPOSITORY:-}" ] || [ -z "${GH_TOKEN:-}" ]; then
    return 1
  fi
  if ! command -v gh >/dev/null 2>&1; then
    return 1
  fi

  local check_file
  check_file="$(mktemp)"
  if ! gh api --paginate "repos/${GITHUB_REPOSITORY}/commits/${previous_head}/check-runs?per_page=100" \
    --jq '.check_runs[] | [.name, .conclusion] | @tsv' > "$check_file"; then
    rm -f "$check_file"
    return 1
  fi

  local missing=()
  for group in "${groups[@]}"; do
    while IFS= read -r check_name; do
      if [ -z "$check_name" ]; then
        continue
      fi
      if ! awk -F '\t' -v name="$check_name" '$1 == name && $2 == "success" { found = 1 } END { exit found ? 0 : 1 }' "$check_file"; then
        missing+=("$check_name")
      fi
    done < <(required_checks_for_group "$group")
  done
  rm -f "$check_file"

  if [ "${#missing[@]}" -gt 0 ]; then
    append_output previous_head_missing_checks "${missing[*]}"
    return 1
  fi
  return 0
}

emit_effective_scope() {
  local full_file="$1"
  local event_name="${GITHUB_EVENT_NAME:-}"
  if [ "$event_name" = "pull_request" ]; then
    local action before_sha head_sha
    action="$(event_value action)"
    before_sha="$(event_value before)"
    head_sha="$(event_value pull_request_head_sha)"
    if [ "$action" = "synchronize" ] && [ -n "$before_sha" ] && [ -n "$head_sha" ] && ! is_all_zero_sha "$before_sha"; then
      local delta_file
      delta_file="$(mktemp)"
      run_delta_scope "$delta_file" "$before_sha" "$head_sha"

      mapfile -t skipped_groups < <(skipped_full_scope_groups "$full_file" "$delta_file")
      if previous_head_passed_checks "$before_sha" "${skipped_groups[@]}"; then
        append_output scope_mode "delta"
        append_output scope_reason "previous head passed skipped full-scope checks"
        run_unprefixed_delta_scope "$before_sha" "$head_sha"
      else
        append_output scope_mode "full"
        append_output scope_reason "delta skipped full-scope checks without green previous-head evidence"
        run_unprefixed_full_scope
      fi
      rm -f "$delta_file"
      return
    fi
  fi

  append_output scope_mode "full"
  append_output scope_reason "event uses full scope"
  run_unprefixed_full_scope
}

full_file="$(mktemp)"
run_full_scope "$full_file"
emit_effective_scope "$full_file"
rm -f "$full_file"
