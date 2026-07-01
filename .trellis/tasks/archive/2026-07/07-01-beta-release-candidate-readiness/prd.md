# Beta Release Candidate Readiness

## Objective

Prepare the repo for the next pre-1.0 beta release after the ten beta roadmap
epics. The immediate target is making `cargo xtask release-readiness --format
json` pass for the next release version while preserving honest support and
experimental classifications.

## Current Gap

The beta roadmap epics are complete, but the master beta program cannot be
marked complete because the latest GitHub release is still `v0.1.0` and this
branch contains unreleased supported and experimental public surfaces. The
release-readiness gate correctly fails in that state.

## Scope

- Select the next pre-1.0 release version from the release train policy.
- Align root package, check-only package, xtask package, release notes, release
  checklist, and release-surface metadata.
- Keep supported, experimental, roadmap, and unsupported surfaces consistent
  across release docs.
- Make release-readiness pass locally before any tag/publish action.
- Run focused release validation and independent review.

## Non-Goals

- Do not push a tag or publish GitHub release artifacts in this task.
- Do not promote roadmap-only long-running socket daemon behavior.
- Do not mark the master beta program complete until a tag and release artifact
  actually exist with validation evidence.
- Do not rerun heavyweight full-suite checks repeatedly while iterating; use
  focused release gates first.

## Acceptance Criteria

- `cargo xtask release-readiness --format json` passes.
- Version metadata and release notes agree on the selected next release.
- Release surface entries do not leave beta-included supported/experimental
  surfaces ambiguously marked as `unreleased` or `next`.
- Release docs still distinguish experimental daemon/editor/agent previews from
  unsupported roadmap behavior.
- Independent reviewer finds no blocker or high-risk release-readiness gaps.
- Master beta program records the readiness slice and still names tag/artifact
  publication as the remaining completion blocker.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo xtask release-readiness --format json
cargo xtask release-smoke
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
git diff --check
```
