# Agent Content Activation And Source Docs

## Problem

`assura agent onboard` creates the broad agent-ready baseline, but it still
reports content models as inactive. Existing `assura init --project-intelligence`
can activate a small Goal/Spec/Decision starter, yet the agent-ready onboarding
program needs common agent-project and document-project content activation plus
source-document custody that is broad, non-domain-specific, and explicit about
what is checked versus unchecked.

## Goal

Implement the next child goal from
`docs/goals/assura-agent-content-activation-source-docs.md` so a project can
intentionally activate baseline content models and source-document custody
without requiring domain-specific domain packs.

## Requirements

- Provide common baseline models for agent-project facts:
  Decision, Task, Requirement, Evidence, Doc, SourceDocument, Finding, Skill,
  Process, and Learning.
- Add an activation path for agent-project content and document-project source
  documents that writes deterministic repo-native files under the existing
  `.assura/models/**` artifact layout.
- Keep draft model files distinct from active content models; doctor must make
  unwired models, empty collections, missing schemas, zero search chunks, and
  relation definitions without edges visible.
- Add a source-document custody pattern using `source-documents/manifest.md`
  and `source-documents/files/`, validating metadata/reference integrity
  without reading binary files as UTF-8.
- Preserve current command-surface truth. Future CLI surfaces must remain
  clearly marked until implemented.
- Keep domain-specific domain-specific behavior out of the core templates.

## Non-Goals

- No hosted content service.
- No semantic search requirement.
- No proposal scoring, domain-specific pack, or domain-specific source-document policy.
- No public plugin API.

## Evidence

- `docs/goals/assura-agent-ready-project-onboarding-program.md`
- `docs/goals/assura-agent-content-activation-source-docs.md`
- `docs/goals/assura-content-model-source-of-truth.md`
- `docs/goals/assura-supported-document-graph.md`
- `docs/goals/assura-content-query-and-search-cli.md`
- `docs/support-policy.md`
- `.trellis/spec/assura/roadmap.md`

## Validation

- `cargo fmt --check`
- `cargo test content_runtime --quiet`
- `cargo test --test project_intelligence_onboarding --quiet`
- `cargo test --test content_runtime_references --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

## Review Criteria

Block if draft models can silently appear active, if source-document custody
requires binary text reads, if the baseline models are proposal-specific, if
doctor hides inactive/empty content state, or if source-document custody
requires a domain pack.
