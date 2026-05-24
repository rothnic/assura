# Pinned LS-Lint Fixture Benchmark Suite

## Goal

Implement the next performance-evidence slice from
`docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md`: add meaningful
LS-Lint benchmark fixtures, preserve machine-readable metadata through
performance rows, and keep website performance docs split between concise
summary, test-case definitions, and implementation detail.

## What I Already Know

- The performance report already emits `assura-cli`, `ls-lint-cli`,
  `assura-in-process`, phase, traversal, and strategy row families.
- Headline public evidence should use only native LS-Lint parity
  `assura-cli` and `ls-lint-cli` rows with `diagnostic=false`.
- Current generated realistic-equivalent cases exist, but `rule_heavy_repo` is
  still mechanically extension-heavy rather than policy-shaped.
- `tests/ls_lint_realistic_fixture_manifest.yml` includes one external mdBook
  materialization fixture, but `assura performance-report` does not measure
  pinned external repository fixtures yet.
- `website/src/content/docs/reference/performance-test-cases.mdx` is already
  structured as the source of truth for test-case definitions and target
  real-project cases.
- LS-Lint 2.3 supports directory-specific overrides, glob and alternative
  directory patterns, wildcard/subextensions, regex rules, multiple rules with
  `|`, and direct `exists` checks for files or directories.

## Requirements

- Add a generated rich monorepo-policy fixture that includes root whitelisting,
  well-known docs, apps/packages scopes, config-file exceptions, source-tree
  extension bans, docs/scripts/infra policies, and generated-output ignores.
- Add at least two pinned external repository fixtures to performance-report
  measurement, or document exact blockers and a deterministic opt-in command if
  they are too large or flaky for default runs.
- Preserve source revision, source type, counts, rule count, config references,
  native parity, and headline/diagnostic classification in report rows.
- Refresh checked-in benchmark and website performance data when the target
  artifact shape is valid.
- Keep `/reference/performance/` concise and place fixture policy detail on
  `/reference/performance-test-cases/`.
- Add focused tests for manifest entries, materialization/revision handling,
  row metadata, classification, report schema acceptance, and website build
  stability.

## Acceptance Criteria

- [ ] Generated rich monorepo-policy fixture is implemented and measured.
- [ ] Pinned external frontend/monorepo fixture is implemented and measured, or
      has a precise documented opt-in/blocker.
- [ ] Pinned external Rust docs/library fixture is implemented and measured, or
      has a precise documented opt-in/blocker.
- [ ] New fixture rows carry complete metadata and correct
      `headline-candidate` versus `diagnostic` classification.
- [ ] Performance docs link summary rows to test-case definitions without
      embedding full LS-Lint configs in the summary page.
- [ ] Goal validation commands are run or any unavailable/too-slow command is
      documented with exact evidence.

## Out Of Scope

- Changing Assura traversal architecture unless a fixture exposes a correctness
  or performance blocker.
- Cold package install or dependency-resolution measurement.
- New Assura-only notation in native LS-Lint parity headline rows.

## Technical Notes

- Use `.agents/skills/assura-performance-reporting/SKILL.md` for report
  refresh workflow.
- Relevant implementation files include
  `src/cli/performance_report/fixtures.rs`,
  `src/cli/performance_report/fixture_metadata.rs`,
  `src/cli/performance_report/fixture_io.rs`, and report row/schema modules.
- Relevant tests include `tests/ls_lint_parity_regression_tests.rs`,
  `tests/realistic_lslint_fixtures.rs`, and the fixture manifest.
- Upstream LS-Lint docs reviewed:
  <https://ls-lint.org/2.3/configuration/the-basics.html> and
  <https://ls-lint.org/2.3/configuration/the-rules.html>.
