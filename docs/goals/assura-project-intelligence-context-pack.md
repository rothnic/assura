---
id: goal-assura-project-intelligence-context-pack
type: goal
title: Assura project intelligence context pack
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
  - docs/goals/assura-project-intelligence-real-repo-proof.md
  - docs/analysis/2026-06-29-project-intelligence-usability-gap-evaluation.md
---

# Assura Project Intelligence Context Pack

## Objective

Give humans, agents, and future transports one bounded context contract for a
project-intelligence editing task, instead of requiring callers to stitch
together check, search, graph expansion, missing relations, diagnostics, and
safe-fix previews themselves.

## Current Gap

The CLI exposes the necessary facts, and `agent-query` wraps individual
capabilities. A usable workflow still needs a single request that answers:
what is broken, what project objects are related, what source sections matter,
and what safe fixes are available.

## Scope

- Define a versioned context-pack schema over existing validation, content
  repository, graph/search, agent-query, and safe-fix preview contracts.
- Support at least diagnostic-oriented and object-oriented requests.
- Include bounded records, Markdown sections, relation edges, diagnostics, and
  safe-fix preview summaries with explicit truncation metadata.
- Add deterministic output ordering and stable IDs so agents can cite context
  back to files and model instances.
- Document when to use lower-level `assura content` commands versus the
  context-pack workflow.

## Non-Goals

- No semantic ranking as validation truth.
- No large prompt-generation framework.
- No hidden writes.
- No transport-specific behavior.

## Definition Of Done

- One command or public operation returns a context pack for the Beacon CRM
  invalid fixture and the Assura goal model.
- The context pack includes diagnostics, related modeled records, Markdown
  sections, relation status, and safe-fix preview metadata where applicable.
- Output is bounded and reports truncation or omitted capability reasons.
- Tests prove the context pack agrees with the lower-level CLI commands it
  composes.
- Docs show a complete agent editing handoff using the context pack.

## Validation Commands

```bash
cargo fmt --check
cargo test --test project_intelligence_context_pack --quiet
cargo test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm the context pack composes existing contracts instead of forking
  validation or query logic.
- R2: Confirm bounding and truncation make the output safe for agent use.
- R3: Confirm diagnostics and relations can be traced back to exact files and
  model instances.
- R4: Confirm docs do not claim semantic search decides correctness.

## Reviewer Blocking Criteria

Block if the context pack hides lower-level evidence, emits unbounded output,
reimplements validation/query behavior separately, or performs writes.

## Progress Log

- 2026-06-29: Completed implementation on task
  `.trellis/tasks/06-29-project-intelligence-context-pack`. Added
  `assura content context-pack` with schema
  `assura.project-intelligence.context-pack.v1`, diagnostic-oriented and
  object-oriented modes, explicit bounds/truncation/omission metadata, and
  composition over existing diagnostics, show, expand, search,
  missing-relations, and safe-fix query contracts. Regression coverage in
  `tests/project_intelligence_context_pack.rs` proves Beacon CRM invalid
  context and Assura goal object context agree with lower-level commands.
  Website docs now show a complete context-pack handoff.
- 2026-06-29: Addressed review findings from agent
  `019f13a9-1ead-79e1-9261-5ce3913059bd` by reporting
  `related.related` truncation in `bounds.truncated` and by expanding the
  website demo with a concrete agent editing handoff that names the task,
  context command, fields to inspect, edit constraints, verification commands,
  and expected evidence.
