# Repair Claim And Release Contract

## Goal

Make claim eligibility, support maturity, current-release availability, and
cross-platform behavioral evidence one enforceable contract.

## Requirements

- Fix the Windows LS-Lint golden fixture without weakening Unix coverage.
- Reject marketing claims mapped to anything except `supported` surfaces.
- Reject a production claim when its surface is absent from the promoted
  release contract.
- Reconcile release-surface, support-policy, command-surface, roadmap, and task
  status so they cannot report contradictory maturity.
- Keep preview builds useful while making production/release gates strict.

## Acceptance Criteria

- [x] The four failing Windows LS-Lint tests pass on Windows and Unix.
- [x] Experimental/planned marketing mappings fail with the surface ID.
- [x] Promoted-release availability is machine-checked before publication.
- [x] Existing 12 marketing claims remain supported with verified/measured evidence.
- [x] Stale task and roadmap status is reconciled or linked to an explicit open task.

## Independent Review

The 2026-08-11 review found publication bypass, incomplete SemVer handling,
prose-based goal ownership, a writable strict mode, incomplete negative tests,
and a circular parent criterion. All six findings are addressed. Hosted
Windows CI run `31539326739` passed every native LS-Lint golden case. The same
job exposed a separate daemon status race, now owned by the support-grade watch
and warm-runtime child task.

## Validation

```bash
cargo test --test ls_lint_rule_coverage_tests
cargo xtask website-demo-data --check
cargo xtask target-state
cargo xtask evidence
cargo xtask release-readiness --format json
pnpm --dir website build
git diff --check
```

## Review Blocking Criteria

Block if platform normalization hides real path differences, previews cannot be
built, production can publish unreleased claims, or separate manifests can
silently disagree.
