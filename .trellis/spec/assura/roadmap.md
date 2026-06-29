---
title: Assura Roadmap
status: active
---

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
| 5 | Windows CI Restore | Completed | PR #93 restored `windows-latest` to the Rust Test Suite matrix with hosted Windows proof |
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
| 16 | Docs Lifecycle Coverage | Completed | PR #83 merged; archive/sync PR #84 merged |
| 17 | Support Matrix Surface Expansion | Completed | PR #88 merged; archive/sync PR #89 merged |
| 18 | Project Intelligence Runtime | Completed | All nine successors and final completion audit completed locally |
| 19 | Project Intelligence Usability | Active | New post-runtime goal set created locally; start with the adoption blueprint |

## Active Roadmap Iteration

Project Intelligence Usability is active.

Most recent owning task:
`.trellis/tasks/06-29-project-intelligence-usability-goals`, creating the
post-runtime usability goal set.

Current branch:
`codex/project-intelligence-agent-surfaces`.

Current recommended goal:
`docs/goals/assura-project-intelligence-adoption-blueprint.md`.

Current program:
`docs/goals/assura-project-intelligence-usability-program.md`.

Project Intelligence Runtime is complete and should remain closed unless a new
concrete drift case is named with evidence.

Most recent completed Project Intelligence successor:
`docs/goals/assura-project-intelligence-agent-surfaces.md`, completed locally
on branch `codex/project-intelligence-agent-surfaces` with independent review.

Earlier Project Intelligence successors completed in this program:
`docs/goals/assura-code-symbol-enrichment.md`, completed locally on branch
`codex/code-symbol-enrichment` with independent review and archived Trellis
task
`.trellis/tasks/archive/2026-06/06-28-06-28-code-symbol-enrichment`.
`docs/goals/assura-local-semantic-search.md`, completed locally on branch
`codex/local-semantic-search` with independent review and archived Trellis task
`.trellis/tasks/archive/2026-06/06-28-local-semantic-search`.
`docs/goals/assura-content-query-and-search-cli.md`, completed locally on
branch `codex/content-query-and-search-cli` with independent review and
archived Trellis task
`.trellis/tasks/archive/2026-06/06-28-06-28-content-query-and-search-cli`.
`docs/goals/assura-embedded-graph-search-store-spike.md`, completed locally on
branch `codex/embedded-graph-search-store-spike` with independent review and
archived Trellis task
`.trellis/tasks/archive/2026-06/06-28-embedded-graph-search-store-spike`.
`docs/goals/assura-project-intelligence-fact-model.md`, completed locally on
branch `codex/project-intelligence-fact-model` with independent review and
archived Trellis task
`.trellis/tasks/archive/2026-06/06-28-project-intelligence-fact-model`.
`docs/goals/assura-documentation-ia-project-intelligence.md`, completed locally
on branch `codex/documentation-ia-project-intelligence` with independent review
and archived Trellis task
`.trellis/tasks/archive/2026-06/06-28-documentation-ia-project-intelligence`.
`docs/goals/assura-content-model-source-of-truth.md` completed on branch
`codex/content-model-source-of-truth`;
`docs/goals/assura-rust-markdown-validation-and-fixing.md` completed on branch
`codex/rust-markdown-validation-and-fixing`.

Most recent completed major roadmap work: Support Matrix Surface Expansion.
Policy Depth Iteration 02 goals 09 through 13 are complete and archived,
ending with Goal 13 PR #55 and archive PR #56.
Release Contract Rules first slice is complete via PR #59 and archive PR #60.
Public Surface Matrix first slice is complete via PR #61 and archive PR #62.
Cargo Manifest Semantics first slice is complete via PR #64 and archive PR
#65.
Test Relationship Rule first slice is complete via PR #68 and archive PR #69.
Module Topology Rule first slice is complete via PR #72 and archive PR #73.
Docs Lifecycle Rule first slice is complete via PR #78 and archive PR #79.
Docs Lifecycle Coverage first dogfood expansion is complete via PR #83.
Support Matrix Surface Expansion is complete via PR #88 and archive/sync PR
#89.
This completion does not mark the broader Assura roadmap complete.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

Planned next roadmap candidate:
Execute Project Intelligence Usability, starting with
`docs/goals/assura-project-intelligence-adoption-blueprint.md`. The iteration
turns the completed runtime foundation into a usable product workflow through
adoption, real-repo proof, persistent sessions, editor/agent transports,
safe-fix workflow, and release hardening.

## Recommended Next Action

Run the workflow gate, then validate and start
`docs/goals/assura-project-intelligence-adoption-blueprint.md`. Do not reopen
the completed Project Intelligence Runtime successors, support-matrix
expansion, or Windows CI Restore work unless a new concrete docs, manifest,
package, binary, support status, or hosted CI drift case is named with
executable evidence. The broader product roadmap remains open until a separate
product decision declares it complete.

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
