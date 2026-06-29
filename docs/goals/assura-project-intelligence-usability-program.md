---
id: goal-assura-project-intelligence-usability-program
type: goal
title: Assura project intelligence usability program
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-runtime-program.md
  - docs/goals/assura-project-intelligence-adoption-blueprint.md
  - docs/goals/assura-project-intelligence-real-repo-proof.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-project-intelligence-persistent-session.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - docs/goals/assura-project-intelligence-editor-agent-transports.md
  - docs/goals/assura-project-intelligence-release-hardening.md
---

# Assura Project Intelligence Usability Program

## Objective

Move the completed Project Intelligence Runtime from a proven local foundation
to a usable product workflow that a maintainer, coding agent, or editor
integration can adopt without knowing Assura internals.

The runtime now has typed content models, Markdown validation and safe-fix
contracts, graph/query/search facts, optional semantic candidates, optional
code-symbol enrichment, and shared agent envelopes. The next product gap is not
more raw capability; it is making those capabilities discoverable, fast,
operable, and trustworthy in realistic workflows.

## Current Gap

Live repo evidence shows these remaining usability gaps:

- project-intelligence setup is possible through config and docs, but there is
  no starter template or guided first-run path from a normal repo to a first
  useful query;
- project-intelligence facts are queryable, but useful editing context still
  requires callers to stitch together check, search, expand,
  missing-relations, agent-query, and safe-fix preview commands;
- repeated agent/editor workflows still pay cold CLI costs unless lower-level
  prepared checks are wired into a public session surface;
- daemon/editor, LSP, and MCP surfaces are planned, not supported product
  contracts;
- safe fixes have a dry-run schema, but not a complete preview, apply,
  rollback, and audit workflow;
- release docs and support matrices do not yet treat project-intelligence
  usability as a promoted release slice.

## Execution Sequence

Execute these goals in order unless a goal records a refreshed dependency
decision with evidence.

1. [Project Intelligence Adoption Blueprint](./assura-project-intelligence-adoption-blueprint.md)
   creates the first-run path from a normal repository to modeled content,
   Markdown checks, graph/search queries, and agent envelopes.
2. [Project Intelligence Real Repo Proof](./assura-project-intelligence-real-repo-proof.md)
   proves the blueprint on Assura plus at least one realistic non-Assura repo
   fixture or pinned project package.
3. [Project Intelligence Onboarding Template](./assura-project-intelligence-onboarding-template.md)
   makes first-run setup reproducible without hand-authoring the initial
   schema, collections, sample records, and commands.
4. [Project Intelligence Context Pack](./assura-project-intelligence-context-pack.md)
   provides one bounded context contract for diagnostics, graph/search context,
   relations, and safe-fix preview metadata.
5. [Project Intelligence Persistent Session](./assura-project-intelligence-persistent-session.md)
   promotes a measured warm session or watch-backed workflow for repeated
   checks and project-intelligence queries.
6. [Project Intelligence Safe Fix Workflow](./assura-project-intelligence-safe-fix-workflow.md)
   turns safe-fix dry-runs into a complete bounded repair workflow.
7. [Project Intelligence Editor And Agent Transports](./assura-project-intelligence-editor-agent-transports.md)
   adds concrete editor and agent transports over the shared contracts without
   forking behavior.
8. [Project Intelligence Release Hardening](./assura-project-intelligence-release-hardening.md)
   locks schemas, docs, support status, compatibility notes, and release
   evidence for the usable product slice.

## Program Definition Of Done

- A new user can follow one documented path from install/init to first useful
  content query and agent envelope.
- A starter template or first-run profile creates a working
  project-intelligence model without hand-authored boilerplate.
- At least one realistic non-Assura project package proves the adoption path
  without relying on untracked local state.
- A bounded context-pack workflow gives agents and humans one task-focused
  packet with diagnostics, related facts, relation status, and safe-fix
  preview metadata.
- Repeated local workflows have a measured warm-session path with explicit
  invalidation rules and fallback to full checks.
- Editor and agent integrations call the same validation/query/safe-fix
  contracts as the CLI.
- Safe fixes support preview, bounded apply, machine-readable audit output, and
  a clear no-automatic-repair policy.
- Project-intelligence schemas and support levels are documented in release
  and support surfaces.
- Every goal has independent review evidence and current validation commands.

## Non-Goals

- No hosted service requirement.
- No plugin marketplace.
- No remote semantic or code-intelligence provider requirement for core use.
- No per-agent CLI families or per-agent output formats.
- No claim that semantic candidates decide validation correctness.
- No automatic repair without explicit user or integration approval.

## Validation Commands

Planning-only updates to this program should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

Implementation goals add their own narrow proof gates. The final program gate
must include each completed successor's validation chain.

## Review Tasks

- R1: Confirm the program addresses usability gaps rather than reopening the
  completed runtime foundation.
- R2: Confirm the sequence does not promote daemon, LSP, MCP, watch, or
  safe-fix apply behavior before tests and docs exist.
- R3: Confirm every goal is executable independently and has a clear exit
  condition.
- R4: Confirm no goal requires hosted infrastructure or external providers for
  normal local use.

## Reviewer Blocking Criteria

Block if the plan claims unsupported surfaces as supported, requires a remote
service for core functionality, introduces per-agent command families, treats
semantic search as validation truth, or marks the usability program complete
without real-repo proof and release hardening.

## Progress Log

- 2026-06-29: Began execution with the first successor,
  `docs/goals/assura-project-intelligence-adoption-blueprint.md`, on task
  `.trellis/tasks/06-29-project-intelligence-usability-execution`. Initial
  progress adds a visual documentation demo and reproducible command examples
  for the current local project-intelligence surfaces. The broader program
  remains open until real-repo proof, persistent sessions, editor/agent
  transports, safe-fix workflow, and release hardening are implemented and
  audited.
- 2026-06-29: Continued with
  `docs/goals/assura-project-intelligence-real-repo-proof.md` on task
  `.trellis/tasks/06-29-project-intelligence-real-repo-proof`. Added the
  Beacon CRM non-Assura fixture package plus tests and analysis evidence for
  valid checks, invalid diagnostics, content search, graph expansion,
  missing-relations, agent-query envelopes, and materialized safe-fix preview.
  Review blockers were fixed by adding Assura-local goal content modeling and
  tightening diagnostics assertions. This completes the real-repo proof
  successor; the program remains open for persistent sessions, editor/agent
  transports, safe-fix workflow, and release hardening.
- 2026-06-29: Re-evaluated the remaining usability gap after the adoption
  demo and real-repo proof. Added
  `docs/analysis/2026-06-29-project-intelligence-usability-gap-evaluation.md`
  and split the next work into a tighter ordered set:
  `docs/goals/assura-project-intelligence-onboarding-template.md`,
  `docs/goals/assura-project-intelligence-context-pack.md`,
  persistent session, safe-fix workflow, editor/agent transports, and release
  hardening. The immediate next goal is onboarding template.
