---
title: Assura Best-Practice Target State
status: active
---

# Assura Best-Practice Target State

This document closes the missing first step of the deslopify effort: define what
Assura should look like as a modern Rust project and as an agent-driven
repository, then compare the current repository against that target before
starting broad cleanup.

## Target-State Rubric

| Area | Target State | Deterministic Detection |
| --- | --- | --- |
| Rust package layout | Root manifest, lockfile, `src/`, `tests/`, `benches/`, `crates/`, and optional `examples/` follow Cargo conventions. Extra roots are intentional and documented. | Cargo metadata plus Assura root allowlist and directory rules. |
| Workspace governance | Workspace members, default members, package metadata, MSRV, feature gates, profiles, and publish settings are explicit. Internal support crates are visibly internal. | `cargo metadata` policy check; manifest semantics rule. |
| Module boundaries | Product surfaces are small and cohesive. Experimental/internal modules have ownership, support status, tests, and no accidental stable export claims. | Public export scan plus support-matrix rule plus module topology rule. |
| Public API and CLI surface | README, website, docs, CLI help, manifest metadata, and `src/lib.rs` agree on supported, experimental, internal, roadmap, and unsupported surfaces. | Existing command-surface docs constraint plus public-surface support-matrix rule. |
| Test architecture | Supported behavior has unit/integration/contract coverage at the right layer. Manual ignored tests are justified and not the only proof for supported behavior. | Test relationship rule using code imports, CLI fixtures, ignored-test scan, and support matrix. |
| Documentation architecture | Root docs are concise; durable analysis/goals are dated or archived; agent-facing instructions route to deeper skills/specs instead of duplicating stale detail. | Assura docs structure rules plus stale-doc/search-contract rules. |
| Build and local workflow | Frequent checks are cheap and scoped; full Rust, release, security, performance, and docs gates run only when matching surfaces changed or before merge. | `.assura/config.yml` quality scopes plus CI scope script. |
| Release/versioning | Version, MSRV, installer scripts, release notes, support policy, changelog, website install docs, and CI release workflows cannot drift. | Release-sync rule and package manifest semantics rule. |
| Dependency hygiene | Unused dependencies, security advisories, MSRV drift, native dependency surprises, and license/source policy are handled by appropriate tools. | `cargo machete`, `cargo audit`, future license/source policy, and manifest rule. |
| Performance evidence | Benchmarks and performance reports are tied to supported claims and current fixtures; historical reports are not treated as current claims. | Existing performance report workflow plus evidence verifier. |
| Agent workflow | Each non-steering task starts with deterministic state, has one active owner/task/branch, uses progressive disclosure, and never carries unclassified uncommitted work across tasks. | `workflow_gate.py`, task metadata validation, clean-start gate, and review-gate checks. |
| Assura-rule generalization | Repo-specific cleanup findings are classified into existing config, config extension, generalized Assura rules, external tools, or human review. | Backlog row requires a detector owner and proof command. |

## Current-State Comparison

| Area | Current State | Judgment | Needed Change |
| --- | --- | --- | --- |
| Rust package layout | Cargo layout is broadly conventional: root package, `src/`, `tests/`, `benches/`, and internal `crates/assura-check-cli`. | Mostly aligned. | Add explicit policy for internal workspace member metadata and purpose. |
| Workspace governance | Root crate has rich metadata. `assura-check-cli` is internal and `publish = []`, but lacks inherited or explicit license/description/MSRV metadata. | Partially aligned. | Add manifest semantics check for internal-vs-public crate metadata. |
| Module boundaries | `src/` is broad and about 43k lines. Several files cluster just under 500 lines, which suggests current limits are shaping files but not proving cohesion. Experimental `intelligence`, `maturity`, and broad `validation` APIs are contained but still publicly reachable through `src/lib.rs`. | Misaligned/contained. | Define desired module families and public export policy before deleting or splitting. Add public export/support scan. |
| CLI and public surface | Command-surface docs constraints now catch unsupported CLI claims. Support policy classifies experimental and roadmap surfaces. | Aligned but fragile. | Generalize into support-matrix rule that also reads manifest metadata and public exports. |
| Test architecture | Test coverage is broad. Some coverage is surface-focused, but there are manual ignored performance tests and experimental module tests that can keep abandoned APIs alive. | Partially aligned. | Map tests to supported surfaces and classify ignored/manual tests. |
| Documentation architecture | There is extensive dated analysis and goal history. This is useful, but the amount of active docs makes stale claims likely unless checks stay strict. | Partially aligned. | Require active/archive lifecycle and stale-claim detectors for docs that mention support, roadmap, performance, or release claims. |
| Build and local workflow | `.assura/config.yml` now defines quality scopes by file type, including lighter docs/workflow checks and heavier Rust/release/security/performance gates. | Aligned. | Keep measuring whether scope selection avoids unnecessary full Rust reruns. |
| Release/versioning | Release docs, support policy, installers, and workflow gates exist. Follow-up rule goals already identify release sync as a target. | Partially aligned. | Implement release-sync and manifest-semantics rules before treating release state as self-enforcing. |
| Dependency hygiene | `cargo machete` and `cargo audit` are adopted/deferred appropriately. License/source policy and semver checks are deferred for good reasons. | Aligned for current stage. | Add a planned license/source policy before adopting `cargo-deny`; add semver checks only after public API policy is stricter. |
| Performance evidence | Performance report command and website data exist, with a skill for performance changes. Historical evidence is present and sometimes dense. | Aligned but high-risk. | Ensure any performance claim references current generated evidence, not archived numbers. |
| Agent workflow | `AGENTS.md`, Trellis, skills, and `workflow_gate.py` now encode clean-start state. Recent user corrections show the state machine must be consulted every turn and dirty work must be resolved automatically when ownership is clear. | Aligned in tooling, needs enforcement habit. | Add deterministic workflow-state checks to planning/PR gates and keep root `AGENTS.md` concise. |
| Assura-rule generalization | Follow-up goals exist for command surface, manifest semantics, module topology, test relationship, release sync, and public surface support matrix. | Good backlog, not done. | Prioritize implementation by risk and reuse across repositories. |

