---
id: goal-assura-incremental-release-train
type: goal
title: Assura incremental release train
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ../release-candidate-checklist.md
  - ../release-notes.md
  - ../github-setup.md
  - ../../.github/workflows/release.yml
---

# Assura Incremental Release Train

## Objective

Make release publishing part of normal Assura progress before 1.0. Meaningful
CLI, validation, daemon, editor, and agent improvements should produce
incremental pre-1.0 versions and GitHub release artifacts instead of leaving
users on the May 2026 `v0.1.0` archives while `master` moves ahead.

## Current Gap

The latest public GitHub release is `v0.1.0`, published on 2026-05-24.
Assura has continued to merge Project Intelligence, Markdown, docs, and
planning work since then, but no newer installable GitHub release has been
published. Current docs already warn that later Project Intelligence work is
not present in the `v0.1.0` archives.

## Versioning Policy Before 1.0

- Stay below `1.0.0` until the public compatibility contract is intentionally
  declared stable.
- Use patch releases such as `0.1.1`, `0.1.2`, and `0.1.3` for bug fixes,
  docs-aligned release metadata, and compatible improvements to existing
  supported surfaces.
- Use minor releases such as `0.2.0` for new user-facing supported or
  experimental CLI surfaces, daemon lifecycle commands, editor packages, or
  output contracts.
- Use release candidates such as `0.2.0-rc.1` when a larger surface needs
  installable validation before a final pre-1.0 release.
- Every version bump must update `Cargo.toml`, release notes, and any install
  docs that name the current version.

## Scope

- Define when a merged feature requires a release candidate or release.
- Keep `.github/workflows/release.yml` as the GitHub release artifact
  publisher for version tags.
- Add a release-readiness check that compares the latest GitHub release tag,
  `Cargo.toml` version, release notes version, and unreleased user-facing
  changes.
- The intended command is `cargo xtask release-readiness --format json`. It
  should report latest GitHub release, local package version, release-note
  version, unreleased supported/experimental surfaces, missing checklist items,
  and a pass/fail release-readiness verdict.
- Make release PRs include the release-candidate checklist and live release
  verification commands.
- Require release artifacts for daemon/editor/agent milestones before docs
  advertise them as usable outside source builds.

## Non-Goals

- No `1.0.0` declaration.
- No publishing unsupported daemon/watch/editor surfaces as stable.
- No package-manager publishing requirement unless a later goal adds it.
- No automatic tag push from ordinary feature PRs.

## Definition Of Done

- A release-train check reports whether the repo has unreleased user-facing
  changes since the latest GitHub release.
- `cargo xtask release-readiness --format json` exists and exits nonzero when
  release notes, package version, support policy, latest GitHub release, or
  release checklist state are inconsistent.
- Release PRs include a version bump, release notes, compatibility/support
  policy updates when needed, and the release-candidate checklist.
- Tag publishing produces GitHub release archives and checksum sidecars through
  existing CI/CD.
- Documentation clearly states which version first contains each new supported
  or experimental surface.
- The Markdown Reference Intelligence program uses this goal before claiming
  daemon, VS Code, or agent-daemon features are available to install.

## Validation Commands

```bash
gh release list --limit 5
cargo xtask release-readiness --format json
cargo xtask release-smoke
cargo xtask docs
cargo xtask evidence
git diff --check
```

After publishing a tag:

```bash
cargo xtask release-live
ASSURA_VERSION=vX.Y.Z cargo xtask release-live
```

## Review Tasks

- R1: Confirm the latest GitHub release date/tag was checked before release
  planning claims were made.
- R2: Confirm the version bump matches pre-1.0 scope: patch for compatible
  improvements, minor for new supported/experimental surfaces.
- R3: Confirm release notes do not claim surfaces that support policy still
  marks unsupported or future.
- R4: Confirm GitHub release assets and checksums are verified after tag
  publication.

## Reviewer Blocking Criteria

Block if a release PR leaves `Cargo.toml`, release notes, docs, and tag names
out of sync; claims daemon/editor/agent features before artifacts include
them; skips release artifact smoke tests; or tries to declare `1.0.0` as part
of this pre-1.0 release train.
