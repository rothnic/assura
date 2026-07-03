# Agent Document Project Preset

## Goal

Make `assura agent onboard --content-template document-project` produce a
reusable, broad document-project pack layered on the agent-project baseline.
The preset should help research, documentation, compliance, and knowledge-base
repositories organize source documents, topics, drafts, final documents,
requirements, evidence, decisions, process docs, and learnings without adding
domain-specific scoring or domain-specific behavior.

## What I Already Know

- Parent program:
  `docs/goals/assura-agent-ready-project-onboarding-program.md` is active.
- Child goal:
  `docs/goals/assura-agent-document-project-preset.md` is the next executable
  goal after the completed website onboarding goal.
- Current support policy classifies `assura agent onboard` and
  `--content-template document-project` as experimental local onboarding
  surfaces.
- Current implementation already supports `--content-template document-project`
  in `src/cli/agent_onboarding_content_templates.rs`.
- Current document-project behavior adds the agent-project content model,
  `source-documents/manifest.md`, `source-documents/files/sample-source.txt`,
  repository-reference checks for `source_files`, and binary-safe custody tests.
- The remaining gap is the broader project-type pack from the parent program:
  `library/topics/`, `docs/drafts/`, `docs/final/`, `requirements/`,
  `evidence/`, `decisions/`, plus clear docs about when to choose
  document-project.

## Revalidation Result

Status: valid.

The goal is partially implemented by the content-activation slice, but not
complete. Existing tests prove source-document custody and binary-safe reference
validation; they do not prove that the document-project preset creates useful
topic, draft, final-document, requirement, evidence, and decision structure or
that those records are modeled and discoverable.

## Requirements

- Extend `document-project` on top of `agent-project`; do not duplicate or
  weaken the broad baseline.
- Generate broad document-project structure for:
  - `source-documents/`;
  - `library/topics/`;
  - `docs/drafts/`;
  - `docs/final/`;
  - requirements, evidence, decisions, process docs, and learnings.
- Keep generated records generic enough for research, documentation,
  compliance, and knowledge-base repositories.
- Keep domain-specific scoring, portals, weighted criteria, and review-package
  behavior out of this preset.
- Preserve existing-file safety and merge behavior.
- Validate referenced source file paths without reading binary files as UTF-8.
- Update website/reference docs so users know when to choose `document-project`
  versus `agent-project`.
- Add target-state or regression coverage that prevents the document-project
  pack from shrinking back to only a source-document manifest.

## Acceptance Criteria

- [ ] `assura agent onboard . --content-template document-project --format json`
      generates document-project folders and starter records for source
      documents, topics, drafts, final docs, requirements, evidence, decisions,
      process docs, and learnings.
- [ ] Generated config models the new document-project records and passes
      `assura check --format json`.
- [ ] Existing source-document custody behavior still catches missing referenced
      files and still accepts binary targets without reading them as text.
- [ ] Existing user-authored files are preserved.
- [ ] Website/reference docs explain the broader document-project preset and
      keep domain-specific domain behavior out of the generic preset.
- [ ] Validation commands from the goal pass.

## Definition Of Done

- Rust implementation and tests updated.
- Docs updated for command-surface truth.
- Independent review checks for overfitting, binary reads, merge safety, and
  duplicated base-preset behavior.
- Goal progress logs updated with evidence.
- Trellis task archived and changes committed.

## Out Of Scope

- No `assura init --preset document-project` surface in this slice; current
  user entrypoint remains `assura agent onboard --content-template`.
- No domain-specific scoring, portal submission, weighted review criteria, or
  final package checks.
- No hosted search, semantic index, or generated PDF/DOCX packaging.

## Technical Notes

- Template implementation:
  `src/cli/agent_onboarding_content_templates.rs`.
- Baseline file installation and merge behavior:
  `src/cli/agent_onboarding_templates.rs` and `src/cli/agent_onboarding.rs`.
- Existing tests:
  `tests/project_intelligence_onboarding.rs`.
- Website onboarding docs:
  `website/src/content/docs/guides/agent-ready-onboarding.md` and
  `website/src/content/docs/reference/api.md`.
- Source-of-truth support policy:
  `docs/support-policy.md` and `docs/compatibility-and-surface.md`.