## Repo Inventory

| Area | Inventory Evidence | Target-State Role | Current Classification |
| --- | --- | --- | --- |
| src | Rust library and CLI implementation, including `cli`, `config`, `constraints`, `intelligence`, `markdown`, `maturity`, and `validation`. | Product implementation plus contained internal/experimental modules. | contained/acceptable |
| crates | `crates/assura-check-cli` internal low-latency support binaries and daemon/client helpers. | Internal workspace member for performance and local-check evidence. | contained/acceptable |
| tests | CLI, LS-Lint, config, constraints, markdown, maturity, policy-language, performance-contract, and agent-feedback tests. | Supported surface proof and regression coverage. | aligned |
| benches | Criterion benchmarks and checked history under `benches/history`. | Performance evidence and claim governance. | aligned |
| docs | Active support, compatibility, release, validation, analysis, goals, archive, and proposal documents. | Human-facing source of truth, release claims, and durable planning evidence. | misaligned |
| .agents | Project skills and skill routing. | Progressive-disclosure operational guidance for agents. | aligned |
| .trellis | Canonical workflow, task, spec, and state scripts. | Agent workflow state machine and task continuity. | aligned |
| .github | PR template, issue templates, and CI/release/security workflows. | Hosted validation, release, and review evidence. | aligned |
| .assura | Structure config, command-surface contract, and local hooks. | Self-enforcement configuration and quality-scope source of truth. | aligned |
| release files | `Cargo.toml`, `Cargo.lock`, release notes, release checklist, install scripts, smoke scripts, and release workflow. | Version, install, release, and compatibility contract. | uncertain |
| website-facing claims | Website docs, install scripts, and checked performance data under `website/public`. | Public onboarding, release, and performance claim surface. | uncertain |

## Source-of-Truth Classification

| Surface | Classification | Evidence | Required Action |
| --- | --- | --- | --- |
| `assura check`, reports, and structure validation | aligned | Support policy, compatibility matrix, CLI tests, and self-check. | Keep command-surface and test relationship detectors active. |
| `assura quality plan` local workflow | aligned | Tooling spec, `.assura/config.yml`, `docs/validation.md`, and changed-check scripts. | Keep classified as supported local workflow. |
| `assura info` diagnostic CLI | uncertain | CLI and website docs expose it, but release notes did not previously classify it. | Classify as experimental diagnostic until release-grade tests/docs exist. |
| `src/intelligence`, `src/maturity`, broad `src/validation` exports | contained/acceptable | `src/lib.rs` unstable/internal markers and support policy. | Keep support-matrix detector; do not delete before test relationship audit. |
| Dense active docs/history | misaligned | Many active dated analysis files and release/performance claims. | Add docs lifecycle and stale-claim detectors before broad cleanup. |
| Manual ignored performance test | remove/refactor candidate | `tests/ls_lint_parity_regression_tests.rs` has an ignored manual audit fixture. | Keep as the only allowed ignored Rust test until it is replaced with automated benchmark evidence. |

## Priority Backlog

| Priority | Change | Detector Owner |
| --- | --- | --- |
| P0 | Produce a support matrix that joins CLI commands, docs claims, manifest metadata, and public Rust exports. | New generalized Assura public-surface support-matrix rule. |
| P0 | Add manifest semantics policy for workspace crates, including internal crate metadata, publish status, MSRV inheritance, and release metadata consistency. | New generalized Assura Cargo manifest semantics rule plus Cargo metadata. |
| P0 | Map supported product surfaces to required test families and classify ignored/manual tests. | New generalized test-relationship rule. |
| P1 | Define and enforce module topology for `src/cli/check`, `src/cli/performance_report`, experimental modules, and public re-exports. | Existing `.assura/config.yml` plus new module topology rule. |
| P1 | Add docs lifecycle checks for active analysis/goals versus archived/historical references and stale roadmap/performance claims. | Config tightening plus stale-claim custom constraint. |
| P1 | Implement release-sync checks across version, MSRV, installers, release notes, support docs, website install copy, and workflows. | New release-sync rule plus evidence verifier. |
| P2 | Evaluate license/source policy before `cargo-deny`; evaluate semver checks after public API support policy is stricter. | External tools with Assura quality-scope routing. |
| P2 | Consider splitting or renaming modules only after the support matrix distinguishes current product code from experimental/internal evidence code. | Human architecture review plus module topology reports. |

