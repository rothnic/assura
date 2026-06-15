---
id: analysis-2026-06-15-notation-clean-start-roadmap
type: analysis
title: Notation clean-start follow-up roadmap
status: active
created: 2026-06-15
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/config-notation.md
  - docs/analysis/2026-05-15-notation-source-truth.md
  - benches/history/current.json
  - website/public/data/performance/current.json
---

# Notation Clean-Start Follow-Up Roadmap

This captures the next chunks after the canonical relationship notation and PR
cleanup work. Assura is pre-1.0, so these tasks should keep removing obsolete
alpha notation instead of preserving compatibility shims.

## Chunk 1: First-Time User Notation Review

Goal: verify that a new user can read the docs and configure Assura without
learning internal structure fields first.

Proof gates:

- Run a simulated first-time setup against a small Rust CLI/library project and
  a small package-style project.
- Record what examples were missing, what wording was confusing, and which
  directives required source reading.
- Update docs so the first example starts with concise `structure:` notation,
  `rules:`, `exists`, captures, `needs`, and `provides`.
- Keep detailed `files:` and `directories:` fields as reference material, not
  the first path.

## Chunk 2: Relationship Semantics Hardening

Goal: make capture-driven relationships predictable before expanding their
surface area.

Proof gates:

- Add fixtures for ambiguous capture providers, overlapping provider kinds,
  missing counterparts, and same-name captures in separate scopes.
- Cover package documentation alternatives: dedicated
  `docs/packages/{package}.md` and aggregate `docs/packages.md` sections.
- Improve diagnostics so relationship failures name the producer, missing
  counterpart or provider kind, and the structure entry that declared it.
- Keep counterpart and provider artifacts configured where they live in the
  tree.

## Chunk 3: Markdown Outline Implementation Review

Goal: make heading validation match the concise outline notation in the config
spec without forcing users to sync heading levels manually.

Proof gates:

- Add parser tests for required headings, `?? ` optional headings, nested
  heading order, required headings containing `?`, and object-form escapes.
- Add passing and failing Markdown fixtures for nested optional parents and
  ambiguous root-level matching.
- Keep nested attributes only for custom match behavior such as regex or
  future validators.

## Chunk 4: Performance Analysis

Goal: identify evidence-backed performance opportunities after the freshness
fixes and canonical notation work land.

Proof gates:

- Refresh baseline artifacts in `benches/history/current.json` and
  `website/public/data/performance/current.json`.
- Profile parsing, shorthand normalization, relationship validation traversal,
  compiled artifacts, and fast-path disabling.
- Compare against the previous LS-Lint and expanded-suite performance history.
- Produce a ranked report that separates confirmed bottlenecks from suspected
  opportunities.
- Do not implement optimizations in this chunk unless the analysis exposes a
  trivial correctness-preserving cleanup.

## Chunk 5: Performance Implementation Follow-Up

Goal: implement only the opportunities proven by Chunk 4.

Proof gates:

- Preserve fast paths when no relationship rules are configured.
- Evaluate traversal reuse, provider lookup indexing, and avoiding repeated
  capture matching.
- Require before/after benchmark evidence and full correctness tests for every
  optimization.
- Keep LS-Lint compatibility performance and Assura-authored notation
  robustness as separate claims.
