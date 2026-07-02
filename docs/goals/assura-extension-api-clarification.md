---
id: goal-assura-extension-api-clarification
type: goal
title: Assura extension API clarification
status: completed
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

## User-Specific Certainty Bar

Nick should be able to ask "what are extension APIs in Assura?" and get one
concrete answer:

- current `extensions.*` entries are first-party config policy families that
  run inside `assura check`;
- supported integrations should use local CLI, daemon, content, agent, and
  editor JSON contracts;
- public Rust module visibility is internal/unstable before 1.0 unless a
  support matrix explicitly says otherwise;
- VS Code and agent packages are local wrappers over shared contracts, not
  marketplaces or private validation engines;
- public third-party plugin APIs, remote plugin loading, shell-executed
  validators, TypeScript plugin APIs, and semver-stable Rust APIs are deferred
  until a separate goal proves sandboxing, versioning, distribution, security,
  diagnostics, and performance gates.

The final output of this goal should prevent a future agent from implementing
or documenting "extension APIs" as a plugin marketplace, shell plugin runner,
or public SDK merely because `extensions.*` exists in `.assura/config.yml`.

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

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Started the Extension API Clarification child after PR #133 merged. The slice is scoped to a canonical extension/API boundary, support rows for every current `extensions.*` family, and target-state checks that prevent accidental public plugin/API claims. | `.trellis/tasks/archive/2026-07/07-02-extension-api-clarification/prd.md`; `.trellis/spec/assura/roadmap.md`; `docs/extension-api-boundaries.md`; branch `codex/extension-api-clarification`. |
| 2026-07-02 | Recorded the beta boundary decision: current `extensions.*` entries are first-party config policies run by `assura check`; supported integrations use local CLI, daemon, content, agent, and editor JSON contracts; internal Rust modules remain unstable before 1.0; public third-party plugin APIs, remote plugin loading, shell-executed validators, marketplace plugins, TypeScript plugin APIs, and semver-stable Rust APIs are deferred until a separate proof-gated goal. | `docs/extension-api-boundaries.md`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `website/src/content/docs/reference/extension-api-boundaries.md`; `website/src/content/docs/reference/api.md`; `website/src/content/docs/reference/configuration.md`. |
| 2026-07-02 | Added target-state guardrails for the boundary language and release-surface registration so future docs must keep the canonical page, support policy, compatibility matrix, public website references, current `extensions.*` family inventory, and unsupported plugin/API markers aligned. | `xtask/src/main.rs`; `docs/data/release-surfaces.json`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `cargo run --quiet -- check --format json .`; `git diff --check`. |
| 2026-07-02 | Closed the Extension API Clarification child after independent review found no blockers. Residual risk is intentionally bounded: the target-state guardrail is marker-based rather than a semantic documentation parser, but it covers the required boundary terms, family inventory, support/compatibility rows, website links, release-surface registration, and obvious unsupported-support claims. | Review agent `019f22cf-d235-7f93-9b03-bc0712c2e90c`; `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo check --workspace --all-targets --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
