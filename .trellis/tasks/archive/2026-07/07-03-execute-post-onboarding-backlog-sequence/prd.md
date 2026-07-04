# Post-Merge Backlog Truth Pass

## Goal

Close the cleanup that follows merged PR #139 and leave the repository ready
for the next implementation chunk. The cleanup must preserve live truth:
agent-ready onboarding is merged, the old delivery branch is gone, stale goals
are closed or re-routed, and the next executable lane is explicit.

## Current State

- PR #139 merged on 2026-07-04 at
  `0020278066bc7498627ae9ef5a32bec54296ce73`.
- Cleanup branch is `codex/post-merge-backlog-truth-pass`.
- The old delivery branch `codex/agent-ready-onboarding-backlog` is historical.
- Performance polish remains the next core implementation lane.
- Compact project review/common-issues and LLM-wiki/OKF starters remain
  planned follow-up backlog items.

## Requirements

- Update roadmap/current-state text that still implies PR #139 is open or the
  old feature branch is active.
- Mark stale planned goals completed when later implementation or a narrower
  successor already closed or superseded the work.
- Keep `docs/goals/assura-performance-polish-program.md` as planned unless
  live evidence proves the broader performance lane is complete.
- Add or refine a compact project review/common-issues goal with measurable
  outcomes and reviewer blocking criteria.
- Preserve OKF and LLM-wiki/research-authoring starter work with validation,
  query, and native performance gates.
- Leave a copy/paste next-goal prompt.

## Acceptance Criteria

- [x] `.trellis/spec/assura/roadmap.md` names PR #139 as merged and routes next
      core work to performance polish.
- [x] `docs/data/public-roadmap.json` no longer shows PR handoff as current
      work.
- [x] The planned goal list under `docs/goals` only contains intentional future
      backlog items.
- [x] Superseded/closed goals include short revalidation notes rather than
      deleted history.
- [x] A compact project review/common-issues goal exists with scope, DoD,
      validation commands, review tasks, and a copy/paste prompt.
- [x] The LLM-wiki/OKF starter goal remains planned and keeps OKF validation,
      query, and native performance gates.
- [x] Validation commands pass or any blocker is recorded exactly.

## Validation Commands

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask target-state`
- `cargo xtask docs`
- `git diff --check`

## Out Of Scope

- No implementation of new performance code, OKF validators, agent harness
  installers, or compact-review commands in this cleanup pass.
- No broad rewrite of historical progress logs.
- No automatic `.assura/config.yml` changes.
