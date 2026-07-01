---
id: goal-assura-self-config-doc-variance-hardening
type: goal
title: Assura self config and documentation variance hardening
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ../../.assura/config.yml
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Self Config And Documentation Variance Hardening

## Objective

Dogfood Assura's own configuration before the next product iteration by
refining `.assura/config.yml`, auditing current repository structure and
Markdown documentation variance, and turning intentional conventions into
clear rules or documented suppressions.

## Current Gap

Assura has grown new docs, goals, integrations, daemon metadata, and
project-intelligence surfaces quickly. The current self-check is clean, but
the configuration should be re-audited against the actual repository shape so
Assura catches the right coarse issues before deeper Markdown, content,
reference, or language-specific validation runs.

## Scope

- Run a baseline self-check and capture the current structure and Markdown
  variance profile.
- Review `.assura/config.yml` for stale allowlists, overly broad exceptions,
  missing root hygiene, directory child limits, docs/goals patterns, model
  artifact placement, integration paths, and generated/runtime paths.
- Encode intentional repository structure, line-length, directory-shape,
  Markdown scope, and docs lifecycle expectations at the coarsest useful layer.
- Fix documentation or structure drift found by the revised configuration.
- Record any intentional variance with bounded suppressions or support docs,
  not silent broad exceptions.

## Non-Goals

- No new Markdown linter engine in this goal.
- No mass rewrite of docs without a rule-backed reason.
- No language-specific or Markdown-specific lint running before structure and
  coarse file-level policy in Assura's staged quality hierarchy.

## Definition Of Done

- `.assura/config.yml` reflects the current repository structure and support
  boundaries without stale broad exceptions.
- Assura's own docs and structure pass the refined self-check.
- Any discovered documentation variance is either corrected or documented as
  intentional with a narrow rule/suppression.
- The docs explain that structure and coarse file-level policy are evaluated
  before deeper Markdown, content-model, reference, and language-specific
  checks.
- Independent review confirms the config is not overfit to hide current drift.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm new or changed config rules encode durable project conventions.
- R2: Confirm suppressions are narrow and reasoned.
- R3: Confirm Markdown and content validation remain deeper layers under
  structure and coarse file policy.
- R4: Confirm docs/goal variance fixes are intentional and not cosmetic churn.

## Reviewer Blocking Criteria

Block if the config hides drift through broad allowlists, if validation order
documentation implies Markdown linting sits above structure, if generated files
or daemon runtime state can pollute the supported repository shape, or if the
goal changes many docs without linking the changes to an Assura rule.
