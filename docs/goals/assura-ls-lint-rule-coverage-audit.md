---
id: goal-assura-ls-lint-rule-coverage-audit
type: goal
title: Assura LS-Lint Rule Coverage Audit
status: completed
created: 2026-05-26
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/goals/assura-cli-to-cli-ls-lint-performance-verification.md
  - docs/goals/assura-real-project-policy-proof.md
  - docs/goals/assura-agent-nudge-mvp.md
  - docs/analysis/2026-05-15-notation-source-truth.md
  - docs/ls-lint-capability-comparison.md
  - src/config/ls_compat.rs
  - src/cli/check/
  - tests/ls_lint_parity_regression_tests.rs
  - tests/realistic_lslint_fixtures.rs
---

# Assura LS-Lint Rule Coverage Audit

## Objective

Verify, from upstream LS-Lint docs and source tests, whether Assura has complete
coverage for the LS-Lint rule surface that matters to migration, parity claims,
and the agentic real-time feedback hot path.

This goal must produce a concrete coverage matrix and close implementation
gaps for LS-Lint 2.3 rule compatibility. Do not reduce or soften public claims
where Assura can instead meet them through implementation.

## Why This Matters

Assura is positioning around agentic coding feedback. Agents need fast,
trustworthy structure feedback while they are still in the same implementation
turn. If Assura claims LS-Lint compatibility, migrated rules must behave
predictably, especially for regex rules, `exists` rules, wildcard extensions,
and directory scopes. Silent semantic drift would create bad nudges and erode
trust in the hot path.

This work also informs the planned Assura-native `rules:` directive. `rules:`
is intended to become a reusable named-rule grouping model that improves on
LS-Lint duplication. Before designing that replacement, we need a precise map
of which LS-Lint semantics we are preserving, extending, or deliberately
leaving behind.

## Required Upstream Baseline

The first implementation step must pin the upstream sources used for the audit:

- LS-Lint docs version reviewed.
- LS-Lint repository URL and commit SHA reviewed.
- npm package version used for live behavior checks.
- exact upstream test files inspected.

At minimum, inspect:

- `internal/rule/regex_test.go`
- `internal/rule/exists_test.go`
- `internal/rule/lowercase_test.go`
- `internal/rule/camelcase_test.go`
- `internal/rule/pascalcase_test.go`
- `internal/rule/snakecase_test.go`
- `internal/rule/screamingsnakecase_test.go`
- `internal/rule/kebabcase_test.go`
- `internal/config/config_test.go`
- `internal/linter/linter_test.go`
- `internal/glob/glob.go`
- `cmd/ls_lint/main.go`

## Rule Coverage Matrix

Create a durable analysis artifact:

`docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md`

The matrix must include one row per LS-Lint behavior, with these columns:

- LS-Lint behavior.
- Upstream evidence: docs URL, source file, test name, or live command.
- Current Assura support status: supported, partially supported, unsupported,
  Assura extension, or intentionally out of scope.
- Existing Assura tests, if any.
- Missing Assura tests.
- Required action.
- Agentic hot-path relevance.

The matrix must cover:

- `lowercase`
- `camelcase` / `camelCase`
- `pascalcase` / `PascalCase`
- `snakecase` / `snake_case`
- `screamingsnakecase` / `SCREAMING_SNAKE_CASE`
- `kebabcase` / `kebab-case`
- `regex`
- regex full-string anchoring
- regex negation
- regex directory substitutions using `${0}`, `${1}`, and deeper ancestors
- multiple regex rules with `|`
- regex alternation inside one pattern
- `exists`
- bare `exists`
- `exists:0`
- `exists:N`
- `exists:N-M`
- invalid exists syntax
- directory `exists` through `.dir`
- direct-child-only exists semantics
- targeted file and directory runs
- wildcard extensions such as `.*`, `.*.js`, and `.*.*.go`
- sub-extension precedence such as `.test.ts` over `.ts`
- `.dir` inheritance and override behavior
- glob directory scopes using `*` and `**`
- brace directory scopes using `{a,b}`
- glob and brace ignore patterns
- multiple `--config` merge behavior
- `--workdir`
- `--error-output-format json`
- `--warn`

## Implementation Requirements

### 1. Add Missing Parity Tests

Add tests only after the matrix identifies the gap. Prefer focused regression
tests over broad fixture rewrites.

Required test categories:

- regex anchoring and regex literal semantics
- regex negation
- regex directory substitution
- multiple regex rules
- exists parser validation
- bare exists and range exists
- directory exists
- direct-child-only count behavior
- wildcard extension precedence
- case-convention edge cases from upstream tests

If a behavior is not part of LS-Lint's rule/config compatibility surface, label
it as outside the compatibility claim. LS-Lint rule behavior itself should be
implemented rather than silently rejected.

### 2. Live Behavior Checks

For ambiguous behaviors, run the pinned LS-Lint binary and Assura against the
same temporary fixture.

Use the native LS-Lint binary from `@ls-lint/ls-lint@2.3.0` when possible, not
only the npm package wrapper.

The audit must record:

- fixture tree
- `.ls-lint.yml`
- generated or equivalent `.assura/config.yml`
- LS-Lint output and exit status
- Assura output and exit status
- interpretation

### 3. Migration Classification

For each LS-Lint behavior that is not yet implemented during the audit,
classify the migration stance:

