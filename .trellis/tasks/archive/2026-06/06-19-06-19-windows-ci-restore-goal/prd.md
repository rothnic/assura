# Define Windows CI Restore Goal

## Goal

Revalidate the deferred Windows CI restore roadmap item and create a bounded
goal that can be executed in a follow-up PR with hosted Windows evidence.

## Evidence

- `.trellis/spec/assura/roadmap.md` lists Windows CI Restore as the only
  non-completed roadmap item.
- `.trellis/spec/assura/tooling-stabilization.md` records the known
  `libgit2-sys` MSVC linker failure and the re-enable criteria.
- `.github/workflows/ci.yml` currently omits `windows-latest` from the Rust
  test matrix while release-scoped Windows smoke jobs remain defined.
- Current `cargo xtask target-state` passes, so this is a deferred tooling
  baseline gap rather than current source-tree drift.

## Acceptance Criteria

- [x] A planned goal doc exists for restoring Windows CI with objective, scope,
  non-goals, definition of done, validation commands, and reviewer blocking
  criteria.
- [x] The roadmap names Windows CI Restore as the next candidate without
  marking it active or complete.
- [x] The goal requires hosted `windows-latest` proof before completion.
- [x] Scoped docs/Trellis validation passes.
