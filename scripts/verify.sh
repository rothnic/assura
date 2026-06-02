#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/verify.sh <fast|check|test|docs|release-size|release-smoke|release-live|pr|full>

Modes:
  fast           Format, whitespace, focused compile checks, normal tests, and Assura self-check.
  check          Focused compile checks for the primary launcher and full companion.
  test           Rust tests excluding benchmark harness targets.
  docs           Website static build.
  release-size   Build the local installable release archive and enforce its size budget.
  release-smoke  Build and smoke the local installable release archive.
  release-live   Verify public no-auth install URLs for an already-published release.
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

run_trellis_state_check() {
  python3 - <<'PY'
import json
import pathlib
import re
import sys

errors = []
goal_statuses = {}
task_root = pathlib.Path(".trellis/tasks")
allowed_task_statuses = {"planning", "in_progress"}
for task_file in sorted(task_root.glob("*/task.json")):
    with task_file.open() as handle:
        task = json.load(handle)
    status = task.get("status")
    if status not in allowed_task_statuses:
        errors.append(
            f"{task_file}: active task status {status!r} should be archived or in progress/planning"
        )

allowed_goal_statuses = {"planned", "active", "completed", "archived"}
for goal_file in sorted(pathlib.Path("docs/goals").glob("*.md")):
    text = goal_file.read_text()
    frontmatter = re.match(r"---\n(.*?)\n---", text, re.DOTALL)
    if not frontmatter:
        continue
    status_match = re.search(r"^status:\s*(\S+)\s*$", frontmatter.group(1), re.MULTILINE)
    if not status_match:
        errors.append(f"{goal_file}: missing frontmatter status")
        continue
    status = status_match.group(1)
    if status not in allowed_goal_statuses:
        errors.append(
            f"{goal_file}: unsupported goal status {status!r}; expected one of {sorted(allowed_goal_statuses)}"
        )
    goal_statuses[goal_file.name] = status

phase_plan = pathlib.Path("docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md")
phase_goal_files = {
    1: "assura-goal-01-trustworthy-self-enforcement.md",
    2: "assura-goal-02-policy-language-completeness.md",
    3: "assura-goal-03-agent-feedback-delivery-loop.md",
    4: "assura-goal-04-fast-incremental-check-engine.md",
    5: "assura-goal-05-installable-adoption-path.md",
    6: "assura-goal-06-review-evidence-and-quality-gates.md",
    7: "assura-goal-07-extension-and-plugin-foundation.md",
    8: "assura-goal-08-release-readiness-and-ecosystem.md",
}
ledger_statuses = {}
if phase_plan.exists():
    for line in phase_plan.read_text().splitlines():
        match = re.match(r"^\|\s*(\d+)\.\s+[^|]+\|\s+([A-Za-z]+)\s+\|", line)
        if match:
            order = int(match.group(1))
            ledger_statuses[order] = match.group(2).lower()

    missing_orders = sorted(set(phase_goal_files) - set(ledger_statuses))
    for order in missing_orders:
        errors.append(f"{phase_plan}: missing Iteration 01 ledger row for goal {order}")

    for order, file_name in phase_goal_files.items():
        expected = ledger_statuses.get(order)
        actual = goal_statuses.get(file_name)
        if actual is None:
            errors.append(f"docs/goals/{file_name}: missing Iteration 01 goal file")
        elif expected is not None and actual != expected:
            errors.append(
                f"docs/goals/{file_name}: frontmatter status {actual!r} does not match Iteration 01 ledger status {expected!r}"
            )

    allowed_active_goals = {
        "assura-roadmap-phase-01-agentic-adoption-foundation.md",
        *{
            file_name
            for order, file_name in phase_goal_files.items()
            if ledger_statuses.get(order) == "active"
        },
    }
    for file_name, status in goal_statuses.items():
        if status == "active" and file_name not in allowed_active_goals:
            errors.append(
                f"docs/goals/{file_name}: active status is not listed as active in the Phase 01 ledger"
            )

if errors:
    for error in errors:
        print(error, file=sys.stderr)
    sys.exit(1)

print("Trellis task and goal status state is clean.")
PY
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
  local archive install_dir
  run_release_bundle
  archive="$RELEASE_ARCHIVE"

  tmp="$(mktemp -d)"
  cleanup() {
    rm -rf "$tmp"
  }
  trap cleanup EXIT INT TERM

  install_dir="$tmp/bin"
  ASSURA_ASSET_URL="$PWD/$archive" BIN_DIR="$install_dir" ./website/public/install.sh
  "$install_dir/assura" --help >"$tmp/assura-help.txt"
  grep -q "Usage: assura" "$tmp/assura-help.txt"
  ASSURA_BIN="$install_dir/assura" ASSURA_SMOKE_DIR="$tmp/adoption" \
    ./scripts/smoke-install-adoption.sh
}

public_url_ok() {
  local url status
  url="$1"
  status="$(curl -I -L -s -o /dev/null -w '%{http_code}' "$url")"
  printf '%s %s\n' "$status" "$url"
  test "$status" = "200"
}

run_release_live() {
  local repo version release_base
  repo="${ASSURA_REPO:-rothnic/assura}"
  version="${ASSURA_VERSION:-latest}"

  if [ "$version" = "latest" ]; then
    release_base="https://github.com/$repo/releases/latest/download"
  else
    release_base="https://github.com/$repo/releases/download/$version"
  fi

  public_url_ok "https://raw.githubusercontent.com/$repo/master/website/public/install.sh"
  public_url_ok "https://raw.githubusercontent.com/$repo/master/website/public/install.ps1"
  public_url_ok "$release_base/assura-linux-amd64.tar.gz"
  public_url_ok "$release_base/assura-macos-amd64.tar.gz"
  public_url_ok "$release_base/assura-macos-arm64.tar.gz"
  public_url_ok "$release_base/assura-windows-amd64.zip"
}

case "$mode" in
  fast)
    cargo fmt --all -- --check
    git diff --check
    run_check
    run_test
    run_self_check
    run_trellis_state_check
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
  release-live)
    run_release_live
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
