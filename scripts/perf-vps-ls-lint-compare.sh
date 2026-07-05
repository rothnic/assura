#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  scripts/perf-vps-ls-lint-compare.sh <label> <repo-path> [<repo-path>...]

Options:
  --host <ssh-host>           SSH host alias. Default: vps
  --remote-root <path>        Remote workspace root. Default: <remote-home>/data/projects
  --iterations <count>        performance-report iterations. Default: 5
  --fixture-id <fixture>      Fixture row to summarize. Default: many_configured_scopes_regression
  --exact-fixture <path>      Shared fixture path for exact-command tie-breakers.
                              Default: <remote-home>/data/projects/manual-many-scopes-common
  --no-exact                  Skip the exact-command tie-breaker
  -h, --help                  Show this help

Examples:
  scripts/perf-vps-ls-lint-compare.sh fast-naming-lazy-path \
    src/cli/check/ls_fast.rs \
    src/cli/check/ls_fast_naming.rs \
    src/cli/check/ls_fast_plan_tests.rs

The final summary prints the target fixture phase deltas, an
accepted_fixture_delta table for every accepted LS-Lint-equivalent fixture, and
exact public-command deltas when the shared fixture is available.
EOF
}

host="${ASSURA_PERF_VPS_HOST:-vps}"
remote_root="${ASSURA_PERF_VPS_REMOTE_ROOT:-}"
iterations="5"
fixture_id="many_configured_scopes_regression"
exact_fixture="${ASSURA_PERF_VPS_EXACT_FIXTURE:-}"
exact_enabled="1"
label=""
paths=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host)
      host="${2:?missing value for --host}"
      shift 2
      ;;
    --remote-root)
      remote_root="${2:?missing value for --remote-root}"
      shift 2
      ;;
    --iterations)
      iterations="${2:?missing value for --iterations}"
      shift 2
      ;;
    --fixture-id)
      fixture_id="${2:?missing value for --fixture-id}"
      shift 2
      ;;
    --exact-fixture)
      exact_fixture="${2:?missing value for --exact-fixture}"
      exact_enabled="1"
      shift 2
      ;;
    --no-exact)
      exact_fixture=""
      exact_enabled="0"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [[ -z "$label" ]]; then
        label="$1"
      else
        paths+=("$1")
      fi
      shift
      ;;
  esac
done

