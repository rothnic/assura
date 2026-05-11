# LS-Lint Parity Performance Regression Audit

## Goal

Establish a trustworthy, narrow audit of Assura's LS-Lint-compatible structure behavior after PRs #5 and #6. The task should confirm the current clean baseline, expand or improve regression coverage where it makes the audit more reliable, measure representative performance risk areas, and publish an analysis report that identifies correctness gaps, performance risks, and recommended follow-up implementation tasks.

## What I Already Know

- PR #5 merged the Assura self-check baseline cleanup.
- PR #6 merged closed-world project structure enforcement, LS-Lint compatibility improvements, and the `integrations/agents/` layout.
- `master` now includes `b8febab fix(config): remove stale docs performance policy` after PR #6.
- The prior completed LS-Lint structure task has been archived before this task started.
- This task is intentionally limited to analysis, regression coverage, and performance gap identification.

## Requirements

- Confirm the baseline health on the new branch:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --quiet`
  - `cargo run --quiet -- check --format json .`
- Audit Assura behavior against LS-Lint 2.3 basics:
  - extension rules
  - wildcard extension rules
  - `.dir`
  - nested path rules
  - OR syntax
  - `exists`, `exists:0`, `exists:1`, `exists:N-M`
  - ignore/exclude behavior
  - direct child vs recursive semantics
- Build or improve regression fixtures comparing intended LS-Lint-compatible behavior against Assura behavior.
- Run performance testing on representative synthetic repo shapes:
  - small, medium, large
  - deep tree
  - wide tree
  - many ignored/generated directories
  - many direct-content checks
  - many wildcard/extension/path rules
- If LS-Lint is locally available, compare Assura against equivalent fixtures. If it is not available, document the missing tool and still produce Assura baseline metrics.
- Fix only narrow correctness or measurement issues that block trustworthy audit results.
- Produce a `docs/analysis/` report with a parity matrix, regression fixture coverage, performance results, correctness gaps, performance risks, and recommended next implementation tasks.
- Update Trellis specs only if the audit establishes durable contracts or policy.
- Run final checks:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-targets --quiet`
  - `cargo run -- check .`
- Commit, push, and open a focused PR.

## Acceptance Criteria

- [x] Baseline health commands are run and recorded.
- [x] LS-Lint parity behavior is documented in a matrix.
- [x] Regression fixtures exist or are improved for each audited behavior class where practical.
- [x] Performance measurements cover the requested synthetic repo shapes and rule-heavy cases.
- [x] LS-Lint availability and any direct comparison result are documented.
- [x] Only narrow audit-enabling fixes are included.
- [x] A `docs/analysis/` report captures findings and recommended follow-up tasks.
- [x] Trellis specs are updated if durable contracts or policy changed.
- [x] Final checks pass or any blocker is documented with evidence.
- [ ] The branch is pushed and a focused PR targets `master`.

## Out Of Scope

- Broad runtime rewrites.
- Windows CI restoration.
- Codex runtime hook implementation.
- Changing hook blocking policy.
- Turning performance findings into full optimization work.
- Expanding product scope beyond LS-Lint parity/regression/performance analysis.

## Technical Notes

- Required specs read before task creation:
  - `.trellis/workflow.md`
  - `.trellis/spec/assura/roadmap.md`
  - `.trellis/spec/assura/workflow-status.md`
  - `.trellis/spec/assura/tooling-stabilization.md`
  - `.trellis/spec/assura/structure-enforcement.md`
- Branch: `codex/ls-lint-parity-performance-regression-audit`
- Base branch: `master`
