---
title: Deslopify Completion Audit
status: active
---

# Deslopify Completion Audit

This audit maps `.trellis/tasks/06-08-full-deslopify-plan/prd.md` to checked
repo evidence after PR #38 merged.

## Acceptance Evidence

| Requirement | Status | Evidence |
| --- | --- | --- |
| R1 public surface alignment | Satisfied with deterministic guard | `src/lib.rs` marks full-CLI Rust modules as unstable internal APIs; `docs/support-policy.md` and `docs/compatibility-and-surface.md` classify unsupported surfaces; `node --run verify:evidence` rejects unsupported release-positioning claims in `Cargo.toml` and CLI about text. |
| R2 Assura config tightening | Satisfied for current rules | `.assura/config.yml` limits skill file size/lines, narrows skill directories, keeps `tests/fixtures/ls-lint/**` and `tests/fixtures/real-project-agentic-feedback/**` as explicit fixture-family exclusions, tightens active docs/goals, and restricts `src/cli` module topology. |
| R3 external Rust hygiene gates | Satisfied for practical subset | `docs/analysis/2026-06-08-deslopify-hygiene-evaluation.md` adopts `cargo-machete` through `node --run verify:hygiene`, keeps `cargo audit`, and records why `cargo-deny`, `cargo-semver-checks`, and `cargo-nextest` are deferred. |
| R4 generalized Assura rule backlog | Satisfied | `docs/goals/assura-rule-command-surface-documentation.md`, `assura-rule-cargo-manifest-semantics.md`, `assura-rule-module-topology.md`, `assura-rule-test-relationship.md`, `assura-rule-release-sync.md`, and `assura-rule-public-surface-support-matrix.md` exist as durable follow-up goals. |
| R5 dead/abandoned path audit | Satisfied as containment | `docs/analysis/2026-06-08-deslopify-dead-path-classification.md` classifies each requested code area and explains why removal is not safe in this cleanup slice without changing tests, benchmarks, or support evidence. |
| R6 performance preservation | Satisfied for merged slices | PR #38 passed CI Performance Report after adding the command-surface rule. Current workflow policy requires Performance Report for Rust, workflow, release, or performance-sensitive changes. |

## Node Runtime Policy

The live workflow already runs Node 24 through `actions/setup-node`.
`package.json` now declares `engines.node: >=24`, and
`node --run verify:evidence` parses workflow `node-version` declarations and
checks that the local/package policy does not drift below the highest CI
baseline. Historical performance reports that record `node v25.6.0` are
measurement evidence, not runtime policy.

## Remaining Work

The deslopify implementation should not delete the experimental Rust module
families in this task. The durable follow-up goals are the right place to turn
the containment decisions into product-grade Assura rule families.
