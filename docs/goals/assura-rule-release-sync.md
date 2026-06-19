---
id: goal-assura-rule-release-sync
type: goal
title: Assura release sync rule
status: completed
created: 2026-06-08
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-13-performance-and-release-evidence-governance.md
  - docs/analysis/2026-06-19-goal-13-release-performance-review.md
  - .trellis/spec/assura/roadmap.md
  - xtask/src/main.rs
---

# Assura Release Sync Rule

## Objective

Create a reusable release-contract rule family that verifies configured release
docs, install scripts, workflow asset names, support matrix rows, checksum
sidecars, and package metadata stay synchronized.

This should generalize the current Assura repo release checks without
duplicating the hard-coded `cargo xtask target-state` governance that now
protects this repository.

## Revalidation Result

`valid`, with narrowed scope.

Goal 13, merged in PR #55, already made Assura's own release/performance claims
deterministic through `cargo xtask target-state`. That current repo-local check
covers public archive names, installer archive references, CI install smoke
labels, checksum sidecars, release docs, website install copy, performance JSON
freshness, and public performance copy.

The remaining gap is not another Assura-only target-state assertion. The valid
next product question is whether Assura should offer a configurable release
contract rule family that other repositories can use for the same class of
drift.

## User Certainty Bar

A repository maintainer should be able to declare the release artifacts their
project publishes and have Assura report drift when docs, installers, workflow
matrices, checksums, or package metadata stop agreeing.

## Completion Result

Completed as a first reusable rule slice in PR #59 and archived in PR #60.
The shipped surface adds explicit `extensions.release_contracts` notation,
semantic config validation, workflow/docs/installer/checksum drift checks,
installer URL branch and asset checks, CLI JSON diagnostics, compiled-config
portability, and integration fixtures.

The first slice intentionally keeps Assura's own hard-coded
`cargo xtask target-state` release checks. Package metadata and support-matrix
joins remain better follow-up owners for the manifest-semantics and
public-surface support-matrix rules.

## Starting Boundary

- Assura repo release governance: covered by `cargo xtask target-state`.
- Public reusable Assura validation: implemented for the first explicit
  release-contract slice.
- Release-contract configuration notation: specified in this goal and
  represented in `extensions.release_contracts`.
- Fixture and CLI integration coverage for release-contract drift: present.

## Detector Hypothesis

Extract archive names, checksums, install URLs, version mentions, and release
workflow matrix entries from configured files, then compare them against a
single release contract.

The first implementation should prefer an explicit configured contract over
inferring release intent from arbitrary prose. A future rule can add inference
only after the configured contract is stable and tested.

## First Slice Config Notation

The first implementation slice uses an explicit first-party extension entry
instead of a custom prose inference layer:

```yaml
extensions:
  release_contracts:
    - id: cli_release
      severity: high
      artifacts:
        - name: example-linux-x86_64.tar.gz
          checksum_sidecar: true
        - name: example-darwin-aarch64.tar.gz
          checksum_sidecar: true
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
      installer_files:
        - scripts/install.sh
      allowed_url_branches:
        - main
```

The contract is deliberately explicit:

- `artifacts` is the allowed publish set.
- `checksum_sidecar: true` requires `<artifact>.sha256` in workflows and
  configured docs/installers.
- `workflow_files` are checked for configured artifact and sidecar mentions.
- `docs_files` and `installer_files` are checked for artifact names outside
  the contract.
- `allowed_url_branches` constrains raw/blob installer URLs.

## Scope

- Define a minimal release-contract config surface for archive names, optional
  checksum sidecars, installer files, workflow files, and docs files.
- Implement validation that reports:
  - docs or installers mention an artifact absent from the contract;
  - the contract names an artifact absent from configured workflow matrices;
  - checksum sidecar expectations are missing from docs or workflows;
  - installer URLs point at unsupported branches or asset names.
- Add passing and failing fixtures that are independent of Assura's own release
  docs.
- Add CLI integration coverage proving the rule runs through `assura check`.
- Keep Assura's own hard-coded target-state release checks until the reusable
  rule proves it can replace them without losing coverage.

## Non-Goals

- No release publishing automation.
- No GitHub API access or remote release inspection.
- No broad natural-language parsing of arbitrary release prose.
- No removal of current `cargo xtask target-state` release checks in the first
  implementation slice.

## Definition Of Done

- Release-contract config notation is documented in the goal or spec before
  implementation.
- Passing fixture covers a complete configured release contract.
- Failing fixtures cover workflow/doc mismatch, installer asset mismatch,
  checksum sidecar drift, and unsupported branch/asset URLs.
- `assura check --format json` reports actionable release-contract violations
  with file and contract context.
- Assura repo target-state checks remain green and are not weakened.
- Independent review confirms this rule adds reusable product coverage rather
  than duplicating repo-local `xtask` governance.

## Required Examples

- Passing: all platform archive names match the release matrix.
- Failing: docs mention an asset that release workflow does not publish.
- Failing: install script URL points at an unsupported branch or asset name.
- Failing: workflow uploads an archive but omits its required `.sha256`
  sidecar.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo xtask target-state
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm the implementation does not weaken current Goal 13
  target-state release/performance checks.
- R1: Review release-contract notation for explicitness and avoid hidden prose
  inference.
- R2: Review passing/failing fixtures for docs, installer, workflow, checksum,
  and package metadata drift.
- R3: Review JSON diagnostics for actionable file and contract context.
- R4: Confirm the rule is reusable outside the Assura repository.

## Reviewer Blocking Criteria

Block the PR if the implementation only moves hard-coded Assura release checks
into product code, if release intent is inferred from arbitrary prose without a
configured contract, if checksum sidecars are not covered, if fixtures do not
exercise both docs and workflow drift, or if current `cargo xtask target-state`
coverage is weakened before the reusable rule proves equivalent coverage.

## Progress Log

- 2026-06-19: Revalidated after Goal 13 merged in PR #55 and the roadmap
  handoff merged in PR #57. Result: valid with narrowed scope. Goal 13 covers
  Assura's own release/performance drift through target-state checks; this
  goal remains as a reusable configured release-contract rule candidate.
- 2026-06-19: Started the first implementation slice under Trellis task
  `06-18-release-contract-rule-slice`. Added explicit
  `extensions.release_contracts` notation, product validation for workflow,
  docs, installer URL, and checksum sidecar drift, plus integration coverage.
  Local gates passed before review: `cargo test --all-targets --quiet`,
  `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo xtask target-state`, `cargo run --quiet -- check --format json .`,
  `cargo xtask evidence`, `cargo xtask docs`, and `git diff --check`.
- 2026-06-19: Independent review found release contracts were runtime-validated
  but not included in config semantic validation. Added semantic checks for
  release-contract ids, duplicate contracts, artifacts, paths, branch allowlist
  values, and severity; split the validator into a child module to keep
  structure policy green. Re-ran the full local gate set successfully.
- 2026-06-19: Addressed PR review feedback on release-contract validation by
  scanning workflow/docs/installer files directly instead of joining large
  content buffers, guarding substring checks against empty search terms, and
  hardening install URL extraction for shell assignments and Markdown links.
- 2026-06-19: Completed the release-contract first slice in PR #59 and archived
  Trellis task `06-18-release-contract-rule-slice` in PR #60. Remaining package
  metadata and support-policy joins route to manifest-semantics and
  public-surface support-matrix follow-up goals.
