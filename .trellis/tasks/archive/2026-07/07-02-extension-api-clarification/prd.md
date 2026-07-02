# Clarify Extension API Boundaries

## Goal

Execute `docs/goals/assura-extension-api-clarification.md` by making the term
"extension API" unambiguous for this beta increment.

The outcome should help a maintainer decide whether they can rely on current
`extensions.*` policies, public CLI contracts, internal Rust modules, or any
future plugin API. The answer must be concrete: current `extensions.*` entries
are first-party config policies, not a public third-party plugin system.

## What I Already Know

- PR #133 merged the supported beta local VS Code package and the parent
  roadmap now routes to Extension API Clarification.
- `.assura/config.yml` uses `extensions.manifest_semantics`,
  `extensions.test_relationships`, `extensions.module_topologies`,
  `extensions.docs_lifecycles`, `extensions.support_matrices`, and
  `extensions.custom_constraints`.
- `extensions.repository_references` is documented and tested as an
  experimental first-party repository-reference check.
- `extensions.relationships` and `extensions.release_contracts` still exist in
  config/source compatibility paths and need an explicit support/boundary
  classification if docs discuss the extension namespace as a whole.
- Support policy already marks many `extensions.*` families experimental
  first-party and marks internal Rust APIs unstable, but there is no single
  canonical boundary page tying together config policies, CLI contracts,
  internal Rust APIs, editor/agent bundles, and deferred public plugins.

## Requirements

- Define a canonical extension/API boundary reference for this beta increment.
- Inventory current `extensions.*` families and classify each as supported,
  experimental first-party, internal/compatibility, roadmap, or unsupported.
- Clarify that first-party config policies run inside `assura check` and do not
  grant remote plugin loading, shell execution, marketplace distribution, or
  semver-stable Rust/TypeScript plugin APIs.
- Update support policy, compatibility matrix, configuration reference, API
  docs, release readiness, release surfaces, and roadmap/goal progress so they
  agree.
- Add target-state or evidence checks for the boundary language so future docs
  cannot casually claim a public plugin API, marketplace, shell plugin, or
  semver-stable Rust API.
- Decide whether to create a successor public-plugin goal. If not, explicitly
  defer it and explain the proof gates required before reopening it.

## Acceptance Criteria

- [ ] A reader can answer "what are Assura extension APIs?" from one canonical
      reference without reading historical goals.
- [ ] Every current `extensions.*` family has a support-status row or explicit
      boundary classification.
- [ ] Public docs distinguish first-party config policies, supported CLI JSON
      contracts, internal Rust APIs, local editor/agent packages, and future
      public plugin/API candidates.
- [ ] Target-state/evidence checks fail when the canonical boundary doc,
      support policy, or compatibility docs stop naming the unsupported plugin
      claims.
- [ ] The child goal and parent progress log record the decision and validation
      evidence.

## Out Of Scope

- Implementing remote plugins, shell-executed validators, marketplace
  distribution, or a public plugin SDK.
- Stabilizing Rust crate APIs before a separate 1.0/API-governance decision.
- Changing existing `extensions.*` runtime behavior unless a docs check exposes
  a concrete bug.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Criteria

Independent review must block if:

- Docs still use "extension API" ambiguously.
- `extensions.*` config policy is described as public plugin support.
- Remote plugins, shell plugins, marketplaces, semver-stable Rust APIs, or
  TypeScript plugin APIs are implied as current supported surfaces.
- Current `extensions.*` families lack support classification.
- Target-state/evidence checks do not cover the new boundary language.
