---
title: Upstream contribution workflow
status: current
owner: maintainers
---

# Upstream contribution workflow

## Before editing

Identify the upstream repository and intended PR branch. Read its contribution
rules, issue and PR templates, CI configuration, and nearby code and tests.
Search issues and open PRs for the behavior. Sample a few recent merged PRs
of the same kind from contributors and maintainers. Record links and typical
patch sizes in a short brief, with documented requirements marked separately
from observations. Use the brief when drafting or reviewing.

## During development

Checkpoint coherent edits on a development branch. Where you own the local
hook setup, `assura hooks install .` provides advisory validation on development
branches by default. Respect existing hooks and any explicitly selected staged
enforcement. Keep the generated Codex integration's automatic patch-size signal
off by default. If opted in, configure `agent_feedback.trajectory.integration_ref`
for the intended comparison and use the output as approximate worktree feedback.
`assura agent nudge --delivery inspect` shows details on demand. Review size and
file-length policy are separate measurements.

## Before opening an issue or draft PR

Check for duplicates. If the behavior is already tracked, link the original
issue. If a PR already covers it, coordinate there. Check the issue draft for
a reproducible case and the upstream template's required evidence. Check the PR
draft for one reviewable change, a regression test, user-facing documentation
or release notes where applicable, and accurate validation claims. Follow the
upstream rules for when to open a draft.

For the committed PR patch, use the explicit review target even when the branch
tracks a fork. This shell function takes a Git remote name and target branch.
Run it in a clean candidate checkout. Confirm the fetch URL is the intended
upstream before using it. It prints the remote, branch, head commit, current
target commit, and unique merge base, one per line.

```sh
capture_pr_snapshot() (
    [ "$#" -eq 2 ] || { echo 'Expected an upstream remote and target branch.' >&2; exit 1; }
    ref="refs/heads/$2"
    git remote get-url -- "$1" >/dev/null || exit 1
    git check-ref-format "$ref" || exit 1
    git fetch --quiet --no-tags --no-write-fetch-head --refmap= -- "$1" "$ref" || exit 1
    target_line=$(git ls-remote --exit-code --refs -- "$1" "$ref") || exit 1
    target=${target_line%%[[:space:]]*}
    tab=$(printf '\t')
    [ "$target_line" = "$target$tab$ref" ] || exit 1
    status=$(git status --porcelain=v1 --untracked-files=normal) || exit 1
    [ -z "$status" ] || { echo 'Snapshot requires a clean checkout.' >&2; exit 1; }
    head=$(git rev-parse --verify HEAD) || exit 1
    base=$(git merge-base --all "$target" "$head") || exit 1
    case "$base" in
        ''|*'
'*) echo 'No unique merge base for this target and head.' >&2; exit 1 ;;
    esac
    printf '%s\n' "$1" "$2" "$head" "$target" "$base"
)

# Replace upstream and main with the verified remote name and target branch.
reviewed=$(capture_pr_snapshot upstream main) || exit 1
```

Save the snapshot. Inspect `git diff --stat --find-renames "$base" "$head"`
and `git diff --numstat --find-renames --no-ext-diff --no-textconv "$base" "$head"`
using its merge base and head. Record effective additions, deletions and changed
files. Binary rows have unknown line counts. For resulting text-file length,
count lines in the selected blob, for example `git show "$head:src/example.py"`
followed by `awk 'END { print NR }'` on the blob content. A shrinking rewrite
can still be a large patch to review. Where a project has Assura file-length
policy, run `assura check-change --base "$base" --head "$head"` on a build that
supports it. Preserve findings and explain large but coherent patches. If the
target is unavailable or the merge base is ambiguous, resolve the target or
fetch the necessary history before reporting a PR patch size.

## Before marking ready

Refresh the snapshot and verify the hosted PR still targets that repository
and branch and names that head. Re-read the complete patch against the brief
and run the upstream-required validation. Record what was tested, including the
integration tree when CI tests a merge commit. Review and validation evidence
must identify the head and target it covers. Re-run `capture_pr_snapshot` before
the ready action. Verify the remote URL and hosted head and target again. If
either changed or a required check is missing, refresh the evidence and keep
the PR in draft. Leave hosted actions to the user or authorized publishing
workflow.

```sh
latest=$(capture_pr_snapshot upstream main) || exit 1
[ "$reviewed" = "$latest" ] || { echo 'Head or target changed. Refresh evidence.' >&2; exit 1; }
```
