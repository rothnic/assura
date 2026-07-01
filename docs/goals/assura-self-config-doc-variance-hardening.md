---
id: goal-assura-self-config-doc-variance-hardening
type: goal
title: Assura self config and documentation variance hardening
status: completed
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

## Contribution To Parent Use Case

This child goal establishes the first layer of the parent verification package.
The final fixture should be able to prove that Assura checks repository shape,
root hygiene, generated/runtime boundaries, directory placement, line limits,
and coarse Markdown scope before deeper Markdown, content-model, reference, or
language-specific diagnostics run.

For Assura's own repository, this means stale config snapshots, root clutter,
overly broad allowlists, generated/runtime outputs, and repeatable coarse
Markdown drift must either be caught, fixed, or intentionally bounded. The work
should leave concrete evidence that deeper document-graph and Markdown-engine
goals are building on a clean structure layer instead of compensating for
unclear repository policy.

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

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-01 | Revalidated this goal as the first child of the post-beta capabilities parent after PR #113 merged. Started from a clean self-check, removed stale live `.assura` config snapshots from the allowed root shape, enabled active-docs trailing-space linting, applied the existing safe Markdown fixer to active docs, and preserved archives outside the first enforcement scope. | `cargo run --quiet -- check --format json .`; `cargo run --quiet -- fix markdown --rule trailing-spaces --apply --format json .`; `.assura/config.yml`; `docs/proposals/gitignore-integration.md`; `docs/unified-tree-design.md`. |
| 2026-07-01 | Completed the focused validation gate for this slice. Self-check remained clean, active-doc Markdown safe-fix dry-run reported zero remaining fixes, Trellis context validated, docs and evidence gates passed, and the focused Markdown fixer/lint tests passed. | `cargo run --quiet -- check --format json .`; `cargo run --quiet -- fix markdown --rule trailing-spaces --dry-run --format json .`; `.trellis/tasks/archive/2026-07/07-01-self-config-doc-variance-hardening`; `git diff --check`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `cargo test --test markdown_lint_fix_tests --quiet`; `cargo test --test markdown_common_lint_tests --quiet`. |
| 2026-07-01 | Independent review found no config hierarchy blocker, but flagged stale references to the archived planning task and removed legacy config snapshot. Updated roadmap routing to the current task/branch, moved historical research references to the archived task path, and rewrote active analysis notes so they do not imply the removed snapshot is a live file. | Review agent `019f1fdd-a0b1-7ec0-bb84-8edf7333561b`; `.trellis/spec/assura/roadmap.md`; [Markdownlint-compatible Rust engine](./assura-markdownlint-compatible-rust-engine.md); [Post-beta capabilities program](./assura-post-beta-capabilities-program.md); `docs/analysis/2026-05-09-project-assessment-and-alignment.md`; `rg -n "07-01-post-beta-followup-roadmap-goals|codex/post-beta-followup-roadmap-goals|config\\.new\\.yml|config\\.yml\\.v1" .assura docs .trellis/spec .trellis/tasks/archive/2026-07/07-01-self-config-doc-variance-hardening`. |
