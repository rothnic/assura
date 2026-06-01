# Assura self-check baseline cleanup

## Goal

Reduce the repository's Assura self-check baseline to zero violations so
`cargo run -- check .` becomes a trustworthy local and CI signal.

## What I already know

- This task starts from latest `master` on branch
  `codex/assura-self-check-baseline-cleanup`.
- PR #1 merged Assura self-enforcement and Trellis governance.
- PR #2 merged repository-wide rustfmt cleanup.
- PR #3 improved CI cache behavior and temporarily made Clippy advisory.
- PR #4 cleaned the Clippy baseline and made Clippy blocking again.
- Current self-check baseline is 94 medium violations.
- Current hook policy is advisory until the repo passes its own
  `.assura/config.yml` baseline consistently.

## Baseline Classification

- `directory_naming`: 6
- `file_naming`: 22
- `markdown_frontmatter`: 28
- `max_lines`: 25
- `require_docs`: 13

The full machine-readable baseline is in `research/baseline.json`; the
human-readable command output is in `research/baseline-text.txt`.

## Requirements

- Fix Assura self-check violations through source-of-truth cleanup, not broad
  suppressions.
- Keep the scope limited to Assura self-check baseline cleanup.
- Preserve public CLI behavior, config schema behavior, library API behavior,
  and CI platform policy.
- Treat Trellis as canonical and OpenSpec / `specs-bak/` as historical unless a
  newer ADR changes that.
- Leave `.assura/hooks/pre-push` advisory unless the project specs explicitly
  require blocking hooks in this PR.
- Update `.trellis/spec/assura/tooling-stabilization.md` with the clean
  baseline state and next policy step.

## Acceptance Criteria

- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test --all-targets --quiet` passes.
- [x] `cargo run -- check .` reports zero violations.
- [x] `cargo run --quiet -- check --format json .` reports success and zero
      violations.
- [x] The PR does not restore Windows CI or change unrelated runtime behavior.
- [x] Renamed or moved docs have updated references.

## Out of Scope

- Restoring Windows CI.
- Adding a new spec, task, documentation, or workflow system.
- Changing user-facing Assura validation semantics except where needed to
  encode legitimate source-of-truth or archive policy.
- Expanding beyond this repository's self-check baseline.

## Technical Notes

- Canonical project guidance:
  - `AGENTS.md`
  - `.trellis/workflow.md`
  - `.trellis/spec/assura/index.md`
  - `.trellis/spec/assura/roadmap.md`
  - `.trellis/spec/assura/tooling-stabilization.md`
  - `.trellis/spec/assura/workflow-status.md`
- Historical source-of-truth treatment is documented in
  `docs/analysis/2026-05-09-trellis-governance-adr.md` and
  `docs/analysis/2026-05-09-documentation-cleanup-register.md`.
