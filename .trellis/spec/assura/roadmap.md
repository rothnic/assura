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
| 8 | Agentic Adoption Iteration 01 / Phase 01 | Completed | Archived task `.trellis/tasks/archive/2026-06/06-01-roadmap-phase-01-execution` |
| 9 | Policy Depth Iteration 02 | Completed | Goals 09-13 merged and archived under `.trellis/tasks/archive/2026-06/` |
| 10 | Release Contract Rules | Completed | First reusable release-contract rule slice merged in PR #59; task archived in PR #60 |
| 11 | Public Surface Matrix | Completed | PR #61 merged; archive PR #62 merged |
| 12 | Cargo Manifest Semantics | Completed | PR #64 merged; archive PR #65 merged |
| 13 | Test Relationship Rule | Completed | PR #68 merged; archive PR #69 merged |
| 14 | Module Topology Rule | Completed | PR #72 merged; archive PR #73 merged |
| 15 | Docs Lifecycle Rule | Completed | PR #78 merged; archive PR #79 merged |

## Active Roadmap Iteration

No roadmap iteration is active after Iteration 01 completion.

Current owning task:
None.

Current branch:
None.

Most recent completed iteration: Policy Depth Iteration 02. Goals 09 through
13 are complete and archived, ending with Goal 13 PR #55 and archive PR #56.
Release Contract Rules first slice is complete via PR #59 and archive PR #60.
Public Surface Matrix first slice is complete via PR #61 and archive PR #62.
Cargo Manifest Semantics first slice is complete via PR #64 and archive PR
#65.
Test Relationship Rule first slice is complete via PR #68 and archive PR #69.
Module Topology Rule first slice is complete via PR #72 and archive PR #73.
Docs Lifecycle Rule first slice is complete via PR #78 and archive PR #79.
This completion does not mark the broader Assura roadmap complete.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

Planned next roadmap candidate:
Create and validate a follow-up goal for broadening docs lifecycle coverage
beyond the completed first slice. Use live target-state evidence to decide
whether the next slice should cover active analysis docs, active goal docs,
website docs, stale roadmap/performance claim tokens, or additional historical
exception patterns.

## Recommended Next Epic

Validate the next docs lifecycle coverage goal.

The next agent should use `assura-goal-validation` to create or revalidate a
new goal for broadening `extensions.docs_lifecycles` dogfood coverage beyond
the first merged slice. Start from current `.assura/config.yml`, this roadmap,
and `docs/analysis/2026-06-09-assura-best-practice-target-state.md`; do not
start broad cleanup until the follow-up detector scope is concrete and
reviewable. The product roadmap remains open until a separate product decision
declares it complete.

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
