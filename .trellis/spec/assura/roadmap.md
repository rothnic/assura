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
| 7 | Agent Feedback MVP | Review | PR #16 merged; keep stable `assura check --format agent` surface |
| 8 | Product Roadmap Sequence | Active | `06-01-assura-roadmap-goal-sequence` |

## Active Epic

**Product Roadmap Sequence** is active.

Current owning task:
`.trellis/tasks/06-01-assura-roadmap-goal-sequence`.

Current objective: publish a master goal plus sequenced two-week roadmap goals
that define the next major chunks of Assura product work and the review tasks
required to prove each goal complete.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

## Recommended Next Epic

After this roadmap PR is reviewed, start
`docs/goals/assura-goal-01-trustworthy-self-enforcement.md` from
`docs/goals/assura-product-roadmap-master-goal.md`.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
