# Release cadence and README professionalization

## Goal

Clarify why Assura has not produced a new GitHub Release since `v0.1.0`, make
the release trigger and current version state obvious, and improve the README so
it reads like a professional project entrypoint without overstating unsupported
surfaces.

## Evidence

- Latest GitHub Release: `v0.1.0`, published 2026-05-24.
- Current package versions remain `0.1.0` in `Cargo.toml` and companion crates.
- `.github/workflows/release.yml` only runs on pushed `v*` tags.
- Push and PR CI builds continue to run and publish short-lived artifacts, but
  they do not create durable GitHub Releases.
- Live release verification initially failed because `v0.1.0` had archives but
  not `.sha256` release assets. The missing checksum assets were generated from
  the published archives and uploaded to the existing release.

## Requirements

- Explain that releases are tag-driven and that normal CI builds are not
  versioned releases.
- Improve README communication for first-time reviewers: status, install,
  supported command surface, verification, release process, and contribution
  pointers.
- Keep claims aligned with `docs/support-policy.md` and
  `docs/compatibility-and-surface.md`.
- Do not create a new release or bump versions in this task unless explicitly
  requested after the review.

## Acceptance Criteria

- [x] README accurately distinguishes current release, CI builds, and release
      workflow trigger.
- [x] README contains professional project signals: status, install options,
      core commands, supported surface, development checks, release process,
      contribution/support/security pointers, and license.
- [x] Release workflow reason for no newer release is documented in active docs
      or README.
- [x] `cargo xtask release-live` passes against the current public release.
- [x] `cargo run --quiet -- check --format json .` passes.
- [x] `cargo xtask evidence` passes.
- [x] `cargo xtask target-state` passes.
- [x] `git diff --check` passes.
- [x] Workspace ends clean with a PR ready for review.
