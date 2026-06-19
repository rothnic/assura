# Goal 12 Self-Enforcing Support And Test Matrix

## Goal

Turn Assura's current support policy, public command surface, manifest metadata,
and test-coverage expectations into deterministic `cargo xtask target-state`
checks that prevent public-surface drift before release or onboarding docs can
claim unsupported behavior.

## What I Already Know

- Goal 12 is the next planned Iteration 02 goal after Goal 11 merged in PR #53.
- `cargo xtask target-state` already exists and runs audit, command-surface,
  manifest, test-relationship, docs/release/performance, workflow-state, root
  tooling, and lint-suppression checks.
- The target-state audit identifies P0 detector owners for command support,
  manifest semantics, test relationships, release/performance drift, and agent
  workflow state.
- Existing planned rule docs should be reused: public-surface support matrix,
  Cargo manifest semantics, and test relationship.
- Goal 12 should classify experimental/internal surfaces instead of deleting
  contained evidence modules prematurely.

## Revalidation Result

`valid`: live repository state still has the support/test matrix gap described
by the goal. The repo has partial target-state checks, but not yet a single
matrix artifact that joins CLI commands, support status, docs claims, manifest
metadata, public Rust exports, and required test families with deterministic
blocking findings.

## Scope

- Extend the existing `cargo xtask target-state` verifier instead of adding a
  parallel command.
- Model a support matrix for current command families, public/internal Rust
  exports, workspace manifest metadata, docs claims, and required test
  evidence.
- Add fixtures or deterministic in-repo examples that prove stale public claims,
  unclassified surfaces, and missing test relationships fail.
- Preserve explicit exceptions for experimental/internal/roadmap surfaces.
- Add a matrix review artifact that maps Goal 12 review tasks to evidence.
- Add a stale-goal creation/revalidation hook or checked documentation path for
  future old/separate-context goals.

## Non-Goals

- No broad module deletion or API cleanup before the matrix classifies product
  versus internal/evidence code.
- No 1.0 semver commitment.
- No semver checker rollout.
- No license/source policy rollout beyond bounded manifest placeholders needed
  for the current matrix.
- No separate support-matrix CLI if `cargo xtask target-state` can carry the
  proof gate.

## Acceptance Criteria

- [ ] Every current public command family has a support classification or an
      explicit exception.
- [ ] Public docs and release notes cannot claim unsupported commands, formats,
      dependency-graph validation, maturity detection, or equivalent roadmap
      surfaces as supported.
- [ ] Workspace manifest metadata follows the selected public/internal crate
      policy.
- [ ] Supported surfaces map to required test families or an accepted exception.
- [ ] `cargo xtask target-state` reports matrix failures deterministically.
- [ ] The PR links Goal 12 and a matrix review artifact.
- [ ] Future goal docs have a deterministic creation/revalidation route through
      `assura-goal-validation` before execution.

## Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo xtask target-state
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm existing planned rule docs are reused instead of duplicated.
- R1: Review support classifications against release notes and support policy.
- R2: Review manifest metadata checks for internal versus public crates.
- R3: Review test-relationship checks and accepted exceptions.
- R4: Confirm contained experimental modules are classified, not accidentally
  promoted.
- R5: Confirm the PR links this goal and a matrix review artifact.
- R6: Confirm old or separate-context goal docs have a revalidation route
  through `assura-goal-validation`.

## Technical Notes

- Existing verifier entrypoint: `xtask/src/main.rs::run_target_state`.
- Current docs sources: `docs/support-policy.md`,
  `docs/compatibility-and-surface.md`, `docs/release-notes.md`, and
  `.assura/command-surface.yml`.
- Target-state source: `docs/analysis/2026-06-09-assura-best-practice-target-state.md`.
- Related planned rule docs:
  `docs/goals/assura-rule-public-surface-support-matrix.md`,
  `docs/goals/assura-rule-cargo-manifest-semantics.md`, and
  `docs/goals/assura-rule-test-relationship.md`.
