# Post-Beta Support Release Hardening

## Goal

Close the final child goal for the post-beta capabilities parent by aligning
support classifications, compatibility docs, release surfaces, release notes,
target-state gates, and release-readiness evidence for the next beta increment.

## What I Already Know

- PR #135 is merged as `a4b5e8ba4b6382d271ca5e9eea30e2f5ad2e29da`.
- `v0.2.0` is already published with the expected five archives and checksum
  sidecars.
- `cargo xtask release-readiness --format json` currently fails because local
  version/release notes still describe `0.2.0` while release surfaces include
  unreleased user-facing changes.
- The parent north-star scenario must remain the acceptance lens: a maintainer
  can decide merge/block/targeted repair from one concrete workflow.
- This work remains beta-track and must not claim GA or post-beta completion.

## Requirements

- Prepare the next beta increment as `0.3.0` unless live release-readiness
  evidence shows a different version is already required.
- Align package versions, release notes, release-surface manifest, support
  policy, compatibility matrix, public roadmap, and website release-readiness
  docs.
- Keep daemon mode experimental, VS Code beta package supported, extension API
  boundaries supported documentation, and public plugin APIs roadmap-only.
- Add or tighten target-state checks so future docs cannot omit the parent
  north-star scenario, the `v0.3.0` release surface entries, or the beta-only
  support language.
- Record release readiness, release-smoke, and validation evidence in the
  child and parent goals.
- Do not tag or publish until PR validation is merged; release publication is
  a subsequent explicit step in the parent goal.

## Acceptance Criteria

- [x] `cargo xtask release-readiness --format json` passes locally for the
      prepared beta increment.
- [x] Release notes describe the increment accurately and do not claim GA.
- [x] `docs/data/release-surfaces.json` has no `unreleased` supported or
      experimental user-facing surfaces that belong to this beta increment.
- [x] Support policy, compatibility matrix, website release-readiness page, and
      public roadmap agree on promoted/deferred surfaces.
- [x] Target-state checks cover the new release-hardening invariants.
- [x] The parent and child goals record current evidence and the next action.
- [x] Validation commands pass and independent review finds no blocker.

## Definition Of Done

- Local branch is clean, reviewed, and merged via PR.
- Release-readiness and release-smoke evidence is recorded.
- The parent goal remains active unless the beta increment is actually tagged,
  published, and live release evidence is recorded.

## Technical Approach

Use repo-local release-readiness tooling as the source of truth. Treat docs/data
alignment as implementation because `xtask target-state` and
`release-readiness` enforce these public support contracts.

## Out Of Scope

- Publishing the `v0.3.0` tag before the release PR is merged.
- Promoting daemon mode or agent nudges beyond their current beta/experimental
  support levels.
- Adding new runtime features except small target-state checks.

## Technical Notes

- Parent goal: `docs/goals/assura-post-beta-capabilities-program.md`
- Child goal: `docs/goals/assura-post-beta-support-release-hardening.md`
- Release surfaces: `docs/data/release-surfaces.json`
- Release notes: `docs/release-notes.md`
- Compatibility and support: `docs/compatibility-and-surface.md`,
  `docs/support-policy.md`
- Website release page:
  `website/src/content/docs/reference/release-readiness.md`
