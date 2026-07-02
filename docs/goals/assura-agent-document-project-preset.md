---
id: goal-assura-agent-document-project-preset
type: goal
title: Assura agent document project preset
status: planned
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
