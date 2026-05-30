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
| 6 | Beyond Ls-Lint Rules | Active | `05-11-structure-check-benchmark-attribution` |
| 7 | Agent Feedback MVP | Active | `05-30-codex-hook-agent-feedback` |

## Active Epic

**Agent Feedback MVP** is active.

Current owning task:
`.trellis/tasks/05-30-codex-hook-agent-feedback`.

Current objective: prove the optional native Codex hook path that injects
Assura feedback without making hook behavior mandatory for ordinary developer
workflows.

## Recommended Next Epic

After the current hook-feedback task is reviewed, continue **Agent Nudge MVP**
only where it improves dogfooding signal. The next likely follow-up is a
separate install/status/verify command that can inspect or merge hook
configuration safely; do not treat that as implemented until it has tests.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
