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

## CI Preview Artifacts

Release-scoped CI jobs produce temporary installable previews named with the
package version and full source SHA, such as
`assura-v<package-version>-<full-source-SHA>-linux-amd64-preview.tar.gz`.
The workflow checks that bundled binaries report the same package version and
retains these previews for 14 days. They are validation artifacts, not public
releases; durable archives are still produced only by the tag-driven release workflow
after an explicit version tag is pushed.

## Version-to-release procedure

For an authorized release, use this sequence so a version increment cannot
silently produce mixed artifacts:

1. Set the intended workspace/package versions and lock metadata using the
   repository's existing versioning practice. Update `docs/release-notes.md`
   and any release-surface rows that first ship in that version.
2. Run `cargo xtask release-readiness --format json`, the local release smoke,
   and the focused release-contract tests. Resolve every failure before the
   release PR is reviewed.
3. After the release PR is merged, verify the versioned preview artifacts and
   their full source SHA. Create the release task bound to the chosen merged
   commit only when publication authority is available.
4. The tag-driven workflow verifies the tag/package version, both binary
   `--version` outputs in every build matrix entry, both binaries again after
   archive extraction, checksums, and an installable Linux proof. It writes an
   `assura.release-receipt.v1` evidence artifact containing the tag
   object/commit, workflow run, five archive identities, and install proof.
5. On a retry, matching release assets are skipped after size and SHA-256
   verification. Missing assets are uploaded; a conflicting or unverifiable
   existing asset stops the workflow for investigation. No retry overwrites a
   durable asset implicitly.
6. Once the release exists, run the explicit-version and latest installer
   checks from the release-live procedure. Those checks are publication
   evidence, not a substitute for explicit release acceptance or release
   authority.

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
- focused release-contract tests covering wrong-version binaries/archives,
  checksum mismatches, missing assets, and safe publication retries; and
- the planned post-tag `cargo xtask release-live` command.

Do not advertise daemon, editor-package, or agent integration support as
installable until the release-readiness check, release artifacts, and live
release verification prove that users can install the advertised surface.
