---
id: phase-assura-roadmap-iteration-01-agentic-adoption-foundation
type: phase
title: Assura roadmap iteration 01 agentic adoption foundation
status: active
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - docs/goals/assura-goal-01-trustworthy-self-enforcement.md
  - docs/goals/assura-goal-02-policy-language-completeness.md
  - docs/goals/assura-goal-03-agent-feedback-delivery-loop.md
  - docs/goals/assura-goal-04-fast-incremental-check-engine.md
  - docs/goals/assura-goal-05-installable-adoption-path.md
  - docs/goals/assura-goal-06-review-evidence-and-quality-gates.md
  - docs/goals/assura-goal-07-extension-and-plugin-foundation.md
  - docs/goals/assura-goal-08-release-readiness-and-ecosystem.md
---

# Assura Roadmap Iteration 01: Agentic Adoption Foundation

## Objective

Define the first execution iteration of the broader Assura roadmap as a sequence
of reviewable, measurable, two-week team goals. This phase document is the
routing ledger for Iteration 01, not an executable goal and not the whole product
roadmap. Each linked goal is intended to be a large but bounded chunk that a
small engineering team could complete in about two focused weeks, including
implementation, validation, documentation, and review closure.

Iteration 01 should move Assura from a structure-first validation CLI with stable
agent feedback into a trustworthy, self-enforcing adoption foundation. Later
roadmap iterations should build on this foundation without treating Iteration 01
completion as completion of the full Assura roadmap.

## Roadmap Hierarchy

Use this document at the phase level:

- Product roadmap: the long-lived direction tracked in
  `.trellis/spec/assura/roadmap.md` and future iteration docs.
- Roadmap iteration: this bounded phase for agentic adoption foundation work.
- Execution goals: the eight linked two-week chunks below.

When all eight linked goals close, mark Iteration 01 complete and create or
point to the next roadmap iteration. Do not mark the broader Assura roadmap
complete because this iteration finishes.

## Current Product Baseline

Assura currently supports:

- `assura check` for structure-first repository validation.
- `assura init` for starter `.assura/config.yml` creation.
- `assura migrate` for supported LS-Lint configuration migration.
- `assura status --format json` for project/config/rule summaries.
- `assura check --format agent` for stable agent feedback JSON.
- `assura check --format agent --agent codex` as an optional Codex
  `UserPromptSubmit` delivery adapter.
- Local docs, website pages, Trellis tasks, and checked analysis artifacts for
  recent real-project and LS-Lint performance proofs.

Iteration 01 deliberately does not reintroduce package feedback CLIs,
per-agent CLI entrypoints, or one `--format <agent>-hook` value per agent.

## Iteration 01 Execution Goal Sequence

| Order | Goal | Primary Outcome | Depends On | Review Gate |
| --- | --- | --- | --- | --- |
| 1 | [Trustworthy Self-Enforcement](./assura-goal-01-trustworthy-self-enforcement.md) | Assura reliably validates its own repository and closes stale workflow state. | Current Phase 01 baseline | Self-check and Trellis closure review |
| 2 | [Policy Language Completeness](./assura-goal-02-policy-language-completeness.md) | Structure policy language covers real repo contracts with documented LS-Lint boundaries. | Goal 1 | Policy fixture and migration review |
| 3 | [Agent Feedback Delivery Loop](./assura-goal-03-agent-feedback-delivery-loop.md) | Stable feedback output becomes a complete local agent loop with measured usefulness. | Goals 1-2 | Agent output and Codex delivery review |
| 4 | [Fast Incremental Check Engine](./assura-goal-04-fast-incremental-check-engine.md) | Repeated agent checks are fast, scoped, deterministic, and backed by evidence. | Goals 1-3 | Performance and correctness review |
| 5 | [Installable Adoption Path](./assura-goal-05-installable-adoption-path.md) | New users can install, initialize, run, and understand Assura without source checkout assumptions. | Goals 1-4 | Fresh-machine adoption review |
| 6 | [Review Evidence And Quality Gates](./assura-goal-06-review-evidence-and-quality-gates.md) | PRs, goals, and reports share reproducible evidence and strict completion gates. | Goals 1-5 | Evidence reproduction review |
| 7 | [Extension And Plugin Foundation](./assura-goal-07-extension-and-plugin-foundation.md) | Assura can add constrained custom rules without fragmenting the core CLI. | Goals 1-6 | API, safety, and fixture review |
| 8 | [Release Readiness And Ecosystem](./assura-goal-08-release-readiness-and-ecosystem.md) | Assura is ready for a public pre-1.0 release with support, compatibility, and roadmap clarity. | Goals 1-7 | Release candidate review |

## Review Task Model

Every Iteration 01 execution goal must close these review tasks before it is
marked complete. Individual goal files specialize the tasks with concrete
evidence.

### R0. Scope And Source-Of-Truth Review

- Confirm the goal file is the current entrypoint.
- Confirm `.trellis/spec/assura/roadmap.md` points to the owning task or phase
  goal when relevant.
- Confirm older docs, historical task notes, or superseded branch history do not
  contradict the current command surface.
- Block if the work depends on conversation-only context or an untracked local
  file.

### R1. Design And Contract Review

- Review public CLI signatures, config fields, output schemas, and docs claims.
- Confirm new behavior has a documented good case, base case, bad case, and
  error matrix.
- Confirm the design does not create parallel command surfaces for the same
  user problem.
