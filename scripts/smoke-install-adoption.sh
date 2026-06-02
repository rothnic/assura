#!/usr/bin/env bash
set -euo pipefail

assura_bin="${ASSURA_BIN:-${1:-assura}}"

if [ ! -x "$assura_bin" ] && ! command -v "$assura_bin" >/dev/null 2>&1; then
  printf 'assura adoption smoke: binary not found: %s\n' "$assura_bin" >&2
  exit 2
fi

if [ -n "${ASSURA_SMOKE_DIR:-}" ]; then
  work_root="$ASSURA_SMOKE_DIR"
  mkdir -p "$work_root"
  cleanup() { :; }
else
  work_root="$(mktemp -d)"
  cleanup() {
    rm -rf "$work_root"
  }
fi
trap cleanup EXIT INT TERM

assert_json_field() {
  local file expr
  file="$1"
  expr="$2"
  python3 - "$file" "$expr" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
expr = sys.argv[2]
data = json.loads(path.read_text())

def array_len(value):
    return len(value) if isinstance(value, list) else 0

def positive_int(value):
    return isinstance(value, int) and value > 0

if expr == "success_true":
    ok = data.get("success") is True
elif expr == "success_false_with_violations":
    ok = data.get("success") is False and array_len(data.get("violations")) > 0
elif expr == "status_has_config":
    ok = bool(data.get("config_path")) and positive_int(data.get("configured_directories"))
else:
    raise SystemExit(f"unknown assertion: {expr}")

if not ok:
    raise SystemExit(f"{path}: assertion failed: {expr}\n{json.dumps(data, indent=2)}")
PY
}

run_failing_check() {
  local project output status
  project="$1"
  output="$2"

  set +e
  "$assura_bin" check --format json "$project" >"$output"
  status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    printf 'assura adoption smoke: expected failing check for %s\n' "$project" >&2
    cat "$output" >&2
    exit 1
  fi
  if [ "$status" -ne 1 ]; then
    printf 'assura adoption smoke: expected exit 1, got %s for %s\n' "$status" "$project" >&2
    cat "$output" >&2
    exit 1
  fi
}

printf 'assura adoption smoke: binary=%s\n' "$assura_bin"
"$assura_bin" --version

empty_project="$work_root/empty-project"
mkdir -p "$empty_project"
printf '# Empty Project\n' >"$empty_project/README.md"

"$assura_bin" init "$empty_project" --no-git-hooks
test -f "$empty_project/.assura/config.yml"

"$assura_bin" status "$empty_project" --format json >"$work_root/empty-status.json"
assert_json_field "$work_root/empty-status.json" status_has_config

"$assura_bin" check --format json "$empty_project" >"$work_root/empty-check-pass.json"
assert_json_field "$work_root/empty-check-pass.json" success_true

printf 'fn main() {}\n' >"$empty_project/BadName.rs"
run_failing_check "$empty_project" "$work_root/empty-check-fail.json"
assert_json_field "$work_root/empty-check-fail.json" success_false_with_violations

lslint_project="$work_root/ls-lint-project"
mkdir -p "$lslint_project"
cat >"$lslint_project/.ls-lint.yml" <<'YAML'
ls:
  .dir: kebab-case
  .rs: snake_case
ignore:
  - target
YAML
printf 'fn main() {}\n' >"$lslint_project/good_name.rs"

"$assura_bin" migrate "$lslint_project/.ls-lint.yml" \
  --output "$lslint_project/.assura/config.yml"
test -f "$lslint_project/.assura/config.yml"

"$assura_bin" status "$lslint_project" --format json >"$work_root/lslint-status.json"
assert_json_field "$work_root/lslint-status.json" status_has_config

"$assura_bin" check --format json "$lslint_project" >"$work_root/lslint-check-pass.json"
assert_json_field "$work_root/lslint-check-pass.json" success_true

printf 'assura adoption smoke: pass; evidence=%s\n' "$work_root"
