# Resolve versioned build artifact generation

## Goal

Make the current Assura source merge-ready and remove the ambiguity between
the latest published release and current source artifacts. The live published
release is `v0.3.0`, while `origin/master` currently declares package version
`0.4.0` and is release-ready locally. The release workflow is tag-driven, so
no durable `v0.4.0` assets can exist until an explicit `v0.4.0` tag is pushed.

## What I already know

* Current integration source: `origin/master` at `a1f6665e`.
* `Cargo.toml`, workspace crates, and `xtask` declare `0.4.0`.
* GitHub's latest published release is `v0.3.0`; its five platform archives
  and checksum sidecars are present.
* `.github/workflows/release.yml` runs only for `v*` tag pushes and publishes
  durable release assets from version-matched archives.
* `.github/workflows/ci.yml` builds installable preview artifacts on scoped
  master/PR changes, but names them `*-preview` and retains them for 14 days.
* A docs-only master push correctly skipped Release Bundle Smoke; a prior
  source push passed the release bundle and installable adoption jobs.
* `cargo xtask release-readiness --format json` currently passes for `0.4.0`.

## Requirements

* Preserve the known untracked A04 research file in the root checkout.
* Keep durable release publication tag-driven unless the owner explicitly
  authorizes cutting and publishing `v0.4.0`.
* Make the selected build-artifact path version-identifiable and assert the
  generated binary version rather than relying on a fixed preview filename.
* Add focused contract coverage for the artifact/version behavior.
* Update only the release/build instructions needed to explain preview versus
  durable release artifacts.
* Merge the verified implementation into `master` and verify post-merge jobs.

## Decision

Implement versioned CI preview artifacts now. Keep the durable `v0.4.0` tag
and publication as a separate explicit action; this task does not create or
push a release tag.

## Acceptance Criteria

* [x] The chosen preview artifact name and archive include the package version
  and exact source identity where the workflow can produce it.
* [x] The artifact contains binaries reporting the same version.
* [x] CI/release contract tests fail if version naming or version matching
  regresses.
* [ ] `cargo xtask release-readiness --format json`, focused tests, current
  base gates, and applicable hosted checks pass.
* [ ] The merged commit and post-merge workflow results are recorded.

## Definition of Done

* Tests and contract checks pass.
* Independent review is complete or its bounded local substitute is recorded.
* No unknown or foreign work is modified.
* No release/tag/publication claim is made without its corresponding evidence.

## Out of Scope

* Automatically tagging or publishing `v0.4.0` without explicit owner approval.
* Changing installer URLs before durable release assets exist.
* Release deployment, website publication, or broad worktree cleanup.

## Technical Notes

* Relevant files: `.github/workflows/ci.yml`, `.github/workflows/release.yml`,
  `scripts/ci-scope*.sh`, `docs/release-candidate-checklist.md`,
  `docs/release-train.md`, and `xtask/src/release_readiness.rs`.
* Hindsight confirms the release/process boundary: CI and process success are
  not product or release acceptance; exact source, artifact, and ownership
  evidence must remain separate.
