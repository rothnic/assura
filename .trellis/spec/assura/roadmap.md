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
| 19 | Project Intelligence Usability | Completed | Adoption blueprint, real-repo proof, onboarding template, context pack, persistent session, safe-fix workflow, `.assura/` directory organization, agent CLI surface, LSP editor transport, release hardening, and final usability audit completed locally |
| 20 | Project Intelligence Simple Usability | Completed | PR #109 merged and deployed; task archived under `.trellis/tasks/archive/2026-06/` |
| 21 | Markdown Reference Intelligence | Completed | Completed for beta in PR #112; post-beta Markdown, graph, daemon, agent, and editor hardening now routes through Post-Beta Capabilities |
| 22 | Beta Code-Agnostic Capabilities | Completed | PR #112 merged; `v0.2.0` tag and GitHub release published with live release evidence |
| 23 | Post-Beta Capabilities | Active | Parent goal `docs/goals/assura-post-beta-capabilities-program.md` owns self-config hardening, supported document graph, true daemon mode, Markdown engine, performance floor, agent installers, VS Code support, extension API clarity, LS-Lint reassessment, and release hardening |

## Active Roadmap Iteration

Post-Beta Capabilities is active after release `v0.2.0`.

Owning task:
the next child task from `docs/goals/assura-post-beta-capabilities-program.md`.

Current branch:
`codex/markdown-engine-selection-record` for closing the Markdown engine
selection and handing off to the next post-beta child goal.

Current recommended goal:
`docs/goals/assura-post-beta-capabilities-program.md`.

Public roadmap artifact:
`docs/data/public-roadmap.json`.

Triggering evidence:
After the beta release, live review identified the next large iteration:
refine Assura's own config and docs variance, make document graph support
fully supported for content validation/search/query, implement a true daemon
process, adopt the fastest practical Rust markdownlint-compatible engine,
enforce a no-slower-than-LS-Lint fixture floor, install Codex/OpenCode/Claude/Pi
agent integrations, support the VS Code extension path, clarify extension API
boundaries, reassess LS-Lint parity after the new surfaces land, and harden
support/release evidence.

North-star outcome:
A documentation-heavy project can use one local Assura workflow to validate
structure first, then Markdown, content models, repository references, graph
queries, daemon-backed warm state, compact agent nudges, VS Code diagnostics,
safe fixes, and performance gates without relying on unsupported services or
private integration logic. The parent goal owns the detailed verification
story: a maintainer renames architecture docs and moves code, then proves the
CLI, daemon, agent hooks, editor, content graph, Markdown fixes, and LS-Lint
performance gate all agree before merge.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

## Recommended Next Action

Execute `docs/goals/assura-post-beta-capabilities-program.md` goal by goal.
True daemon mode is complete in PR #117. The Markdown engine child is closed
for this beta increment: Assura's native Markdown validation and safe-fix path
is the supported default, `rumdl` remains an explicit opt-in compatibility
adapter, and `mdlint` is rejected as a supported fixer until its safety
failures are resolved. Continue with the next incomplete child goal from the
parent program, preferring document-graph or self-dogfood hardening only if
live goal state shows remaining gaps.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep public roadmap item labels at 2-4 words.
- Keep internal epic names short enough to scan.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new roadmap iteration or epic is needed, add it here and identify the
  first Trellis task that owns it.
- If an iteration or epic is active, say which task owns it and what the next
  recommended action is.
- Completing an iteration closes only that iteration; the roadmap remains open
  until a separate product decision declares the full roadmap complete.
