# Assura Roadmap

This is the high-level roadmap agents should use to orient work. Keep each epic
name short enough to scan quickly, then track concrete work in Trellis tasks.

## Roadmap Iterations And Epics

| Order | Epic | Status | Active/Open Work |
| --- | --- | --- | --- |
| 1 | Trellis Workflow Foundation | Completed | PR #1 merged; task archived under `.trellis/tasks/archive/2026-06/` |
| 2 | Tooling Baseline Cleanup | Completed | PRs #2-#4 merged; tasks archived under `.trellis/tasks/archive/2026-06/` |
| 3 | Assura Self-Check Clean | Completed | PR #5 merged; current self-check remains clean |
| 4 | Documentation Source Truth | Completed | Historical workflow systems are archived or marked historical |
| 5 | Windows CI Restore | Later | fix `libgit2-sys` MSVC linker failure and restore matrix entry |
| 6 | Beyond Ls-Lint Rules | Completed | PRs #8, #11, and #12 merged; performance/parity tasks archived |
| 7 | Agent Feedback MVP | Completed | PRs #13-#16 merged; keep stable `assura check --format agent` surface |
| 8 | Agentic Adoption Iteration 01 / Phase 01 | Active | `06-01-roadmap-phase-01-execution` |

## Active Roadmap Iteration

**Agentic Adoption Iteration 01 / Phase 01** is active.

Current owning task:
`.trellis/tasks/06-01-roadmap-phase-01-execution`.

Current branch:
`codex/phase-01-goal-07-extension-plugin-foundation`.

Current objective: execute the bounded Iteration 01 / Phase 01 sequence with
two-week team goals. Goals 01 through 06 are complete; the active chunk is
`docs/goals/assura-goal-07-extension-and-plugin-foundation.md`. Completing
Iteration 01 should create or identify the next roadmap iteration, not mark the
broader Assura roadmap complete.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

## Recommended Next Epic

Continue with
`docs/goals/assura-goal-07-extension-and-plugin-foundation.md` under
`docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`.
That document is the control plane for the active roadmap iteration; it is not
the product roadmap and should not be treated as the final roadmap completion
state.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new roadmap iteration or epic is needed, add it here and identify the
  first Trellis task that owns it.
- If an iteration or epic is active, say which task owns it and what the next
  recommended action is.
- Completing an iteration closes only that iteration; the roadmap remains open
  until a separate product decision declares the full roadmap complete.
