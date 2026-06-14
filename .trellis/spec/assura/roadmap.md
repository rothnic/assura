# Assura Roadmap

This is the high-level roadmap agents should use to orient work. Keep each epic
name short enough to scan quickly, then track concrete work in Trellis tasks.

## Epic Roadmap

| Order | Epic | Status | Active/Open Work |
| --- | --- | --- | --- |
| 1 | Trellis Workflow Foundation | Active | `00-bootstrap-guidelines` |
| 2 | Tooling Baseline Cleanup | Review | rustfmt, Clippy, CI cache, and self-check cleanup PRs |
| 3 | Assura Self-Check Clean | Review | keep `cargo run -- check .` clean through normal review/merge flow |
| 4 | Documentation Source Truth | Next | migrate, archive, or delete stale docs and workflow artifacts |
| 5 | Windows CI Restore | Later | fix `libgit2-sys` MSVC linker failure and restore matrix entry |
| 6 | Beyond Ls-Lint Rules | Active | `05-21-bring-pr11-performance-home` |
| 7 | Agent Nudge MVP | Next | resume only after the current PR 11 performance lane is coherent and validated |

## Active Epic

**Beyond Ls-Lint Rules** is active.

Current owning task:
`.trellis/tasks/05-21-bring-pr11-performance-home`.

Current objective: finish the PR 11 performance lane on
`codex/ls-lint-realistic-parity-core-performance` so the branch is coherent,
validated, and ready for a truthful PR update covering the scoped Linux
static-CRT cold claim and the warm/editor-session claim. Fresh 2026-06-14
verification reruns turned green again: the canonical command `cargo test
--all-targets --quiet` passes, and all named exact repro commands also pass in
their current venues:

- `cargo test --quiet cli::check::prepared::tests::prepared_check_reloads_when_config_changes -- --exact`
- `cargo test --quiet cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix -- --exact`
- `cargo test -p assura --lib cli::check::compiled_artifact_tests::source_fingerprint_detects_same_size_rewrite_on_unix -- --exact`

Keep the branch in shaping mode anyway because the remaining blocker is now the
broad dirty review/handoff batch across `.trellis/spec/assura/roadmap.md`,
`.trellis/spec/assura/workflow-status.md`, `.trellis/tasks/05-21-bring-pr11-performance-home/prd.md`,
and the committed cache/fingerprint fixes in `src/cli/check/cache.rs`,
`src/cli/check/compiled_artifact.rs`, `src/cli/check/compiled_fingerprint.rs`,
and `src/cli/check/prepared.rs`. The next narrowing step is to collapse that
verified green checkpoint into one reviewable PR-facing slice instead of
continuing to narrate stale verification failures.

## Recommended Next Epic

After the current nudge MVP is reviewed, continue **Agent Nudge MVP** only
where it improves dogfooding signal.

The first recommended follow-up is a real Codex hook installation design that
uses the MVP nudge library without making hook behavior mandatory for ordinary
developer workflows.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