## Backlog And Detector Ownership

| Priority | Concrete Finding | Affected Files/Surfaces | Expected Target State | Remediation Action | Deterministic Detector | Owner |
| --- | --- | --- | --- | --- | --- | --- |
| P0 | CLI commands can exist without clear support classification. | `.assura/command-surface.yml`, `docs/support-policy.md`, `docs/compatibility-and-surface.md`, `docs/release-notes.md` | Every exposed command is supported, experimental, internal, roadmap, or unsupported. | Classify `assura quality plan` and `assura info`; reject future unclassified commands. | `cargo xtask target-state` command-surface support check | Assura support-matrix rule |
| P0 | Workspace member metadata can drift from release/MSRV policy. | `Cargo.toml`, `crates/assura-check-cli/Cargo.toml` | Root and internal workspace crates declare compatible version, MSRV, license, and publish policy. | Add internal crate metadata and check Cargo manifests. | `cargo xtask target-state` manifest semantics check | Cargo manifest semantics rule |
| P0 | Supported surfaces can lose test or source coverage evidence. | `tests/**`, `src/cli/**`, `crates/assura-check-cli/tests/**` | Supported CLI and workflow surfaces have mapped coverage markers. | Add relationship mapping for current supported surfaces. | `cargo xtask target-state` test relationship check | Test relationship rule |
| P0 | Release and performance claims can drift across docs, installers, and checked JSON. | Release docs, install scripts, `benches/history/current.json`, `website/public/data/performance/current.json` | Version, archive names, and current performance data agree. | Check release docs/scripts and compare performance JSON artifacts. | `cargo xtask target-state` docs/release/performance check | Release-sync rule |
| P0 | Agents can claim readiness while workflow state is ambiguous. | `.trellis/scripts/workflow_gate.py`, active task metadata, branch state | Active task, branch, and task artifacts are deterministic before target-state validation. | Run workflow gate JSON checks and keep final clean status as handoff evidence. | `cargo xtask target-state` workflow-state check plus `git status --short --branch` | Agent workflow gate |
| P1 | Active docs can accumulate stale roadmap or historical analysis claims. | `docs/analysis/**`, `docs/goals/**`, website docs | Active docs have lifecycle status and stale public claims are rejected. | Extend docs lifecycle detection beyond current command-surface patterns. | `cargo xtask evidence` plus future stale-claim custom constraint | Docs lifecycle rule |
| P1 | Module topology limits cap file size but do not prove cohesion. | `src/cli/check/**`, `src/cli/performance_report/**`, `src/{intelligence,maturity,validation}` | Module families have named ownership and public-support status. | Implement module topology rule before large refactors. | Future module-topology rule | Human architecture review |
| P2 | License/source policy is not defined. | Cargo dependencies and CI security workflow | Dependency source and license policy is explicit before adopting `cargo-deny`. | Draft policy, then wire external tool. | External `cargo-deny` only after policy | Security/tooling follow-up |

## Deterministic Detection Strategy

1. Build a repo inventory from Cargo metadata, file tree, `.assura/config.yml`,
   docs frontmatter, CLI help snapshots, public Rust exports, tests, benches,
   and workflow files.
2. Normalize each inventory item into a small set of surfaces: product CLI,
   internal performance tooling, Rust library API, docs/website, release,
   security, performance, workflow, fixtures, and experimental/roadmap.
3. Join surfaces against support status. Any public claim without a support row,
   or any support row without tests/docs, becomes a finding.
4. Classify the finding:
   - structure or naming issue: existing Assura config or config tightening;
   - semantic cross-file relationship: generalized Assura rule;
   - Rust-specific compiler/dependency issue: external Cargo tool;
   - architecture tradeoff: human review with detector evidence;
   - stale roadmap/doc issue: docs lifecycle or command-surface rule.
5. Require each remediation PR to state which detector will prevent regression.

## What This Changes About The Deslopify Plan

The correct next move is not another broad cleanup pass. The next move is to
turn this rubric into a checked audit artifact and then implement P0 detectors
in small PRs. Only after those detectors can distinguish product code from
contained experimental/internal code should broad module deletion or large
reorganization happen.

## Source Notes

- Cargo project layout, workspace, manifest, and test behavior are defined by
  the Cargo Book.
- Rustdoc documentation-test behavior is defined by the rustdoc book.
- Public Rust API review should use the Rust API Guidelines as a checklist, not
  as a blind mandate.
- Agent instruction routing should follow the AGENTS.md and Codex instruction
  model: concise operational root instructions, deeper skills/specs/scripts for
  progressive disclosure, and deterministic state scripts for workflow state.