- **support now**: needed for credible migration or agent hot-path feedback
- **support later**: valuable but not required for the next agentic slice
- **reject clearly**: Assura should not support it, but migration must fail
  with a precise diagnostic
- **Assura-native replacement**: should be expressed through the planned
  `rules:` model instead of reproducing LS-Lint syntax directly

Current resolved candidates:

- Glob and brace directory scopes must be supported as validation scopes, not
  required literal child directories.
- Regex negation and directory substitutions are documented LS-Lint 2.3
  behavior and must be supported by migrated configs.
- Exact filename `exists` is an Assura extension, not native LS-Lint parity.
  Keep it labeled that way.

### 4. Agentic Feedback Implications

For every missing or deferred behavior, explain the effect on agentic coding:

- Could a migrated config produce false positives?
- Could a migrated config miss violations?
- Would a nudge tell the agent to make the wrong change?
- Would performance of the real-time hot path be affected?
- Does the behavior need a specialized fast path?

This section should directly inform the later agentic feedback and nudge goals.

## Non-Goals

This goal does not require:

- implementing the full agent nudge publishing flow
- redesigning the website
- replacing LS-Lint syntax with Assura `rules:` in the same PR
- changing Assura-native `rules:` syntax in the same PR
- matching LS-Lint behavior silently without regression evidence

## Completion Criteria

This goal is complete only when:

- the coverage matrix exists and is linked from this goal
- upstream LS-Lint docs and source commit are recorded
- every upstream rule test category has a mapped Assura status
- missing tests for intended parity are added
- rule compatibility gaps identified by the matrix are implemented or proven
  outside the LS-Lint 2.3 compatibility claim
- exact filename `exists` remains labeled as an Assura extension
- docs and website claims are corrected if the audit changes the support story
- `rules:` follow-up requirements are updated with any reusable-grouping
  implications discovered during the audit

## Validation Commands

Run the relevant subset during implementation and the full set before completion:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test ls_lint_parity_regression_tests --quiet
cargo test --test cli_check_tests --quiet
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
```

When live LS-Lint comparison is needed:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json
```

Website or docs claim changes must also run:

```bash
cd website
pnpm build
```

## Review Requirements

Before opening a PR for this goal, run a review agent with this scope:

- verify the upstream LS-Lint source/test mapping
- verify each claimed supported behavior has an Assura test
- verify unsupported behavior fails clearly
- verify docs do not overstate compatibility
- verify agentic hot-path implications are captured

The review output should be saved under:

`docs/analysis/2026-05-26-ls-lint-rule-coverage-review.md`

## Suggested First Slice

Start with regex and exists because they are the highest-risk semantic areas:

1. Pin upstream LS-Lint source and package versions.
2. Build the matrix rows for regex and exists.
3. Add missing tests for regex anchoring, negation, substitutions, bare exists,
   range exists, invalid exists syntax, and directory exists.
4. Decide which gaps are support-now versus explicit-reject.
5. Update docs and claims only after tests prove the behavior.

## Progress Log

| Date | Progress | Evidence |
| --- | --- | --- |
| 2026-05-26 | Pinned LS-Lint 2.3 upstream docs, package, and source commit; reviewed rule/config/linter/glob tests. | `/tmp/assura-ls-lint-upstream` at `49b4e7b`; `npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version` |
| 2026-05-26 | Implemented LS-Lint regex anchoring, negation, directory substitutions, exists parser validation, case-rule edge behavior, wildcard extension precedence, and exact extension-combination matching. | `src/config/ls_compat.rs`; `src/cli/check/case.rs`; `src/cli/check/patterns.rs`; `tests/ls_lint_parity_regression_tests.rs` |
| 2026-05-26 | Implemented LS-Lint glob and brace directory scopes as matcher-backed validation scopes instead of literal required directories. | `src/cli/check/rule_plan.rs`; `src/cli/check/rules.rs`; `converted_lslint_directory_pattern_scopes_*` |
| 2026-05-26 | Added advisory `assura check --warn` and LS-Lint multi-config merge conversion. | `src/cli/check/fast_cli.rs`; `src/cli/commands.rs`; `test_convert_multiple_ls_lint_configs_merges_like_config_flags`; `check_warn_reports_violations_but_exits_successfully` |
| 2026-05-26 | Created the coverage matrix and live LS-Lint-vs-Assura comparison artifact. | `docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md` |
| 2026-05-26 | Resolved independent review findings for `.dir` self-scope semantics, missing-scope `exists`, canonical LS-Lint case aliases, and raw regex alternation in the fast path. | `docs/analysis/2026-05-26-ls-lint-rule-coverage-review.md`; `tests/ls_lint_rule_coverage_tests.rs` |
| 2026-05-26 | Added explicit missing-test accounting to the matrix and closed residual evidence gaps for converted glob/brace ignores and multi-config CLI migration. | `docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md`; `converted_lslint_glob_and_brace_ignore_patterns_exclude_matches`; `cli_migrate_accepts_multiple_lslint_configs_in_merge_order` |
| 2026-05-26 | Completed full verification for Rust, Assura self-check, website build, stale claim search, and live LS-Lint comparison. | `cargo test --all-targets --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo run --quiet -- check --format json .`; `npm exec --yes --package pnpm@10.25.0 -- pnpm build` |
