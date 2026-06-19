---
id: goal-assura-roadmap-09-first-time-configuration-authoring
type: goal
title: Assura roadmap 09 first-time configuration authoring
status: completed
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/2026-06-15-notation-clean-start-roadmap.md
---

# Goal 09: First-Time Configuration Authoring

## Objective

Make the first successful Assura configuration path obvious for a new user who
has not read the internal config model or historical notation documents.

This is a two-week team chunk for docs, examples, config ergonomics, and
reviewers focused on first-run adoption.

## Current Gap

This goal is not complete today because the repo has a canonical config
notation spec and reference docs, but no checked first-time-user journey that
proves a new user can author useful config from the public docs alone. Existing
command-surface checks prevent some stale command claims; they do not prove the
authoring path is understandable or complete.

## User Certainty Bar

A new user should be able to start with an empty project, add Assura, write the
first useful `.assura/config.yml`, run the documented command, understand every
violation, and recover without reading Rust source, historical analysis docs, or
agent-only planning notes.

## Scope

- Run a simulated first-time setup against a small Rust CLI/library project and
  a small package-style project.
- Build a notation use-case matrix that starts with LS-Lint-equivalent naming,
  extension, closed-world, ignore, and direct-child presence cases, then adds
  Assura-native `exists`, capture, relationship, Markdown outline, and reusable
  `rules:` cases.
- Rewrite the first-path docs and examples around concise `structure:`,
  `rules:`, `exists`, captures, `needs`, and `provides`.
- Update all affected public examples, website examples, generated examples,
  fixture configs, and test-case `.assura/config.yml` files to the same
  first-path notation.
- Keep detailed `files:` and `directories:` fields as reference material, not
  the initial authoring path.
- Record any confusing directives, missing examples, or source-reading traps in
  a durable review artifact.
- Keep the canonical relationship notation from Goal 08 follow-up work intact;
  this goal improves adoption around it rather than redesigning it.

## Non-Goals

- No new parser or runtime relationship semantics unless a first-run bug blocks
  a documented example.
- No watch-mode hardening.
- No plugin marketplace or remote extension loading.
- No revival of removed alpha notation such as `${name}` or `{{name}}`.
- No backwards-compatibility shims for superseded notation.

## Definition Of Done

- A new user can create a minimal `.assura/config.yml` for both fixtures by
  following docs without reading source code.
- The notation use-case matrix proves the first path covers LS-Lint-style
  policies and the next Assura-native policies without separate notation
  families.
- The first examples use the canonical tree-shaped notation.
- Docs explain when to reach for detailed reference fields.
- The review artifact records the simulated first-run journey, confusing steps,
  and fixes made.
- Every failed step in the simulated journey becomes either a docs fix, a
  diagnostic fix, or an explicitly deferred product gap.
- Existing command-surface and self-check constraints pass.
- If this goal changes notation behavior, performance gates either pass or the
  PR records a bounded inherent notation cost with user-value justification.

## Required Validation

```bash
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask target-state
cargo run --quiet -- performance-report --output target/performance/current.json
git diff --check
```

## Review Tasks

- R0: Confirm this goal starts from the current config notation spec.
- R1: Review the first-run examples as a user who has never seen Assura.
- R2: Review the notation use-case matrix for LS-Lint baseline coverage,
  Assura-native extensions, and reusable-rule modularity.
- R3: Verify examples do not depend on removed alpha notation or custom
  constraints for common relationships.
- R4: Reproduce the small Rust and package-style setup flows.
- R5: Confirm docs keep detailed reference material available without making it
  the first path.
- R6: Confirm examples and test-case configs were migrated consistently.
- R7: Confirm the PR links this goal and the first-run review artifact.

## Reviewer Blocking Criteria

Block the PR if a user must read source code to finish the first config, if the
docs present `extensions.custom_constraints` as the common relationship syntax,
if examples rely on notation that the current spec rejects, or if notation
changes skip performance evidence without a documented bounded-cost exception.
Also block if the use-case matrix does not start from LS-Lint-equivalent cases
or fails to show how reusable rules make broader Assura policies more modular.

## Progress Log

- 2026-06-18: Started Goal 09 execution after PR #50 merged. Revalidated the
  goal as current, added first-time configuration docs/examples, recorded the
  simulated first-run review artifact, and added executable temp-project tests
  for the small Rust and package-style first-run paths. Migrated the generated
  `assura init` starter config to concise tree-shaped notation.
- 2026-06-19: Revalidated status against live PR and Trellis evidence. Goal 09
  merged in PR #51 (`979a86138fe6eb0fc72751fa6dd71f2404d7e5fc`) and is
  archived under `.trellis/tasks/archive/2026-06/06-18-goal-09-first-time-config`.
