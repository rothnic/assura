---
id: goal-assura-extension-api-clarification
type: goal
title: Assura extension API clarification
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-goal-07-extension-and-plugin-foundation.md
  - ../compatibility-and-surface.md
  - ../support-policy.md
---

# Assura Extension API Clarification

## Objective

Clarify what "extension APIs" means for Assura after `v0.2.0`, then either
stabilize a narrow first-party extension contract or explicitly defer public
third-party plugin APIs with documented boundaries.

## Current Gap

Assura currently uses `extensions.*` for first-party policy families such as
custom constraints, support matrices, manifest semantics, test relationships,
module topologies, docs lifecycles, and repository references. That is not the
same as a public remote plugin API, marketplace, shell-executed plugin system,
or semver-stable Rust crate API. The product docs need to make this distinction
unambiguous before later work depends on it.

## Scope

- Inventory all current `extensions.*` config families and their support
  status.
- Decide which surfaces are first-party config extensions, internal Rust APIs,
  public CLI contracts, or future plugin API candidates.
- Define the pre-1.0 compatibility promise for each category.
- Update support policy, compatibility matrix, configuration docs, and release
  surface manifests.
- Add target-state or evidence checks that prevent docs from claiming remote
  plugins, shell plugins, marketplaces, or semver-stable Rust APIs without a
  deliberate goal.
- If a public plugin API is desired, create a successor implementation goal with
  sandboxing, versioning, security, distribution, and performance proof gates.

## Non-Goals

- No remote plugin loading in this goal.
- No shell-executed third-party validators.
- No marketplace or hosted extension registry.
- No semver-stable public Rust API before an explicit stabilization decision.

## Definition Of Done

- Docs define "first-party extension policy" versus "public plugin API" in one
  canonical place.
- Every current `extensions.*` family has a support-status row and evidence
  pointer.
- Unsupported plugin/API claims are detectable by target-state or evidence
  checks.
- A successor goal exists only if the product decision is to build a real public
  plugin API.
- Independent review confirms the term "extension API" is no longer ambiguous.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
```

## Reviewer Blocking Criteria

Block if docs still conflate `extensions.*` config with a public plugin API, if
unsupported remote/shell/marketplace claims remain, if support status rows are
missing, or if any new public API compatibility promise lacks a release policy.
