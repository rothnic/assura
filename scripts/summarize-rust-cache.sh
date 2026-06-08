#!/usr/bin/env bash
set -euo pipefail

job_name="${1:-${GITHUB_JOB:-unknown}}"
cache_hit="${2:-unknown}"
shared_key="${3:-unknown}"
summary_file="${GITHUB_STEP_SUMMARY:-}"

if [ -z "$summary_file" ]; then
  printf 'Rust cache: job=%s shared-key=%s cache-hit=%s\n' "$job_name" "$shared_key" "$cache_hit"
  exit 0
fi

{
  printf '## Rust cache\n\n'
  printf -- '- Job: `%s`\n' "$job_name"
  printf -- '- Shared key: `%s`\n' "$shared_key"
  printf -- '- Exact cache hit: `%s`\n' "$cache_hit"
} >> "$summary_file"
