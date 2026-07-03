---
id: goal-assura-agent-content-activation-source-docs
type: goal
title: Assura agent content activation and source docs
status: completed
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-content-model-source-of-truth.md
  - ./assura-project-intelligence-onboarding-template.md
---

# Assura Agent Content Activation And Source Docs

## Objective

Make content runtime activation obvious and provide common project facts plus
binary source-document custody so agents do not confuse draft models with
active modeled content.

## Scope

- Add content initialization templates for agent-project and document-project
  style repositories.
- Add content doctor output for model files that exist but are not wired into
  config, empty collections, missing schemas, zero search chunks, and relation
  definitions with no edges.
- Provide baseline models for Decision, Task, Requirement, Evidence, Doc,
  SourceDocument, Finding, Skill, Process, and Learning.
- Support a source-document custody pattern with manifest, file existence,
  naming, optional checksums, kind, origin, related requirements, and notes.
- Ensure binary files are never read as UTF-8 by default.

## Non-Goals

- No one-off scoring pack in the core model set.
- No hosted content service.
- No mandatory semantic search.

## Definition Of Done

- A first-run project can intentionally activate baseline content models.
- Doctor reports inactive or empty content capabilities clearly.
- Source documents are tracked through metadata and manifest/reference
  integrity, not text reads.
- Fixtures cover missing binary files, bad manifest links, and active versus
  inactive content models.

## Validation Commands

```bash
cargo fmt --check
cargo test content_runtime --quiet
cargo test --test project_intelligence_onboarding --quiet
cargo test --test content_runtime_references --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if draft models can silently appear active, if binary files are read as
text, if baseline models are tailored to one document-heavy workflow, or if
source-document custody requires a separate pack.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-03 | Completed content activation and source-document custody. `assura agent onboard` now has explicit `--content-template none`, `agent-project`, and `document-project` modes; generated agent projects activate baseline Decision, Task, Requirement, Evidence, Doc, Finding, Skill, Process, and Learning models; generated document projects add SourceDocument custody metadata and file-reference validation without reading binary targets as UTF-8. Doctor now reports content runtime schema, empty-collection, zero-chunk, and relation-edge gaps when content is active. | `src/cli/agent_args.rs`; `src/cli/agent_onboarding.rs`; `src/cli/agent_onboarding_templates.rs`; `src/cli/agent_onboarding_content_templates.rs`; `src/cli/doctor.rs`; `src/cli/doctor_content.rs`; `tests/project_intelligence_onboarding.rs`; `tests/doctor_explain_cli.rs`; independent review agent `019f2624-a975-7812-9bcb-cac759e40e73`; `cargo fmt --check`; `cargo test content_runtime --quiet`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo test --test doctor_explain_cli --quiet`; `cargo test --test content_runtime_references --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
