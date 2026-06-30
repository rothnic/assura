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
| 21 | Markdown Reference Intelligence | Active | Child program under the beta workstream for fast Markdown linting, repository reference graph, daemon readiness, daemon management CLI, VS Code integration, agent daemon awareness, incremental pre-1.0 releases, public roadmap artifact, and future Zed/JetBrains editor follow-ups |
| 22 | Beta Code-Agnostic Capabilities | Active | Master beta program covering roadmap/release train, structure severity, content collections, Markdown quality, reference graph, daemon core, daemon CLI, agent nudges, VS Code, and the LS-Lint no-slower performance gate |

## Active Roadmap Iteration

Beta Code-Agnostic Capabilities is active.

Owning task:
`.trellis/tasks/06-30-markdown-lint-link-reference-engine`.

Current branch:
`codex/markdown-reference-master-goal`.

Current recommended goal:
`docs/goals/assura-beta-code-agnostic-capabilities-program.md`.

Public roadmap artifact:
`docs/data/public-roadmap.json`.

Triggering evidence:
After defining the Markdown Reference Intelligence parent program, live review
clarified that the real target is a beta release across ten large epics:
public roadmap and release train, structure validation severity messaging,
frontmatter and collection modeling/querying, high-performance Markdown
linting and heading validation, code/doc reference validation, daemon mode,
daemon management CLI, concise agent nudges for Codex/OpenCode/Claude/Pi
agents, VS Code integration, and a hard LS-Lint no-slower performance gate.

The current checked performance artifact shows cold `assura-cli` no slower
than native LS-Lint on 8 of 8 realistic-equivalent fixtures, with the strict
2x claim not complete, while warm session evidence remains complete. The beta
performance gate blocks any merge where a headline LS-Lint-equivalent fixture
is slower than native LS-Lint.

Prior triggering evidence:
After the Project Intelligence docs follow-up merged and deployed, live review
clarified a separate Markdown and repository-reference quality requirement:
Assura should provide a hyper-fast local Markdown linter that validates
required headings, optionally adds missing headings, enforces GitHub-renderable
relative internal links, discovers code/comment references to docs, detects
broken file/code/heading/line references, supports configurable warning levels,
allows reasoned suppressions, and can later power context-efficient daemon or
warm-session feedback from the same reference graph.

The beta program tracks the required child goals. The Markdown Reference
Intelligence program remains a child workstream for the reference engine,
daemon readiness, daemon management CLI, VS Code integration, agent daemon
awareness, incremental pre-1.0 releases, public website roadmap, and later
Zed/JetBrains integrations.

Most recent completed iteration:
Project Intelligence Simple Usability completed in PR #109. It added
deterministic scores to `assura content search`, updated the Project
Intelligence demo with concrete graph expansion and validation examples, and
created companion goals for simple CLI, repo-wide code/content search, and
content-model validation demos.

Prior triggering evidence:
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
Execute the beta program one epic at a time from
`docs/goals/assura-beta-code-agnostic-capabilities-program.md`. The first
incomplete epic is the public roadmap and release-train source of truth. Keep
CLI-first local workflows as the default product path; MCP or hosted adapters
are optional later surfaces over the same contracts.

## Recommended Next Action

Run the workflow gate, then continue
`.trellis/tasks/06-30-markdown-lint-link-reference-engine`. The immediate path
is to execute the beta master goal in
`docs/goals/assura-beta-code-agnostic-capabilities-program.md`, starting with
the first incomplete epic in its Ten Major Iterations table. Do not reopen
completed Project Intelligence Runtime or lower-level Usability successors
unless new evidence names a narrower drift case.

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
