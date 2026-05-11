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
| 6 | Beyond Ls-Lint Rules | Active | `05-10-ls-lint-structure-parity-and-agent-integrations` |

## Active Epic

**Beyond Ls-Lint Rules** is active.

Current owning task:
`.trellis/tasks/05-10-ls-lint-structure-parity-and-agent-integrations`.

Current objective: make Assura enforce expected direct project contents and keep
installable agent integrations under `integrations/agents/`.

## Recommended Next Epic

After the current structure-enforcement PR is reviewed, continue
**Beyond Ls-Lint Rules** only where it improves dogfooding signal.

The first recommended follow-up is the Codex integration runtime hook design,
building on the skeleton created under `integrations/agents/codex/`.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
