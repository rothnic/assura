---
title: Self config and documentation variance hardening
status: active
---

# Self Config And Documentation Variance Hardening

## Goal

Execute the first child goal from
`docs/goals/assura-post-beta-capabilities-program.md`: refine Assura's own
`.assura/config.yml`, audit repository structure and Markdown documentation
variance, and convert durable conventions into enforceable coarse rules before
deeper Markdown/content/reference validation work starts.

## Current State

- PR #113 is merged into `master` as commit `be4e33e`.
- This task runs on branch `codex/self-config-doc-variance-hardening`, based on
  `origin/master`.
- Baseline `cargo run --quiet -- check --format json .` is clean.
- The parent program remains a beta-track increment. This work should not claim
  Assura is beyond beta, but later child goals should produce a versioned beta
  increment.

## Requirements

- Revalidate `docs/goals/assura-self-config-doc-variance-hardening.md` against
  live repo state.
- Audit `.assura/config.yml` for stale allowlists, broad exclusions, missing
  root hygiene, directory-shape gaps, Markdown scope gaps, generated/runtime
  path handling, and docs/goals conventions.
- Inspect current Markdown documentation variance with deterministic commands,
  prioritizing coarse file-level issues before deeper Markdown linting.
- Fix repo docs/structure drift that the config should catch, or document
  intentional variance with narrow suppressions or support notes.
- Record any repeatable Markdown formatting/fixing opportunities discovered
  during dogfooding so the Markdown engine goal can implement deterministic
  fix utilities with tests.
- Preserve the validation hierarchy: structure and coarse file-level policy
  first; Markdown, content models, repository references, and language-specific
  checks later.
- Anchor the parent program to a concrete final verification use case so later
  child goals prove the user outcome rather than only isolated task completion.
- Update the parent/child goal progress logs with evidence.

## Acceptance Criteria

- [x] `.assura/config.yml` is audited and updated where durable conventions are
      missing or stale.
- [x] Assura's own structure and docs pass the refined self-check.
- [x] Documentation variance is either corrected, narrowly documented, or
      captured as a future Markdown fixer/test candidate.
- [x] The parent program describes a concrete, user-specific final verification
      scenario for the full major increment.
- [x] The parent and child goal docs record progress and evidence.
- [x] Validation passes:
      `cargo run --quiet -- check --format json .`, `cargo xtask target-state`,
      `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check`.
- [x] Independent review finds no broad config exception, hidden drift, or
      inverted validation hierarchy.

## Out Of Scope

- Implementing the full markdownlint-compatible engine.
- Implementing daemon, agent, VS Code, extension API, or LS-Lint performance
  work in this child slice.
- Promoting Assura out of beta.

## Technical Notes

- Use `.trellis/spec/assura/structure-enforcement.md` and
  `.trellis/spec/assura/config-notation.md` for config behavior.
- Prefer config changes that make durable structure policy explicit. Avoid
  broad exemptions that only make today's tree pass.
- If Markdown fixes are obvious and deterministic but not yet supported, record
  them as testable candidates rather than hand-waving them away.
