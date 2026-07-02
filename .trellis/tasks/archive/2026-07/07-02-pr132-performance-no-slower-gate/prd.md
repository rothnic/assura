# Fix PR 132 no-slower performance gate

## Objective

Restore the PR #132 `Performance Report` CI job without weakening the strict
accepted-fixture no-slower policy.

## Context

The hosted report for PR #132 failed only on
`many_configured_scopes_regression`: `assura-cli` measured `17.593805ms` and
`ls-lint-cli` measured `17.569939ms`.

## Requirements

- Keep `cargo xtask performance-no-slower` strict for every accepted fixture.
- Do not mark the fixture diagnostic or add tolerance to hide the failure.
- Fix the Assura path or measurement path so the accepted fixture is no slower
  than LS-Lint.
- Re-run the local no-slower gate against a fresh performance report.
- Push the fix to PR #132 and merge only after hosted checks pass.

## Verification

- `target/release/assura performance-report --output target/performance/ls-lint-comparison.json --iterations 5`
- `cargo xtask performance-no-slower target/performance/ls-lint-comparison.json`
- PR #132 hosted `Performance Report` passes.
