#!/usr/bin/env bash
# Collect one independently scheduled, exact R03 calibration observation.
#
# This diagnostic preserves the production no-slower gate. Its metadata is
# intentionally incomplete when collection fails so the classifier can only
# return inconclusive, never a synthetic pass.

set -euo pipefail

requested_commit="${ASSURA_R03_CALIBRATION_COMMIT:-}"
slot="${ASSURA_R03_CALIBRATION_SLOT:-}"
output_dir="${ASSURA_R03_CALIBRATION_OUTPUT_DIR:-target/performance/r03-performance-calibration-${slot}}"
runner_image_name="${ImageOS:-}"
runner_image_version="${ImageVersion:-}"

if ! [[ "$requested_commit" =~ ^[0-9a-f]{40}$ ]]; then
  echo "ASSURA_R03_CALIBRATION_COMMIT must be a full 40-character commit SHA" >&2
  exit 2
fi

if ! [[ "$slot" =~ ^[123]$ ]]; then
  echo "ASSURA_R03_CALIBRATION_SLOT must be one of 1, 2, or 3" >&2
  exit 2
fi

if [[ -z "$runner_image_name" || -z "$runner_image_version" || "$runner_image_name" == *latest* || "$runner_image_version" == *latest* ]]; then
  echo "runner image identity must be nonempty and immutable; ubuntu-latest is not accepted" >&2
  exit 2
fi

if [[ -z "${GITHUB_RUN_ID:-}" || -z "${GITHUB_JOB:-}" ]]; then
  echo "GITHUB_RUN_ID and GITHUB_JOB are required for independent job provenance" >&2
  exit 2
fi

if [[ "$(git rev-parse HEAD)" != "$requested_commit" ]]; then
  echo "checked-out commit does not match ASSURA_R03_CALIBRATION_COMMIT" >&2
  exit 2
fi

if [[ -e "$output_dir" ]]; then
  echo "refusing to overwrite calibration output: $output_dir" >&2
  exit 2
fi

cpu_model="$(lscpu | awk -F: '/^Model name:/ {gsub(/^[[:space:]]+/, "", $2); print $2; exit}')"
if [[ -z "$cpu_model" ]]; then
  echo "CPU model is required for calibration provenance" >&2
  exit 2
fi

mkdir -p "$output_dir"
git rev-parse HEAD > "$output_dir/source-commit.txt"
git status --short > "$output_dir/source-status.txt"
printf '%s\n' "$cpu_model" > "$output_dir/cpu-model.txt"
printf '%s@%s\n' "$runner_image_name" "$runner_image_version" > "$output_dir/runner-image.txt"

set +e
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
build_assura_exit=$?
cargo rustc --release --bin assura --no-default-features --features json-output,yaml-config -- -C target-feature=+crt-static -C link-arg=-lgcc_eh
build_static_exit=$?
cargo build --release --bin assura-full
build_full_exit=$?
cargo build --release -p assura-check-cli
build_check_exit=$?
set -e

build_exit=0
if (( build_assura_exit != 0 || build_static_exit != 0 || build_full_exit != 0 || build_check_exit != 0 )); then
  build_exit=1
fi

binary_sha256="missing"
binary_identity_exit=125
report_exit=125
gate_exit=125
report_sha256="missing"
paired_deltas='[]'

if (( build_exit == 0 )); then
  sha256sum target/release/assura target/release/assura-full target/release/assura-check > "$output_dir/binaries-before.sha256"
  binary_sha256="$(sha256sum target/release/assura | awk '{print $1}')"
  report="$output_dir/report.json"
  set +e
  target/release/assura performance-report --output "$report" --iterations 16
  report_exit=$?
  if (( report_exit == 0 )); then
    cargo xtask performance-no-slower "$report"
    gate_exit=$?
  fi
  sha256sum -c "$output_dir/binaries-before.sha256" > "$output_dir/binaries-after.txt" 2>&1
  binary_identity_exit=$?
  set -e

  if [[ -f "$report" ]]; then
    report_sha256="$(sha256sum "$report" | awk '{print $1}')"
    paired_deltas="$(jq -c '
      ([.results[] | select(.fixture_id == "many_configured_scopes_regression" and .row_family == "assura-cli") | .distribution.samples_ms]) as $assura_rows
      | ([.results[] | select(.fixture_id == "many_configured_scopes_regression" and .row_family == "ls-lint-cli") | .distribution.samples_ms]) as $ls_lint_rows
      | if ($assura_rows | length) == 1 and ($ls_lint_rows | length) == 1 and ($assura_rows[0] | length) == 16 and ($ls_lint_rows[0] | length) == 16
        then [range(0; 16) | $assura_rows[0][.] - $ls_lint_rows[0][.]]
        else []
        end
    ' "$report")"
  fi

  if (( binary_identity_exit != 0 )); then
    # Preserve the raw report and hash-check output, but never emit a record
    # that the classifier can mistake for an immutable-binary observation.
    binary_sha256="missing"
    paired_deltas='[]'
  fi
fi

job_id="${GITHUB_RUN_ID}:${GITHUB_RUN_ATTEMPT:-1}:${GITHUB_JOB}:${slot}"
jq -n \
  --arg job_id "$job_id" \
  --arg cpu_model "$cpu_model" \
  --arg runner_image_name "$runner_image_name" \
  --arg runner_image_version "$runner_image_version" \
  --arg source_sha "$requested_commit" \
  --arg assura_binary_sha256 "$binary_sha256" \
  --arg report_sha256 "$report_sha256" \
  --argjson report_exit "$report_exit" \
  --argjson gate_exit "$gate_exit" \
  --argjson build_exit "$build_exit" \
  --argjson binary_identity_exit "$binary_identity_exit" \
  --argjson paired_deltas_ms "$paired_deltas" \
  '{job_id: $job_id, cpu_model: $cpu_model, runner_image_name: $runner_image_name, runner_image_version: $runner_image_version, source_sha: $source_sha, assura_binary_sha256: $assura_binary_sha256, report_sha256: $report_sha256, report_exit: $report_exit, gate_exit: $gate_exit, build_exit: $build_exit, binary_identity_exit: $binary_identity_exit, paired_deltas_ms: $paired_deltas_ms}' \
  > "$output_dir/run.json"

overall_exit=0
if (( build_exit != 0 || report_exit != 0 || gate_exit != 0 || binary_identity_exit != 0 )); then
  overall_exit=1
fi
if [[ "$(jq 'length' <<<"$paired_deltas")" != "16" ]]; then
  overall_exit=1
fi

if (( overall_exit != 0 )); then
  echo "R03 calibration retained a build, report, strict gate, binary identity, or pair extraction failure" >&2
fi
exit "$overall_exit"
