---
title: Beta v0.2.0 release publication evidence
status: active
---

# Beta v0.2.0 Release Publication Evidence

## Objective

Finish the beta master goal by publishing the already-green release candidate
through the project release path and recording live release evidence.

## Scope

- Mark PR #112 ready and merge it if the live check rollup remains green.
- Tag the merged commit as `v0.2.0` using the release checklist process.
- Wait for the GitHub release workflow to publish assets and checksum sidecars.
- Run the live release verification command.
- Update the beta master goal with PR merge, tag, release URL, asset evidence,
  validation commands, and independent completion review.

## Non-Goals

- Do not add new beta product surfaces.
- Do not change release automation unless the tag workflow fails and diagnosis
  proves an automation defect.
- Do not mark the persistent goal complete until the release evidence exists.

## Acceptance

- PR #112 is merged to `master`.
- `v0.2.0` exists as a remote tag.
- GitHub release `v0.2.0` exists with all expected archives and `.sha256`
  files.
- `ASSURA_VERSION=v0.2.0 cargo xtask release-live` passes.
- The beta master goal records the final evidence and no longer describes the
  release as missing.

## Completion Evidence

- PR #112 merged at `2026-07-01T15:17:20Z` with merge commit
  `1ea9f95138e38fef9283687a0fc7e2256adfc23d`.
- Remote tag `v0.2.0` peels to the same merge commit.
- Release workflow run `28528132216` published
  <https://github.com/rothnic/assura/releases/tag/v0.2.0> at
  `2026-07-01T15:24:36Z`.
- The release contains the five expected platform archives and five `.sha256`
  sidecars.
- `ASSURA_VERSION=v0.2.0 cargo xtask release-live` passes.
- `cargo xtask release-readiness --format json` reports `ready: true`.
- `docs/goals/assura-beta-code-agnostic-capabilities-program.md` records the
  final evidence and is marked `status: completed`.
