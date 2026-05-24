---
id: analysis-2026-05-15-ls-lint-good-enough-comparison-contract
type: analysis
title: LS-Lint good-enough comparison contract
status: active
created: 2026-05-15
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - tests/ls_lint_realistic_fixture_manifest.yml
  - tests/realistic_lslint_fixtures.rs
---

# LS-Lint Good-Enough Comparison Contract

This document defines the comparison contract used by LS-Lint compatibility
tests, benchmarks, CI artifacts, PR review, and website reporting. It is the
current source of truth for what counts as "good enough" evidence while Assura
is still pre-1.0.

## Fixture Source Contract

The stable fixture corpus is declared in
`tests/ls_lint_realistic_fixture_manifest.yml`.

Every fixture entry must record:

- `id`: stable fixture identifier used by tests, benchmarks, and performance
  history.
- `name`: human-readable fixture name.
- `source.kind`: `generated` for in-repo deterministic generators or
  `external_git` for downloaded external projects.
- `source.repository`: immutable upstream repository URL for external entries,
  or this repository for generated entries.
- `source.revision`: commit SHA, release tag, or explicit generated fixture
  version. Stable baseline revisions must not change silently.
- `purpose`: why the fixture exists.
- `ls_lint_rules`: LS-Lint rule surface covered.
- `assura_rules`: equivalent Assura structure-first rule surface covered.
- `cohort`: `stable_baseline` or `feature_<name>`.
- `native_lslint_parity`: true only for behavior supported by LS-Lint 2.3.
- `assura_extensions`: Assura-only compatibility extensions, such as exact
  filename `exists`.

Generated fixtures may stay in Rust code when they model narrow compatibility
or stress cases more clearly than a third-party project. External-real-repo
fixtures must be materialized from the manifest instead of vendoring whole
repositories into this source tree.

## Harness Contract

The reusable fixture harness in `tests/realistic_lslint_fixtures.rs` must be the
shared source for realistic fixture families used by integration tests and
benchmarks. One-off temp-directory setup is still acceptable for narrow
regressions, but it must not replace reusable realistic families.

External git entries are materialized by the test harness through
`materialize_external_git_fixture`, which clones or fetches into a cache keyed
by `(repository, revision)`, checks out the pinned revision, and copies the
working tree without `.git` metadata into the requested fixture directory. CI
can restore that cache, but the pinned manifest remains the source of truth.

## Compatibility Pass/Fail Contract

For native LS-Lint parity fixtures:

- valid fixture variants must pass Assura and LS-Lint with equivalent rules;
- invalid fixture variants must fail both tools where LS-Lint supports the
  behavior;
- tests must assert expected Assura violation rule names, not only exit code;
- ignored files and directories must not produce Assura violations;
- validation scopes must not imply required directories unless an explicit
  existence or required-directory rule requires them;
- unsupported directory pattern scopes such as `packages/*`, `**`, and
  `{src,tests}` must return clear errors until implemented fully.

Assura-only extensions must be labeled separately and excluded from native
LS-Lint parity claims. Exact filename `exists` remains an Assura compatibility
extension because live LS-Lint 2.3 does not treat `README.md: exists:1` as an
exact direct filename count.

## Performance Pass/Fail Contract

Every PR that changes checker behavior or fixture generation must produce a
machine-readable comparison artifact for stable baseline fixtures. The first
tracked artifact format must include:

- schema version;
- timestamp;
- branch and commit SHA;
- OS and toolchain details;
- Assura version or binary path;
- LS-Lint version;
- fixture id, source revision, and cohort;
- rule cohort;
- tool name;
- median runtime and distribution details when available;
- pass/fail status;
- comparison baseline id.

Stable baseline fixtures must not regress beyond the agreed threshold recorded
with the benchmark runner or PR artifact. New feature fixtures establish a new
baseline and must not weaken old baseline expectations. Assura and LS-Lint must
be compared against equivalent rules on the same materialized fixture tree.

## Baseline Update Process

Updating stable fixtures or performance baselines requires an explicit PR
change that:

1. Updates the manifest entry or result history.
2. Explains why the baseline changed.
3. Preserves old history so longitudinal comparison remains possible.
4. Labels new feature coverage as a feature cohort instead of rewriting stable
   baseline fixture meaning.

Website reporting may publish generated summaries after merge, but PR review
must still expose either the current artifact or a preview link before merge.
