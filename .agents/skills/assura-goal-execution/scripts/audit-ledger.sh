#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: audit-ledger.sh <repository> [task-relative-path]

Read the canonical backlog and declared execution branches from a Git
revision. The command is read-only and emits tab-separated routing records.
Set ASSURA_BASE_REF to override the default origin/master.
USAGE
}

if [[ ${1:-} == "-h" || ${1:-} == "--help" ]]; then
  usage
  exit 0
fi

repo_root=${1:-}
task_rel=${2:-.trellis/tasks/09-04-maturity-portfolio-strategy}
base_ref=${ASSURA_BASE_REF:-origin/master}

if [[ -z "$repo_root" ]]; then
  usage >&2
  exit 2
fi

command -v git >/dev/null
command -v jq >/dev/null

repo_root=$(git -C "$repo_root" rev-parse --show-toplevel)
base_sha=$(git -C "$repo_root" rev-parse "$base_ref^{commit}")
ledger_path="$task_rel/research/backlog.json"
task_path="$task_rel/task.json"
# Resolve the symbolic base once, then load every task blob by that immutable
# object ID. This keeps the advertised BASE and all routing records coherent
# even if the remote-tracking ref moves while the command is running.
ledger_json=$(git -C "$repo_root" show "$base_sha:$ledger_path")
task_json=$(git -C "$repo_root" show "$base_sha:$task_path")

printf 'BASE\t%s\t%s\n' "$base_ref" "$base_sha"
printf '%s\n' "$ledger_json" | jq -e '.items | type == "array"' >/dev/null
printf 'TASK\t%s\t%s\n' "$task_rel" "$(printf '%s\n' "$task_json" | jq -r '.status // "unknown"')"

printf '%s\n' "$ledger_json" | jq -r '
  .items[]
  | ["CARD", .id, (.state // "unknown"), (.owner // ""), ((.depends_on // []) | join(",")), (.evidence // ""), (.blocker // "")]
  | @tsv
'

printf '%s\n' "$ledger_json" | jq -r '
  . as $root
  | .items[]
  | select(.state == "pending")
  | . as $item
  | ($item.depends_on // []) as $dependencies
  | ([($dependencies)[] as $dependency
      | ($root.items[] | select(.id == $dependency) | .state)]
    ) as $dependency_states
  | select(
      ($dependency_states | length) == ($dependencies | length)
      and ($dependency_states | all(.[]; . == "done" or . == "verified"))
    )
  | ["READY_PENDING", .id, (.owner // ""), ((.depends_on // []) | join(",")), (.evidence // "")]
  | @tsv
'

printf '%s\n' "$ledger_json" | jq -r '
  .items[]
  | select(.state == "active" or .state == "implemented" or .state == "verified" or .state == "blocked")
  | ["UNFINISHED", .id, .state, (.owner // ""), (.evidence // ""), (.blocker // "")]
  | @tsv
'

worktree_for_branch() {
  local target_ref="refs/heads/$1"
  local worktree_record=""
  local candidate_dir=""
  while IFS= read -r -d '' worktree_record; do
    case "$worktree_record" in
      "worktree "*) candidate_dir=${worktree_record#worktree } ;;
      "branch "*)
        if [[ "$worktree_record" == "branch $target_ref" ]]; then
          printf '%s\n' "$candidate_dir"
          return 0
        fi
        ;;
    esac
  done < <(git -C "$repo_root" worktree list --porcelain -z)
}

printf '%s\n' "$task_json" | jq -r '.meta.execution_branches // [] | .[]' | while IFS= read -r branch_name; do
  branch_sha="absent"
  ref_state="absent"
  if git -C "$repo_root" show-ref --verify --quiet "refs/heads/$branch_name"; then
    branch_sha=$(git -C "$repo_root" rev-parse "refs/heads/$branch_name")
    ref_state="local"
  elif git -C "$repo_root" show-ref --verify --quiet "refs/remotes/origin/$branch_name"; then
    branch_sha=$(git -C "$repo_root" rev-parse "refs/remotes/origin/$branch_name")
    ref_state="remote"
  fi
  worktree_dir=$(worktree_for_branch "$branch_name")
  if [[ -z "$worktree_dir" ]]; then
    worktree_dir="absent"
  fi
  printf 'BRANCH\t%s\t%s\t%s\t%s\n' "$branch_name" "$ref_state" "$branch_sha" "$worktree_dir"
done

printf '%s\n' "$ledger_json" | jq -r '
  . as $root
  | [.items[]] as $items
  | ($items | map(select(.state == "pending"))) as $pending
  | ($pending | map(select(
      . as $item
      | ($item.depends_on // []) as $dependencies
      | ([($dependencies)[] as $dependency
          | ($root.items[] | select(.id == $dependency) | .state)]
        ) as $dependency_states
      | ($dependency_states | length) == ($dependencies | length)
        and ($dependency_states | all(.[]; . == "done" or . == "verified"))
    ))) as $ready
  | ["SUMMARY",
     ("items=" + (($items | length) | tostring)),
     ("ready_pending=" + (($ready | length) | tostring)),
     ("nonterminal=" + (($items | map(select(.state != "done" and .state != "not_needed")) | length) | tostring)),
     ("done=" + (($items | map(select(.state == "done")) | length) | tostring)),
     ("not_needed=" + (($items | map(select(.state == "not_needed")) | length) | tostring)),
     ("held=" + (($items | map(select(.state == "blocked")) | length) | tostring))]
  | @tsv
'
