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
| 23 | Post-Beta Capabilities | Completed | `v0.3.0` beta increment published and live-verified with self-config hardening, supported document graph, true daemon mode, Markdown engine/safe fixes, performance floor, agent installers, VS Code support, extension API clarity, LS-Lint reassessment, and release hardening |
| 24 | Performance Polish | Planned / Separate Lane | Planning task `.trellis/tasks/07-02-native-performance-roadmap-and-goal-text`; kickoff goal `docs/goals/assura-performance-polish-program.md` |
| 25 | Agent-Ready Project Onboarding | Completed | Parent program and all twelve child goals completed locally on `codex/agent-ready-onboarding-backlog`; final audit in `docs/analysis/2026-07-03-agent-ready-onboarding-final-audit.md` |

## Active Roadmap Iteration

Agent-Ready Project Onboarding completed the adoption iteration after the
`v0.3.0` beta increment. It made Assura the default scaffold, doctor, and
feedback loop for agent-ready repositories, starting with a broad first-run
baseline and then asking only the specialization questions needed to avoid
invented project conventions.

Completion audit:
`docs/analysis/2026-07-03-agent-ready-onboarding-final-audit.md`.

Current branch:
`codex/agent-ready-onboarding-backlog`.

Current recommended goal:
`docs/goals/assura-post-onboarding-backlog-execution-sequence.md`.

Performance lane:
`docs/goals/assura-performance-polish-program.md`.

Completed support-hardening details from the prior beta increment remain
routed through `docs/goals/assura-post-beta-support-release-hardening.md`.

Public roadmap artifact:
`docs/data/public-roadmap.json`.

Triggering evidence:
A new-project dogfood run exposed a broad adoption gap: Assura can make a
configured policy pass while the repository still does not feel scaffolded,
discoverable, queryable, or self-explaining to a coding agent. That backlog is
not "more intelligence" in the abstract; the product milestone is to make
Assura the default scaffold, doctor, and feedback loop for agent-ready
repositories.

North-star outcome:
A coding agent can enter a new project and reliably answer: what rules apply
here, what guidance should I follow, which skills exist, what project facts are
modeled, what references are broken, what is checked versus unchecked, what
should I fix next, and whether the feedback is a nudge, warning, or merge gate.
The detailed backlog lives in
`docs/goals/assura-agent-ready-project-onboarding-program.md`.

Performance polish remains a separate planned lane. Its detailed kickoff
criteria live in `docs/goals/assura-performance-polish-program.md`.

Direction lock, clarified on 2026-05-31: do not create or revive
`assura-codex-feedback`, do not add one CLI entrypoint per agent, and do not add
one `--format <agent>-hook` value per agent. Treat older roadmap/task wording in
that direction as superseded by `.trellis/spec/assura/codex-agent-feedback.md`.

## Recommended Next Action

Agent-ready onboarding is complete for this increment. First execute
`docs/goals/assura-post-onboarding-backlog-execution-sequence.md` to publish
and revalidate the local PR #139 follow-up commits, then continue into
`docs/goals/assura-performance-polish-program.md` for native performance
evidence, no-slower gates, and CLI-floor optimization.

Priority rule: performance work remains separate from the completed
agent-ready onboarding program. Reopen onboarding only for follow-up product
decisions, regressions, or post-merge review findings.

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
