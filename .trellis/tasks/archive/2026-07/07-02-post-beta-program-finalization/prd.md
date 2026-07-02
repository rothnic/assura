# Post-Beta Program Finalization

## Goal

Finalize the parent post-beta capabilities program after PR #136 by recording
merged support-hardening evidence, closing stale child/task state, publishing
the `v0.3.0` beta increment, verifying it live, and recording the parent
completion audit.

## What I Already Know

- PR #136 merged on 2026-07-02 as
  `a37d6245d9340237e8cf95452326fc8d4d1f5703`.
- `origin/master` contains the `0.3.0` package metadata and release-hardening
  work.
- `cargo xtask release-readiness --format json` passes locally for `v0.3.0`.
- `cargo xtask target-state` passes after archiving the completed
  support-hardening Trellis task.
- The `v0.3.0` tag and GitHub release are now published from merged `master`.
- The parent goal remains beta-track work. Finalization must not imply GA or
  post-beta product status.

## Requirements

- Record PR #136 merge evidence in the support-hardening child goal and parent
  goal progress log.
- Mark the support-hardening child complete only if current evidence proves its
  definition of done.
- Update roadmap routing so it no longer points at the merged support-hardening
  branch as active work.
- Keep the final north-star verification scenario as the acceptance lens for
  parent completion.
- Run local release and support gates before any tag/publish step.
- Verify GitHub release assets and checksums with the documented live gate and
  record evidence.
- Mark the parent completed only if the completion audit maps every requested
  capability to a beta-supported or explicitly experimental surface.

## Acceptance Criteria

- [x] Stale Trellis task state from support hardening is archived and committed.
- [x] Parent and child goal logs record PR #136 merge and the post-merge
      release-readiness state.
- [x] Roadmap current action points at either live `v0.3.0` release
      verification or the next explicitly deferred parent step.
- [x] `cargo xtask release-readiness --format json` passes.
- [x] `cargo xtask target-state` passes.
- [x] `cargo xtask release-smoke` passes before any tag is pushed.
- [x] Release workflow for `v0.3.0` succeeds and publishes all expected assets.
- [x] `cargo xtask release-live` and
      `ASSURA_VERSION=v0.3.0 cargo xtask release-live` pass.
- [x] Parent completion audit records the beta/pre-1.0 boundary.
- [x] Final validation evidence is recorded in the task and goal files.
- [x] Independent review finds no blocker before a finalization PR is opened.

## Validation Evidence

- `cargo xtask release-readiness --format json` passed for local package
  version `0.3.0`, local tag `v0.3.0`, and no unreleased user-facing surfaces.
- `cargo xtask release-smoke` passed and the preview binary reported
  `assura 0.3.0`.
- `cargo xtask target-state` passed after routing the roadmap to parent
  finalization while keeping the support-hardening goal linked.
- `cargo run --quiet -- check --format json .` passed with zero violations.
- `cargo xtask docs` passed and built 39 website pages.
- `cargo xtask evidence` passed review evidence and CI scope policy checks.
- `git diff --check` passed.
- Release workflow `28601020713` succeeded for tag `v0.3.0`.
- `gh release view v0.3.0 --json tagName,url,isDraft,isPrerelease,publishedAt,assets`
  confirmed a non-draft, non-prerelease release published at
  `2026-07-02T15:23:53Z` with five platform archives and five checksum files.
- `cargo xtask release-live` passed for the `latest` release URLs.
- `ASSURA_VERSION=v0.3.0 cargo xtask release-live` passed for the explicit
  versioned release URLs.
- Independent review agent `019f2358-2c2a-7432-aa0f-57a3a4bf65c1` found no
  blockers. It flagged that support-policy and website release-readiness did
  not repeat the unpublished-`v0.3.0` caveat strongly enough before the tag was
  published. After live verification, both pages now state that `v0.3.0` is
  published and live-verified while Assura remains pre-1.0 beta.

## Definition Of Done

- Finalization branch is clean, reviewed, pushed, and merged.
- The parent goal is complete after `v0.3.0` is tagged, published,
  live-verified, and the completion audit proves every parent requirement.

## Out Of Scope

- New daemon, Markdown, content graph, agent, editor, or performance features.
- Claiming GA or stable 1.0 support.
- Publishing `v0.3.0` without the release checklist gates.

## Technical Notes

- Parent goal: `docs/goals/assura-post-beta-capabilities-program.md`
- Release-hardening child: `docs/goals/assura-post-beta-support-release-hardening.md`
- Roadmap: `.trellis/spec/assura/roadmap.md`
- Checklist: `docs/release-candidate-checklist.md`
- Release readiness command: `cargo xtask release-readiness --format json`
- Live release command after publish:
  `ASSURA_VERSION=v0.3.0 cargo xtask release-live`
