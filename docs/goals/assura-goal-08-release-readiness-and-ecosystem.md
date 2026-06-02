---
id: goal-assura-roadmap-08-release-readiness-and-ecosystem
type: goal
title: Assura roadmap 08 release readiness and ecosystem
status: active
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - docs/release-notes.md
  - docs/release-candidate-checklist.md
  - docs/support-policy.md
  - docs/compatibility-and-surface.md
  - website/src/content/docs/
---

# Goal 08: Release Readiness And Ecosystem

## Objective

Prepare Assura for a public pre-1.0 release with clear support boundaries,
installable artifacts, compatibility claims, and next-roadmap guidance.

This is a two-week team chunk for release, docs, CI, and product owners.

## Scope

- Produce a release candidate checklist that covers builds, installers, docs,
  website, examples, and known limitations.
- Update release notes with current supported surfaces and removed/superseded
  surfaces.
- Verify artifact checksums, install scripts, and version output.
- Publish compatibility language for LS-Lint migration and Assura-specific
  extensions.
- Add support policy for issues, bug reports, docs fixes, and future breaking
  changes before 1.0.
- Define the next roadmap iteration after Iteration 01.

## Non-Goals

- No 1.0 semantic-versioning commitment.
- No hosted SaaS launch.
- No plugin marketplace.

## Definition Of Done

- Release candidate artifacts build and smoke-test on supported platforms.
- Website docs describe current stable commands and limitations accurately.
- Release notes identify breaking changes, removed experimental surfaces, and
  supported feedback delivery.
- Compatibility claims are backed by checked tests or reports.
- The next roadmap iteration is linked from this goal and the Iteration 01
  ledger.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo run --quiet -- check --format json .
node --run verify:fast
node --run verify:docs
node --run verify:release-smoke
node --run verify:evidence
cd website && npx pnpm@10.25.0 build
git diff --check
```

## Review Tasks

- R0: Confirm release scope matches the current roadmap iteration state.
- R1: Review every public compatibility and performance claim against evidence.
- R2: Review release artifact and installer smoke results.
- R3: Reproduce install and first-check flow from published artifacts.
- R4: Review website navigation and release notes for stale claims.
- R5: Confirm the next roadmap iteration and support policy are explicit.

## Reviewer Blocking Criteria

Block the PR if release notes imply 1.0 stability, if artifacts cannot be
installed from documented commands, if compatibility claims lack tests, or if
removed feedback surfaces reappear as supported paths.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-02 | Started Goal 08 from updated `master` after Goal 07 merged; moved the active Trellis task to `codex/phase-01-goal-08-release-readiness`. | `gh pr view 24 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-08-release-readiness`. |
| 2026-06-02 | Added release readiness artifacts: release notes, release candidate checklist, support policy, compatibility matrix, website release readiness page, and planned Iteration 02 handoff. | `docs/release-notes.md`; `docs/release-candidate-checklist.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `website/src/content/docs/reference/release-readiness.md`; `docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md`. |
