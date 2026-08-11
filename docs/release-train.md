---
title: Release Train
status: active
---

# Release Train

Assura stays below `1.0.0` until the public compatibility contract is declared
stable. Pre-1.0 releases should be small, intentional, and tied to installable
GitHub artifacts.

## Version Selection

- Use patch releases such as `0.1.1` for fixes, documentation-aligned release
  metadata, and compatible improvements to already-supported surfaces.
- Use minor releases such as `0.2.0` for new supported or experimental CLI
  surfaces, daemon lifecycle commands, editor packages, or output contracts.
- Use release candidates such as `0.2.0-rc.1` when a larger surface needs
  installable validation before a final pre-1.0 release.

## Readiness Command

Run the release-readiness command before opening a release PR:

```bash
cargo xtask website-demo-data --check --released
cargo xtask release-readiness --format json
```

The first command is the marketed-release gate. It rejects every core website
claim that is not `supported`, lacks verified or measured evidence, or first
ships after the local candidate version. Ordinary preview builds use
`website-demo-data --check` so unreleased candidate work can still be reviewed.

It emits `assura.release-readiness.v1` with:

- the latest GitHub release tag and publication time;
- the local package version and expected tag;
- the release-notes version;
- unreleased user-facing changes from `docs/data/release-surfaces.json`;
- missing checklist gates; and
- a pass/fail readiness verdict.

The command exits nonzero when versions, release notes, support policy,
checklist gates, latest GitHub release state, or unreleased public surfaces are
inconsistent.

Automation should parse JSON from stdout. Stderr is diagnostic output from
Cargo or from the failing readiness verdict.

## Release Surface Manifest

`docs/data/release-surfaces.json` is the structured source for release-surface
state. Supported or experimental surfaces with `"first_release": "unreleased"`
block every release candidate, regardless of the latest GitHub release.

Rows marked `"marketing_claim": true` must use `"status": "supported"`. A
release candidate must also assign each marketed row to the candidate tag or an
earlier release; `unreleased` is never valid at publication time.

Roadmap-only surfaces stay in the manifest as `"status": "roadmap"` with a
future release marker until their own beta goals prove installable support. When
a beta goal promotes a supported or experimental user-facing surface, set its
`first_release` to the prepared local tag and let `cargo xtask
release-readiness --format json` prove no supported surface remains
unreleased.

## Release PR Contract

A release PR should include:

- the version bump;
- release notes for that version;
- compatibility and support-policy updates when support status changed;
- the release-candidate checklist;
- `cargo xtask release-readiness --format json` output;
- local `cargo xtask release-smoke` evidence; and
- the planned post-tag `cargo xtask release-live` command.

Do not advertise daemon, editor-package, or agent integration support as
installable until the release-readiness check, release artifacts, and live
release verification prove that users can install the advertised surface.