- Block if any feature depends on unsupported daemon, graph, hosted telemetry,
  automatic Codex approval, or per-agent CLI behavior unless the goal explicitly
  changes the product contract first.

### R2. Implementation Review

- Inspect the diff for duplicate logic, compatibility shims, stale helpers, and
  unclear ownership boundaries.
- Confirm public items have appropriate rustdoc or equivalent docs.
- Confirm tests exercise both passing and failing paths.
- Block if implementation adds a source-of-truth layer next to Trellis,
  `.assura/config.yml`, or the documented CLI without explaining why.

### R3. Evidence Reproduction Review

- Re-run the goal's validation commands from a clean checkout.
- Reproduce checked reports, fixtures, generated examples, or benchmark outputs.
- Verify checked artifacts are generated by documented commands, not hand
  assembled.
- Block if evidence requires local state, stale paths, or commands that do not
  appear in the PR.

### R4. User Journey Review

- Read website/docs from the viewpoint of a first-time user.
- Confirm commands are copy/paste-ready and current.
- Confirm limitations are concrete, not vague roadmap disclaimers.
- Block if docs imply unavailable behavior such as dependency graph validation,
  hosted telemetry, automatic repair, or hidden agent setup.

### R5. PR And Completion Review

- PR description links the goal file, review evidence, and validation commands.
- CI passes or every blocker is documented with owner and next action.
- Review-agent findings are addressed or explicitly rejected with rationale.
- Gemini or other code review comments are inspected and resolved when they are
  actionable.
- The final handoff includes the PR URL, next goal path, and any known risk.

## Execution Goal And Iteration Completion Definition

An execution goal in Iteration 01 is complete only when:

- the goal's definition of done is satisfied;
- required validation commands pass;
- review tasks R0 through R5 are complete;
- docs and checked artifacts are reproducible from committed files;
- the PR is open with review evidence linked;
- all actionable review feedback is addressed; and
- the next goal in this iteration sequence is ready to start.

Iteration 01 is complete only when all eight linked goals are complete, their
evidence is linked in the ledger below, and the next roadmap iteration is
created or identified. Iteration 01 completion is not product-roadmap completion.

## Progress And Evidence Ledger

Future PRs should add one progress-log entry to the active execution goal and
one summary entry here when each goal completes.

| Goal | Status | Required Completion Evidence | Next Action |
| --- | --- | --- | --- |
| 1. Trustworthy Self-Enforcement | Completed | PR #18 merged with Assura self-check report, archived stale Trellis task diff, root/config policy fixture updates, and review notes proving stale workflow state cannot remain active unnoticed. | Complete; continue with Goal 2. |
| 2. Policy Language Completeness | Active | Supported policy matrix, unsupported LS-Lint boundary table, migration fixture corpus, generated docs examples, and passing CLI/config tests for good, base, and bad cases. | Execute and verify `docs/goals/assura-goal-02-policy-language-completeness.md`. |
| 3. Agent Feedback Delivery Loop | Planned | Ten-violation same-turn feedback proof, generic agent JSON schema fixture, Codex `UserPromptSubmit` fixture under 24 KiB, deterministic rerun diff, and fixed-before-new-turn counts. | Start after Goal 2 establishes policy fixtures. |
| 4. Fast Incremental Check Engine | Planned | Pre/post performance report with 30-run p95 rows, hardware metadata, cold CLI regression comparison, warm p95 <= 250 ms, changed-path p95 <= 100 ms, and deterministic output proof. | Start after Goal 3 defines feedback loop latency needs. |
| 5. Installable Adoption Path | Planned | Release-style artifact smoke evidence for Ubuntu x86_64, macOS arm64, macOS x86_64, and Windows x86_64, plus first-run docs proving install, init, status, passing check, and failing check. | Start after Goal 4 proves local check performance. |
| 6. Review Evidence And Quality Gates | Planned | Shared PR evidence template, goal completion checklist, reproduction script or command index, review-agent/Gemini closure process, and at least two backfilled example PR evidence records. | Start after install/docs flows stabilize. |
| 7. Extension And Plugin Foundation | Planned | Public extension contract, safety model, fixture plugins, failure-mode tests, API docs, and review notes proving extensions cannot bypass core validation or fragment the CLI. | Start after review gates can protect extension behavior. |
| 8. Release Readiness And Ecosystem | Planned | Release candidate checklist, version/support policy, changelog, migration/adoption docs, package artifact proof, website release page, and post-release issue triage plan. | Start after extension boundaries and evidence gates are proven. |

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-01 | Started Iteration 01 execution from updated `master` on `codex/roadmap-phase-01-execution`; created `.trellis/tasks/06-01-roadmap-phase-01-execution`; archived the merged roadmap-publication task and stale completed/merged Trellis tasks; confirmed self-check baseline is clean. | `git switch -c codex/roadmap-phase-01-execution`; `python3 ./.trellis/scripts/task.py list` shows one active task; `cargo run --quiet -- check --format json .` reports zero violations. |
| 2026-06-01 | Completed Goal 01 via merged PR #18 and moved Iteration 01 execution to Goal 02 on `codex/phase-01-goal-02-policy-language`; clarified this document as an iteration ledger rather than an executable roadmap goal. | `gh pr view 18 --json state,mergedAt,url`; `git status --short --branch`; `.trellis/tasks/06-01-roadmap-phase-01-execution/task.json` branch field. |

## Handoff Prompt

```text
/goal docs/goals/assura-goal-02-policy-language-completeness.md
```
