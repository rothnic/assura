---
id: analysis-2026-05-26-ls-lint-rule-coverage-review
type: review
title: LS-Lint rule coverage implementation review
status: resolved
created: 2026-05-26
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md
  - docs/goals/assura-ls-lint-rule-coverage-audit.md
  - tests/ls_lint_rule_coverage_tests.rs
---

# LS-Lint Rule Coverage Implementation Review

Independent review was run against the uncommitted LS-Lint compatibility
implementation. The review found four blocking parity gaps and one non-blocking
native-notation documentation issue.

## Blocking Findings Resolved

1. `.dir` was being represented as a direct-child directory policy instead of
   a rule for the indexed directory itself.
   - Resolution: migrated `.dir` rules now use `self_directory` on
     `DirectoryNode`; runtime validation checks the directory represented by
     the matching scope.
   - Evidence: `converted_lslint_dir_rule_validates_scoped_directory_itself`.

2. `exists` rules under missing migrated scopes were skipped.
   - Resolution: configured-structure validation now evaluates direct count
     constraints with count `0` for missing LS-Lint scopes, while still avoiding
     implicit required-directory violations for scopes without `exists`.
   - Evidence: `converted_lslint_file_exists_under_missing_scope_still_fails`.

3. LS-Lint canonical case names such as `camelcase`, `snakecase`, and
   `kebabcase` were not accepted everywhere.
   - Resolution: case validation and config semantic validation accept the
     canonical LS-Lint names alongside Assura display aliases.
   - Evidence: `converted_lslint_canonical_case_aliases_match_upstream_names`.

4. Fast-path regex simplification changed raw regex alternation semantics.
   - Resolution: simple literal optimization no longer rewrites ungrouped
     anchored alternation such as `^foo|bar$`.
   - Evidence:
     `converted_lslint_raw_regex_alternation_keeps_upstream_anchor_semantics`.

## Non-Blocking Finding Tracked

The review also noted that older comparison notes marked Assura-native
direct-content shorthand and top-level pattern ergonomics as complete. Those are
not LS-Lint compatibility blockers. The comparison doc now distinguishes
implemented LS-Lint parity from pending native shorthand ergonomics.

## Verification

- `cargo test --test ls_lint_rule_coverage_tests --quiet`
- `cargo test --test ls_lint_parity_regression_tests --quiet`

Both focused suites passed after the fixes. The rule coverage suite currently
contains 18 focused tests, including converted glob/brace ignore coverage and a
multi-config CLI migration smoke test.