if [[ -z "$label" || ${#paths[@]} -eq 0 ]]; then
  usage >&2
  exit 2
fi

if [[ -z "$remote_root" ]]; then
  remote_home="$(ssh "$host" 'printf %s "$HOME"')"
  remote_root="${remote_home}/data/projects"
fi

if [[ "$exact_enabled" == "1" && -z "$exact_fixture" ]]; then
  exact_fixture="${remote_root}/manual-many-scopes-common"
fi

timestamp="$(date +%Y%m%d)"
remote_dir="${remote_root}/assura-perf-${timestamp}-${label}"
patch_file="/tmp/assura-${label}.patch"
remote_patch="${remote_dir}/artifacts/${label}.patch"

git diff -- "${paths[@]}" > "$patch_file"
if [[ ! -s "$patch_file" ]]; then
  printf 'No diff found for the requested paths.\n' >&2
  exit 1
fi

printf '==> Remote workspace: %s:%s\n' "$host" "$remote_dir"
ssh "$host" "rm -rf '$remote_dir' && mkdir -p '$remote_dir/artifacts' '$remote_dir/after'"
rsync -az --delete \
  --exclude .git \
  --exclude target \
  --exclude website/node_modules \
  --exclude .trellis/.runtime \
  ./ "$host:$remote_dir/after/"
scp "$patch_file" "$host:$remote_patch"
ssh "$host" "cd '$remote_dir' && cp -a after before && cd before && patch -p1 -R < ../artifacts/${label}.patch"

ssh "$host" "bash -s" -- "$remote_dir" "$iterations" "$fixture_id" "$exact_fixture" <<'EOF'
set -euo pipefail

remote_dir="$1"
iterations="$2"
fixture_id="$3"
exact_fixture="$4"

build_report() {
  local dir="$1"
  local output="$2"
  local history="$3"
  local website_dir="$4"
  cd "$dir"
  cargo build --release --bin assura --no-default-features --features json-output,yaml-config
  cargo build --release --bin assura-full
  cargo build --release -p assura-check-cli
  target/release/assura performance-report \
    --output "$output" \
    --history "$history" \
    --website-dir "$website_dir" \
    --iterations "$iterations" \
    --suite ls-lint
}

build_report \
  "$remote_dir/before" \
  target/performance/before.json \
  target/performance/before.jsonl \
  target/performance/before-website

build_report \
  "$remote_dir/after" \
  target/performance/after.json \
  target/performance/after.jsonl \
  target/performance/after-website

cd "$remote_dir/after"
cargo xtask performance-no-slower target/performance/after.json

if [[ -n "$exact_fixture" && -d "$exact_fixture" ]]; then
  if command -v hyperfine >/dev/null 2>&1; then
    cd "$exact_fixture"
    hyperfine \
      --warmup 3 \
      --runs 30 \
      --export-json "$remote_dir/artifacts/exact-assura-check.json" \
      "$remote_dir/before/target/release/assura check --quiet >/dev/null" \
      "$remote_dir/after/target/release/assura check --quiet >/dev/null"
    hyperfine \
      --warmup 3 \
      --runs 30 \
      --export-json "$remote_dir/artifacts/exact-assura-check-cli.json" \
      "$remote_dir/before/target/release/assura-check --quiet >/dev/null" \
      "$remote_dir/after/target/release/assura-check --quiet >/dev/null"
  else
    python3 - "$remote_dir" "$exact_fixture" <<'PY'
import json
import statistics
import subprocess
import sys
import time
from pathlib import Path

remote_dir = Path(sys.argv[1])
fixture_dir = Path(sys.argv[2])

def measure(output_name, before_cmd, after_cmd):
    commands = [
        {
            "command": " ".join(before_cmd) + " >/dev/null",
            "args": before_cmd,
            "samples": [],
        },
        {
            "command": " ".join(after_cmd) + " >/dev/null",
            "args": after_cmd,
            "samples": [],
        },
    ]
    for command in commands:
        for _ in range(3):
            subprocess.run(
                command["args"],
                cwd=fixture_dir,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
    for _ in range(30):
        for command in commands:
            started = time.perf_counter()
            subprocess.run(
                command["args"],
                cwd=fixture_dir,
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            command["samples"].append(time.perf_counter() - started)
    payload = {
        "runner": "python-fallback",
        "results": [
            {
                "command": command["command"],
                "mean": statistics.fmean(command["samples"]),
                "times": command["samples"],
            }
            for command in commands
        ],
    }
    (remote_dir / "artifacts" / output_name).write_text(json.dumps(payload))

measure(
    "exact-assura-check.json",
    [f"{remote_dir}/before/target/release/assura", "check", "--quiet"],
    [f"{remote_dir}/after/target/release/assura", "check", "--quiet"],
)
measure(
    "exact-assura-check-cli.json",
    [f"{remote_dir}/before/target/release/assura-check", "--quiet"],
    [f"{remote_dir}/after/target/release/assura-check", "--quiet"],
)
PY
  fi
fi

python3 - "$remote_dir" "$fixture_id" <<'PY'
import json
import sys
from pathlib import Path

remote_dir = Path(sys.argv[1])
fixture_id = sys.argv[2]

before = json.loads((remote_dir / "before/target/performance/before.json").read_text())
after = json.loads((remote_dir / "after/target/performance/after.json").read_text())

rows = [
    "assura-cli",
    "assura-check-cli",
    "assura-in-process",
    "assura:phase:config-load",
    "assura:phase:checker-init",
    "assura:phase:configured-structure",
    "assura:phase:walk-and-validate",
]

def pick(doc, tool_name):
    for row in doc["results"]:
        if row.get("fixture_id") == fixture_id and row.get("tool_name") == tool_name:
            return row["median_runtime_ms"]
    raise KeyError(tool_name)

print("fixture\tbefore_ms\tafter_ms\tdelta_pct")
for tool in rows:
    before_ms = pick(before, tool)
    after_ms = pick(after, tool)
    delta_pct = ((after_ms - before_ms) / before_ms) * 100 if before_ms else 0.0
    print(f"{tool}\t{before_ms:.6f}\t{after_ms:.6f}\t{delta_pct:+.1f}%")

before_total = before["claim_summary"]["total_assura_runtime_ms"]
after_total = after["claim_summary"]["total_assura_runtime_ms"]
delta_pct = ((after_total - before_total) / before_total) * 100 if before_total else 0.0
print(
    "claim_summary.total_assura_runtime_ms"
    f"\t{before_total:.6f}\t{after_total:.6f}\t{delta_pct:+.1f}%"
)

def accepted_assura_rows(doc):
    rows_by_fixture = {}
    for row in doc["results"]:
        if (
            row.get("fixture_cohort") == "realistic-equivalent"
            and row.get("fixture_acceptance") == "accepted-ls-lint-equivalent"
        ):
            rows_by_fixture.setdefault(row["fixture_id"], {})[row.get("row_family")] = row
    return rows_by_fixture

before_by_fixture = accepted_assura_rows(before)
after_by_fixture = accepted_assura_rows(after)
fixtures = sorted(set(before_by_fixture) | set(after_by_fixture))
print("\naccepted_fixture_delta")
print("fixture\tbefore_assura_ms\tafter_assura_ms\tdelta_pct\tafter_ls_lint_ms\tno_slower\t2x_status\tgap_to_2x_ms")
for fixture in fixtures:
    before_row = before_by_fixture.get(fixture, {}).get("assura-cli")
    after_row = after_by_fixture.get(fixture, {}).get("assura-cli")
    after_ls_lint = after_by_fixture.get(fixture, {}).get("ls-lint-cli")
    if before_row is None or after_row is None or after_ls_lint is None:
        print(f"{fixture}\tmissing\tmissing\tmissing\tmissing\tmissing\tmissing\tmissing")
        continue
    before_ms = before_row["median_runtime_ms"]
    after_ms = after_row["median_runtime_ms"]
    after_ls_ms = after_row.get("native_ls_lint_median_runtime_ms") or after_ls_lint["median_runtime_ms"]
    delta_pct = ((after_ms - before_ms) / before_ms) * 100 if before_ms else 0.0
    target = after_row.get("two_x_target_runtime_ms")
    gap = after_ms - target if target is not None else None
    no_slower = "yes" if after_ms <= after_ls_ms else "no"
    two_x_status = after_row.get("two_x_claim_status") or "unknown"
    gap_text = f"{gap:+.6f}" if gap is not None else "n/a"
    print(
        f"{fixture}\t{before_ms:.6f}\t{after_ms:.6f}\t{delta_pct:+.1f}%"
        f"\t{after_ls_ms:.6f}\t{no_slower}\t{two_x_status}\t{gap_text}"
    )

for artifact_name in ("exact-assura-check.json", "exact-assura-check-cli.json"):
    artifact = remote_dir / "artifacts" / artifact_name
    if not artifact.exists():
        continue
    payload = json.loads(artifact.read_text())
    print(f"\n{artifact_name}")
    exact_results = payload.get("results", [])
    means = []
    for result in exact_results:
        mean_ms = result["mean"] * 1000.0
        means.append(mean_ms)
        print(f"{result['command']}\t{mean_ms:.6f}")
    if len(means) == 2 and means[0]:
        delta_pct = ((means[1] - means[0]) / means[0]) * 100
        print(f"exact_delta_pct\t{delta_pct:+.1f}%")
PY
EOF

printf '\n==> Patch: %s\n' "$patch_file"
printf '==> Remote artifacts: %s:%s\n' "$host" "$remote_dir"
