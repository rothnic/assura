---
id: goal-assura-project-intelligence-usability-program
type: goal
title: Assura project intelligence usability program
status: completed
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
  - docs/goals/assura-project-intelligence-assura-directory-organization.md
  - docs/goals/assura-project-intelligence-agent-cli-surface.md
  - docs/goals/assura-project-intelligence-lsp-editor-transport.md
  - docs/goals/assura-project-intelligence-release-hardening.md
  - docs/goals/assura-project-intelligence-simple-cli.md
  - docs/goals/assura-project-intelligence-repo-wide-code-content-search.md
  - docs/goals/assura-project-intelligence-content-model-validation-demo.md
---

# Assura Project Intelligence Usability Program

## Objective

Move the completed Project Intelligence Runtime from a proven local foundation
to a usable product workflow that a maintainer, coding agent, or editor
integration can adopt without knowing Assura internals.

The completed program added typed content models, Markdown validation and
safe-fix contracts, graph/query/search facts, optional semantic candidates,
optional code-symbol enrichment, and shared agent, session, and editor
surfaces. The lower-level local usability slice is complete; the separate
follow-up is to make the common path simpler and less primitive-facing.

## Current Gap

Live repo evidence shows that the original usability gaps are now covered by
completed successors: onboarding, real-repo proof, context packs, warm
sessions, safe-fix apply/audit behavior, `.assura/` model organization, local
agent commands, local editor diagnostics/code actions, and release hardening.

The new gap discovered after publishing the docs is product clarity rather
than primitive capability. The current commands work, but a new user still has
to understand too many surfaces before getting a useful repo-wide answer. The
follow-up goals linked in this document are intentionally outside the completed
program's definition of done:

- a simple CLI command for common project-intelligence search;
- repo-wide code and content search with bidirectional traversal;
- a direct content-model validation demo that shows frontmatter schema drift,
  relation drift, and Markdown linting in one short workflow.

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
7. [Project Intelligence Assura Directory Organization](./assura-project-intelligence-assura-directory-organization.md)
   keeps `.assura/` scalable by moving model artifacts under one organized
   model directory with optional hierarchy.
8. [Project Intelligence Agent CLI Surface](./assura-project-intelligence-agent-cli-surface.md)
   adds local agent commands over the shared context, query, diagnostics, and
   safe-fix preview contracts. MCP remains an optional future adapter over
   those contracts, not a prerequisite for this program slice.
9. [Project Intelligence LSP Editor Transport](./assura-project-intelligence-lsp-editor-transport.md)
   adds local editor diagnostics, context, and code-action behavior over the
   same contracts.
10. [Project Intelligence Release Hardening](./assura-project-intelligence-release-hardening.md)
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
- Agent and editor integrations call the same validation/query/safe-fix
  contracts as the CLI through independently validated local surfaces.
- Safe fixes support preview, bounded apply, machine-readable audit output, and
  a clear no-automatic-repair policy.
- `.assura/` keeps well-known root files bounded and stores project-intelligence
  model artifacts under a documented model directory, with user-defined
  hierarchy allowed inside that directory.
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
  safe-fix apply behavior before tests and docs exist, and does not require
  MCP for local agent usability.
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
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-onboarding-template.md` locally on
  task `.trellis/tasks/06-29-project-intelligence-onboarding-template`. Added
  `assura init --project-intelligence`, deterministic starter schema/records,
  broken-state example diagnostics, website/API docs, focused integration
  coverage, and command-surface policy updates. The immediate next goal is
  `docs/goals/assura-project-intelligence-context-pack.md`.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-context-pack.md` locally on task
  `.trellis/tasks/06-29-project-intelligence-context-pack`. Added
  `assura content context-pack` as a bounded, read-only handoff packet for
  diagnostics, related modeled records, Markdown sections, missing relation
  status, keyword search, and safe-fix preview metadata. The immediate next
  goal is `docs/goals/assura-project-intelligence-persistent-session.md`.
