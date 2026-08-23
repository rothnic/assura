# Prepare Assura v0.4.0 Release Metadata

## Goal

Prepare the next intentional pre-1.0 release candidate so the package version,
release documentation, capability manifest, generated landing data, and release
readiness gate agree on the public product already merged to `master`.

## Evidence

- `v0.3.0` is the latest published GitHub tag, dated 2026-07-02.
- The `master` branch now includes a new standalone landing and project-review
  experience, plus previously unshipped supported agent, watch, daemon,
  Markdown, and project-intelligence surfaces.
- `cargo xtask release-readiness --format json` fails because 19 supported
  user-facing manifest rows are still `first_release: unreleased`.
- The release train requires a minor pre-1.0 release for new supported or
  experimental CLI surfaces. The candidate is therefore `0.4.0`.

## Scope

- Bump every workspace package from `0.3.0` to `0.4.0`.
- Assign the supported public capability rows that already have evidence to
  `v0.4.0`.
- Update release notes and generated marketing data to the candidate version.
- Verify release-readiness and the released marketing-claim gate.

## Acceptance Criteria

- [ ] Every workspace package reports version `0.4.0`.
- [ ] `cargo xtask release-readiness --format json` reports `ready: true` for
      the candidate, except for the expected absence of the post-tag release.
- [ ] `cargo xtask website-demo-data --check --released` passes.
- [ ] Release notes, capability manifest, website demo data, and Cargo metadata
      contain no accidental `0.3.0` candidate references.
- [ ] `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check` pass.

## Out Of Scope

- Creating or pushing the `v0.4.0` tag or GitHub release.
- Changing product behavior, support policy, or benchmark thresholds.
- Promoting roadmap-only or unsupported capabilities.

## Release Follow-Up

After merge, a maintainer must run the release checklist, tag `v0.4.0`, and
allow the release workflow to publish and verify installable artifacts.
