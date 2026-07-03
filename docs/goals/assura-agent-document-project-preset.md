---
id: goal-assura-agent-document-project-preset
type: goal
title: Assura agent document project preset
status: completed
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-content-activation-source-docs.md
---

# Assura Agent Document Project Preset

## Objective

Provide a reusable document-project preset for research, documentation,
proposal, compliance, and knowledge-base repositories without making the core
agent-project preset domain-specific.

## Scope

- Add a document-project preset layered on top of the broad agent-project
  baseline.
- Provide default structures for source documents, topics, drafts, final docs,
  requirements, evidence, decisions, process docs, and learnings.
- Include binary/source-document custody defaults from the content activation
  goal.
- Keep project-type detection conservative: apply the pack only when explicit
  or confidently detected, otherwise ask specialization questions.
- Document how the preset composes with agent-project, Rust, Node, Python,
  web-app, monorepo, and future domain packs.

## Non-Goals

- No proposal/SBIR scoring in the generic document-project preset.
- No required hosted search or semantic index.
- No generated final PDF/DOCX packaging.

## Definition Of Done

- A new document-heavy repo can apply the preset and get useful structure,
  custody, Markdown, reference, and content-model readiness checks.
- Existing repos can merge the preset without losing local conventions.
- The preset remains broad enough for docs, research, compliance, and knowledge
  bases.
- Website onboarding explains when to choose document-project versus
  agent-project.

## Validation Commands

```bash
cargo fmt --check
cargo test onboarding --quiet
cargo test content_runtime --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if the preset hardcodes proposal/SBIR assumptions, reads binary files as
text, cannot merge safely into an existing repository, or duplicates behavior
that belongs in the base agent-project preset.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-03 | Completed the generic document-project preset layer. `--content-template document-project` now composes from the base agent-project content config, adds source-document custody plus `library/topics/`, `docs/drafts/`, and `docs/final/` starter records, models Topic, Draft, and FinalDocument collections and relations, preserves binary-safe source-file validation, and keeps proposal/SBIR behavior out of the generic preset. | `src/cli/agent_onboarding_content_templates.rs`; `src/cli/agent_onboarding_document_project_templates.rs`; `src/cli/agent_onboarding_templates.rs`; `tests/project_intelligence_onboarding.rs`; `website/src/content/docs/guides/agent-ready-onboarding.md`; `website/src/content/docs/reference/api.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; independent review agent `019f2679-dad9-7d81-8b83-40e08504cc94`; `cargo fmt --check`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo test --test project_intelligence_onboarding source_document_custody_does_not_read_binary_targets_as_utf8 --quiet`; `cargo test content_runtime --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
