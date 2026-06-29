---
id: goal-assura-project-intelligence-assura-directory-organization
type: goal
title: Assura project intelligence Assura directory organization
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-onboarding-template.md
  - docs/goals/assura-project-intelligence-safe-fix-workflow.md
  - .assura/config.yml
---

# Assura Project Intelligence Assura Directory Organization

## Objective

Keep `.assura/` scalable by enforcing a bounded root and requiring
project-intelligence model artifacts to live under one documented model
directory, with user-defined hierarchy allowed inside that directory.

## Current Gap

The current project and starter path can place project-intelligence schema or
model artifacts directly under `.assura/`. That does not scale as Assura adds
more content collections, generated schemas, migrations, examples, transport
metadata, or support contracts. Root-level `.assura/` files should remain
reserved for a short list of well-known entrypoints, not become a catch-all for
every model-adjacent artifact.

## Scope

- Define the canonical layout for project-intelligence model artifacts, with a
  default of `.assura/models/**` unless implementation evidence identifies a
  better compatible path.
- Keep root-level `.assura/` files bounded to documented well-known entrypoints
  such as `config.yml`, command-surface contracts, and hooks.
- Allow users to add hierarchy below the model directory for domains,
  collections, generated schemas, fixtures, or versioned model packages.
- Move Assura's own project-intelligence model/schema artifacts into the
  canonical model directory.
- Update `assura init --project-intelligence` so new projects start with the
  organized layout.
- Add validation that rejects or warns on root-level `.assura/` model artifacts
  and points users to the model directory.
- Update docs, demos, support policy, and command-surface examples so the
  organized layout is the only promoted path.

## Non-Goals

- No migration assistant for arbitrary historical user layouts beyond a clear
  diagnostic and documented manual move.
- No ban on user hierarchy inside the model directory.
- No broad redesign of the project-intelligence content model language.
- No transport protocol work; MCP and LSP goals consume the organized layout
  after this goal proves it.

## Definition Of Done

- `.assura/` root policy is documented and self-checked in this repository.
- Project-intelligence model artifacts in this repo live under the canonical
  model directory rather than the `.assura/` root.
- Generated starter files from `assura init --project-intelligence` use the
  canonical layout.
- Validation catches at least one root-level `.assura/` model artifact fixture
  with an actionable diagnostic.
- Tests prove nested model-directory hierarchy remains valid.
- Website docs and the project-intelligence demo show the organized layout.

## Validation Commands

```bash
cargo fmt --check
cargo test --test project_intelligence_onboarding --quiet
cargo test --test content_runtime_validation --quiet
cargo test --test content_runtime_dx_docs project_intelligence_demo_is_discoverable_and_covers_adoption_commands --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the `.assura/` root allowlist is small, documented, and not a
  loophole for new model files.
- R2: Confirm nested paths under the model directory remain supported.
- R3: Confirm generated starters and existing Assura project config use the
  same layout.
- R4: Confirm diagnostics explain how to move root-level model artifacts
  without implying unsupported automatic migration.

## Reviewer Blocking Criteria

Block if project-intelligence model/schema artifacts can still be promoted in
the `.assura/` root, if the rule prevents reasonable hierarchy under the model
directory, if the starter and docs diverge, or if the implementation weakens
existing project-intelligence validation.

## Progress Log

- 2026-06-29: Completed locally on task
  `.trellis/tasks/06-29-project-intelligence-assura-directory-organization`.
  Added content-runtime path policy that rejects `models.source` and
  `models.validation_artifact` under `.assura/` unless the artifact is under
  `.assura/models/**`, while preserving project-relative paths outside
  `.assura/` such as `schemas/**`. Moved Assura's own runtime schema to
  `.assura/models/goals/project-intelligence-goals.schema.json`, updated
  `assura init --project-intelligence` to generate
  `.assura/models/project-intelligence/starter.schema.json`, and updated the
  docs/demo/support surfaces to show the organized layout.
- 2026-06-29: Independent review found that `./.assura/...` paths bypassed the
  policy because current-directory path components were not normalized before
  the `.assura/` check. Fixed by normalizing `Component::CurDir` out of
  project-relative paths and added regressions for dot-prefixed
  `models.validation_artifact`, dot-prefixed `models.source`, and the CLI
  `assura check --format json` diagnostic.
