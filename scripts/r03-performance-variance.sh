#!/usr/bin/env bash
# Collect strict, repeatable R03 cold-comparison evidence on one runner class.
#
# This diagnostic never widens or bypasses the production no-slower gate. It
# intentionally exits nonzero when any retained exact gate fails, after
# collecting every requested identical-binary A/A repetition and its raw report.

set -euo pipefail

run_count="${ASSURA_PERF_VARIANCE_RUNS:-3}"
output_dir="${ASSURA_PERF_VARIANCE_OUTPUT_DIR:-target/performance/r03-performance-variance}"
requested_commit="${ASSURA_PERF_VARIANCE_COMMIT:-}"

if ! [[ "$run_count" =~ ^[0-9]+$ ]] || (( run_count < 3 )); then
  echo "ASSURA_PERF_VARIANCE_RUNS must be an integer of at least 3" >&2
  exit 2
fi

if ! [[ "$requested_commit" =~ ^[0-9a-f]{40}$ ]]; then
  echo "ASSURA_PERF_VARIANCE_COMMIT must be a full 40-character commit SHA" >&2
  exit 2
fi

if [[ "$(git rev-parse HEAD)" != "$requested_commit" ]]; then
  echo "checked-out commit does not match ASSURA_PERF_VARIANCE_COMMIT" >&2
  exit 2
fi

if [[ -e "$output_dir" ]]; then
  echo "refusing to overwrite diagnostic output: $output_dir" >&2
  exit 2
fi

mkdir -p "$output_dir"
git rev-parse HEAD > "$output_dir/source-commit.txt"
git status --short > "$output_dir/source-status.txt"

cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo rustc --release --bin assura --no-default-features --features json-output,yaml-config -- -C target-feature=+crt-static -C link-arg=-lgcc_eh
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli

sha256sum target/release/assura target/release/assura-full target/release/assura-check > "$output_dir/binaries-before.sha256"
printf 'run\treport_exit\tgate_exit\tbinary_identity_exit\treport_sha256\n' > "$output_dir/records.tsv"

overall_exit=0
for (( run = 1; run <= run_count; run += 1 )); do
  report="$output_dir/report-${run}.json"

  set +e
  target/release/assura performance-report --output "$report" --iterations 16
  report_exit=$?
  if (( report_exit == 0 )); then
    cargo xtask performance-no-slower "$report"
    gate_exit=$?
  else
    gate_exit=125
  fi
  sha256sum -c "$output_dir/binaries-before.sha256" > "$output_dir/binaries-after-${run}.txt" 2>&1
  binary_identity_exit=$?
  set -e

  if [[ -f "$report" ]]; then
    sha256sum "$report" | awk '{print $1}' > "$output_dir/report-${run}.sha256"
    report_sha256="$(<"$output_dir/report-${run}.sha256")"
    jq -r '.results[] | select(.fixture_id == "many_configured_scopes_regression" and (.row_family == "assura-cli" or .row_family == "ls-lint-cli" or (.row_family | startswith("assura:phase:")))) | [.row_family,.median_runtime_ms,.p95_runtime_ms] | @tsv' "$report" > "$output_dir/report-${run}-many-scope.tsv"
  else
    report_sha256="missing"
  fi

  printf '%s\t%s\t%s\t%s\t%s\n' "$run" "$report_exit" "$gate_exit" "$binary_identity_exit" "$report_sha256" >> "$output_dir/records.tsv"
  if (( report_exit != 0 || gate_exit != 0 || binary_identity_exit != 0 )); then
    overall_exit=1
  fi
done

sha256sum target/release/assura target/release/assura-full target/release/assura-check > "$output_dir/binaries-after.sha256"
if ! cmp -s "$output_dir/binaries-before.sha256" "$output_dir/binaries-after.sha256"; then
  echo "measured binary identities changed during collection" >&2
  overall_exit=1
fi

if (( overall_exit != 0 )); then
  echo "R03 diagnostic retained one or more exact gate or identity failures" >&2
  exit 1
fi
