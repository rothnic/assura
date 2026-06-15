# Clean remote PRs and merge canonical notation

## Goal

Return Assura to a clean handoff state: no uncommitted changes, no detached
local-only work carrying meaningful changes, no stale active PRs, and the
latest useful canonical notation and compiled-artifact freshness work merged
through normal review gates.

## Requirements

- Preserve the detached local commit containing canonical relationship notation
  before rebasing or switching branches.
- Do not preserve the stale local release-readme Trellis task commit in the
  final merge path.
- Merge the relevant compiled-artifact freshness PR if its current checks are
  healthy.
- Close or supersede stale remote PRs that conflict with the latest canonical
  notation direction.
- Rebuild canonical notation work on current `origin/master` by cherry-picking
  only the meaningful notation commit.
- Keep alpha-era backwards compatibility out of the final result.
- Leave local `master` aligned with `origin/master` and delete temporary local
  branches after the work is reachable from the remote.
- Record the next chunks of work, including a performance analysis task, in a
  durable project location.

## Acceptance Criteria

- [ ] A safety branch exists for the detached local notation commit until the
      work is merged remotely.
- [ ] PR #48 is merged or explicitly documented as blocked/stale.
- [ ] PR #46 and PR #47 are closed or explicitly documented as still relevant.
- [ ] Canonical notation changes are merged from a clean branch based on the
      current `origin/master`.
- [ ] The stale release-readme Trellis task commit is not included in the final
      notation PR.
- [ ] Validation passes: `cargo test --all-targets --quiet`,
      `cargo run --quiet -- check --format json .`, `cargo xtask evidence`,
      `cargo xtask docs`, `cargo fmt --all -- --check`, and
      `git diff --check`.
- [ ] A review agent checks the final branch before PR creation or merge.
- [ ] The final workspace is clean: no uncommitted changes, no local-only
      commits on `master`, no relevant open PRs, and no lingering temporary
      branch needed for recovery.

## Definition of Done

- Useful work is merged through PRs.
- Superseded PRs are closed with clear rationale.
- Local checkout is on `master`, up to date with `origin/master`, and clean.
- The follow-up roadmap is captured in repo-native task or planning files.

## Technical Approach

1. Create a temporary safety branch pointing at the detached local notation
   commit.
2. Refresh GitHub state and merge the clean freshness PR before applying the
   notation work.
3. Close remote PRs that are superseded by the no-backwards-compatibility
   canonical notation direction.
4. Create a clean branch from updated `origin/master`, cherry-pick only the
   meaningful notation commit, resolve conflicts in favor of current master
   plus the canonical notation implementation, then validate.
5. Run a review agent against the branch and address valid findings.
6. Open and merge the canonical notation PR, then clean local branches and
   confirm the repository/PR state.

## Decision (ADR-lite)

Context: The current detached HEAD contains one useful notation commit and one
stale task commit, while remote `master` has already advanced and contains PR
#45 documentation and release-readme cleanup.

Decision: Preserve the detached HEAD with a safety branch, but rebuild final
work from `origin/master` and cherry-pick only the useful canonical notation
commit.

Consequences: This avoids accidental rollback of newer documentation and
Trellis cleanup while keeping the notation work recoverable until remote merge
is complete.

## Out of Scope

- Redesigning notation beyond the already-approved canonical relationship
  direction.
- Preserving removed alpha notation or compatibility shims.
- Implementing performance optimizations before the follow-up performance
  analysis task ranks opportunities.

## Technical Notes

- Useful detached commit: `a9f1c57 feat(config): add canonical relationship notation`.
- Stale detached commit not intended for merge: `2461b2f docs(trellis): record release readme task`.
- Relevant PRs at planning time: #46 compact notation, #47 stale freshness
  branch, #48 clean freshness rebuild.
- Live default branch is `master`, not `main`.
