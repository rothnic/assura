# Project Intelligence Assura Directory Organization

## Goal

Complete `docs/goals/assura-project-intelligence-assura-directory-organization.md`
by keeping `.assura/` scalable: root-level files stay bounded to well-known
entrypoints, while project-intelligence model artifacts that live under
`.assura/` must live below `.assura/models/**`.

## What I Already Know

- The current Assura repo has `.assura/project-intelligence-goals.schema.json`
  in the `.assura/` root.
- `.assura/config.yml` points `models.source` and
  `models.validation_artifact` at root-level `.assura/project-intelligence-*`
  paths.
- `assura init --project-intelligence` currently writes the starter runtime
  schema under `schemas/project-intelligence-starter.schema.json`.
- The user wants future model artifacts to be grouped in one model directory,
  with user-controlled hierarchy allowed inside that directory.
- The usability program requires this organization before MCP/LSP transports
  promote the starter layout further.

## Requirements

- Define `.assura/models/**` as the canonical organized model-artifact location
  for artifacts stored under `.assura/`.
- Keep `.assura/` root policy bounded in this repository's self-check config.
- Move Assura's current project-intelligence runtime schema artifact under
  `.assura/models/**`.
- Update `assura init --project-intelligence` to generate the starter schema
  under `.assura/models/**`.
- Add content-runtime validation for model config paths that point under
  `.assura/` but outside `.assura/models/**`, with actionable diagnostics.
- Preserve support for nested hierarchy under `.assura/models/**`.
- Update docs/demo/support surfaces to show the organized layout.

## Acceptance Criteria

- [x] `cargo run --quiet -- check --format json .` passes with Assura's own
  model schema under `.assura/models/**`.
- [x] Starter generation writes
  `.assura/models/project-intelligence/starter.schema.json` and the generated
  config points to that path.
- [x] A fixture with `.assura/root-level.schema.json` as
  `models.validation_artifact` produces an actionable
  `content_runtime:*` diagnostic.
- [x] A fixture with nested `.assura/models/**` schema passes content-runtime
  validation.
- [x] Website docs and project-intelligence demo mention the organized layout.
- [x] The goal and usability program progress logs record completion evidence.

## Technical Approach

Add the policy in the content repository model compiler, because every
project-intelligence command already builds through that path. The check should
only constrain model artifacts that are placed inside `.assura/`; it should not
ban projects from keeping runtime schemas under `schemas/` or another existing
project directory.

For this repo's own dogfood policy, update `.assura/config.yml` structure rules
so `.assura/models/` is an allowed directory, root-level model schema artifacts
are no longer allowlisted, and nested model files are permitted under the model
directory.

## Out Of Scope

- No automatic migration command for existing user projects.
- No redesign of content model notation or collection declarations.
- No MCP/LSP transport implementation in this task.
- No ban on external `schemas/**` paths outside `.assura/`.

## Technical Notes

- Goal: `docs/goals/assura-project-intelligence-assura-directory-organization.md`
- Program: `docs/goals/assura-project-intelligence-usability-program.md`
- Runtime model compiler: `src/content_repository/model.rs`
- Starter generation: `src/cli/init_support.rs`
- Starter tests: `tests/project_intelligence_onboarding.rs`
- Runtime validation tests: `tests/content_runtime_validation.rs`
- Self-check policy: `.assura/config.yml`