- 2026-06-29: Re-evaluated the remaining usability gap after onboarding and
  context-pack completion on task
  `.trellis/tasks/06-29-project-intelligence-usability-remaining-goals`.
  Confirmed the remaining work is persistent warm reuse, safe-fix apply/audit,
  agent CLI surface, LSP editor transport, and release hardening. Superseded
  the broader combined editor/agent transport goal with narrower local
  surfaces.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-persistent-session.md` locally on
  task `.trellis/tasks/06-29-project-intelligence-persistent-session`. Added
  `assura content session` as a local JSON-line session for repeated
  project-intelligence diagnostics, context packs, graph/search queries,
  missing relations, and safe-fix previews, with conservative reload metadata
  and benchmark evidence. The immediate next goal is
  `docs/goals/assura-project-intelligence-safe-fix-workflow.md`.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-safe-fix-workflow.md` locally on
  task `.trellis/tasks/06-29-06-29-project-intelligence-safe-fix-workflow`.
  Safe fixes now preview by default, require `--apply` before writing, emit
  machine-readable per-file/per-fix audit output, report partial write
  failures in the same JSON report, and expose `audit_id` correlation through
  content-query, context-pack, and session safe-fix previews. The next usability
  gap is `.assura/` organization: root-level project-intelligence model/schema
  artifacts should move under a shared model directory before local
  agent/editor surfaces promote the starter layout further.
- 2026-06-30: After publishing the completed usability slice, reviewed the
  live docs and CLI from a first-user perspective. The shipped primitives work,
  but the guide is still too meta and the common path requires too much command
  knowledge. Added the follow-up goal set:
  `docs/goals/assura-project-intelligence-simple-cli.md`,
  `docs/goals/assura-project-intelligence-repo-wide-code-content-search.md`,
  and
  `docs/goals/assura-project-intelligence-content-model-validation-demo.md`.
  These goals keep the CLI as the primary local surface; MCP remains optional
  future adaptation over stable local contracts, not the product path.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-assura-directory-organization.md`
  locally on task
  `.trellis/tasks/06-29-project-intelligence-assura-directory-organization`.
  Model artifacts under `.assura/` must now live below `.assura/models/**`;
  Assura's own project-intelligence schema moved to `.assura/models/goals/`;
  the onboarding starter writes
  `.assura/models/project-intelligence/starter.schema.json`; docs, support
  policy, release notes, and website examples show the organized layout. The
  immediate next goal is
  `docs/goals/assura-project-intelligence-agent-cli-surface.md`.
- 2026-06-29: Revalidated the former MCP transport successor after user
  feedback on task
  `.trellis/tasks/06-29-project-intelligence-agent-cli-surface`. The next
  usability step is now CLI-first: add a local `assura agent ...` surface over
  existing project-intelligence contracts. MCP is explicitly optional future
  adapter work, with no remote access requirement for this program slice.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-agent-cli-surface.md` locally on task
  `.trellis/tasks/06-29-project-intelligence-agent-cli-surface`. Added
  `assura agent` as a supported local project-intelligence command group with
  JSON-default context, diagnostics, context-pack, search/show/expand,
  missing-relations, safe-fix preview, and session commands. The surface
  delegates to existing content-query contracts and does not implement or
  require MCP or remote access. The immediate next goal is
  `docs/goals/assura-project-intelligence-lsp-editor-transport.md`.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-lsp-editor-transport.md` locally on
  task `.trellis/tasks/06-29-project-intelligence-lsp-editor-transport`.
  Added `assura editor session` as a supported local JSON-line editor protocol
  with LSP-shaped diagnostics, context-pack, and safe-fix code-action preview
  methods over shared content-query contracts. Independent review fixed
  absolute `file://` URI handling and strengthened parity tests against
  content diagnostics, context packs, safe-fix previews, and dry-run audit IDs.
  Full LSP server framing, editor marketplace packaging, MCP, and remote access
  remain out of scope. The immediate next goal is
  `docs/goals/assura-project-intelligence-release-hardening.md`.
- 2026-06-29: Completed
  `docs/goals/assura-project-intelligence-release-hardening.md` locally on task
  `.trellis/tasks/06-29-project-intelligence-release-hardening`. Release
  readiness docs, support policy, compatibility notes, release notes,
  command-surface target-state rows, checked schema examples, final audit, and
  clean-source performance provenance now agree on the supported local Project
  Intelligence surface. MCP and remote access remain optional future adapter
  work, not requirements for local agent usability.
