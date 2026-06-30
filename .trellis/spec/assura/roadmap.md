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
| 20 | Project Intelligence Simple Usability | Active | New follow-up from live docs review: simple CLI, repo-wide code/content search, and content-model validation demo |

## Active Roadmap Iteration

Project Intelligence Simple Usability is active.

Owning task:
`.trellis/tasks/06-30-project-intelligence-simple-repo-usability`.

Current branch:
`codex/project-intelligence-usability-followup`.

Current recommended goal:
`docs/goals/assura-project-intelligence-simple-cli.md`, with companion goal
docs for repo-wide code/content search and content-model validation demos.

Triggering evidence:
After the Project Intelligence docs were deployed, live review found that the
demo remains too meta, keyword search did not expose a score, graph expansion
was not explained from a direct anchor, and the current command set still
requires users to understand too many primitives before getting a useful
repo-wide answer.

Project Intelligence Usability is complete locally as the lower-level
primitive and release-hardening slice.

Most recent owning task:
`.trellis/tasks/archive/2026-06/06-29-project-intelligence-release-hardening`,
completing
`docs/goals/assura-project-intelligence-release-hardening.md`.

Current program:
`docs/goals/assura-project-intelligence-usability-program.md`.

Project Intelligence Runtime is complete and should remain closed unless a new
concrete drift case is named with evidence.

Most recent completed Project Intelligence successor:
`docs/goals/assura-project-intelligence-release-hardening.md`, completed
locally on branch `codex/project-intelligence-agent-surfaces` with independent
review and clean-source target-state evidence.

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
Complete the Simple Usability follow-up before creating another Project
Intelligence successor. The follow-up should keep CLI-first local workflows as
the default product path; MCP or hosted adapters are optional later surfaces
over the same contracts.

## Recommended Next Action

Run the workflow gate, then continue
`.trellis/tasks/06-30-project-intelligence-simple-repo-usability`. The
immediate path is to land the docs/CLI score correction and keep the larger
repo-wide search and validation-demo work in the new goal docs. Do not reopen
completed Project Intelligence Runtime or lower-level Usability successors
unless new evidence names a narrower drift case.

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
