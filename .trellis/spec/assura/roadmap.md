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
| 7 | Agent Feedback MVP | Active | `05-29-real-project-agentic-feedback-proof` |

## Active Epic

**Agent Feedback MVP** is active.

Current owning task:
`.trellis/tasks/05-29-real-project-agentic-feedback-proof`.

Current objective: prove a realistic project policy, local hook verification,
agent-neutral feedback output, same-turn observation evidence, and user-facing
docs without claiming unsupported daemon or native hook behavior.

## Recommended Next Epic

After the current feedback MVP is reviewed, continue **Agent Feedback MVP** only
where it improves dogfooding signal.

The first recommended follow-up is a real Codex hook installation design that
uses the MVP feedback library without making hook behavior mandatory for ordinary
developer workflows.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
