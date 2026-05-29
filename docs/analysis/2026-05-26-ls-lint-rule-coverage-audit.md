---
id: analysis-2026-05-26-ls-lint-rule-coverage-audit
type: analysis
title: LS-Lint rule coverage audit
status: active
created: 2026-05-26
owners:
  - assura-maintainers
related:
  - docs/goals/assura-ls-lint-rule-coverage-audit.md
  - docs/goals/assura-ls-lint-counterexample-closure.md
  - docs/analysis/2026-05-26-ls-lint-counterexample-challenge.md
  - docs/analysis/2026-05-15-notation-source-truth.md
  - docs/ls-lint-capability-comparison.md
  - src/config/ls_compat.rs
  - src/cli/check/
  - tests/ls_lint_parity_regression_tests.rs
---

# LS-Lint Rule Coverage Audit

## Upstream Baseline

- Docs reviewed: [LS-Lint 2.3 rules](https://ls-lint.org/2.3/configuration/the-rules.html) and [LS-Lint 2.3 basics](https://ls-lint.org/2.3/configuration/the-basics.html).
- Source reviewed: `https://github.com/loeffel-io/ls-lint`, commit `49b4e7b`.
- Package checked: `@ls-lint/ls-lint@2.3.0`; `ls-lint --version` reported `ls-lint v2.3.0`.
- Source files inspected: `internal/rule/regex_test.go`, `internal/rule/exists_test.go`, all six case-rule tests, `internal/config/config_test.go`, `internal/linter/linter_test.go`, `internal/glob/glob.go`, and `cmd/ls_lint/main.go`.

## Post-Audit Counterexamples

The adversarial follow-up in
`docs/analysis/2026-05-26-ls-lint-counterexample-challenge.md` found additional
compatibility and performance gaps after this initial matrix was written. The
counterexample-closure goal now keeps these as covered regression cases:

- Exact scalar `exists` keys such as `README.md: exists:1` and
  `src/: exists:1` intentionally get stronger Assura semantics after
  migration. This is an Assura extension for structural requirements such as
  package-scoped `AGENTS.md`, not upstream LS-Lint 2.3 behavior.
- Non-`exists` scalar LS-Lint path keys such as `src: kebab-case` are
  LS-Lint-compatible no-ops during migration.
- Native targeted directory runs recurse in Assura, while
  `--ls-lint-target-semantics` provides LS-Lint-compatible target behavior.
- Root `.dir: exists` behavior is covered by parity tests.
- Long multipart extension rules use bounded suffix matching instead of wildcard
  candidate explosion.
- Many configured child scopes are included in release performance evidence.

Treat `docs/goals/assura-ls-lint-counterexample-closure.md` as the regression
gate for these counterexamples before updating public compatibility claims.

## Coverage Matrix

| LS-Lint behavior | Upstream evidence | Assura status | Assura evidence | Missing Assura tests | Required action | Agentic hot-path relevance |
| --- | --- | --- | --- | --- | --- | --- |
| `lowercase` | `internal/rule/lowercase_test.go` | Supported | `converted_lslint_case_rule_edges_cover_upstream_examples` | None | Keep parity fixture | Prevents false nudges on names like `abc-1`. |
| `camelcase` / `camelCase` | `internal/rule/camelcase_test.go` | Supported | `converted_lslint_canonical_case_aliases_match_upstream_names` and case-edge test | None | Keep parity fixture | Avoids over-rejecting `camelVCase`. |
| `pascalcase` / `PascalCase` | `internal/rule/pascalcase_test.go` | Supported | `converted_lslint_canonical_case_aliases_match_upstream_names` and case-edge test | None | Keep parity fixture | Avoids over-rejecting `PascalVCase`. |
| `snakecase` / `snake_case` | `internal/rule/snakecase_test.go` | Supported | `converted_lslint_canonical_case_aliases_match_upstream_names` and case-edge test | None | Keep parity fixture | Common agent file-generation rule. |
| `screamingsnakecase` / `SCREAMING_SNAKE_CASE` | `internal/rule/screamingsnakecase_test.go` | Supported | `converted_lslint_canonical_case_aliases_match_upstream_names` and case-edge test | None | Keep parity fixture | Useful for constants and env-like files. |
| `kebabcase` / `kebab-case` | `internal/rule/kebabcase_test.go` | Supported | Same case-edge test and canonical alias test | None | Keep parity fixture | Common docs/frontend rule. |
| `regex` | `internal/rule/regex_test.go` | Supported | regex anchoring, negation, substitution, and alternation tests | None | Keep live comparison | High risk because bad regex parity creates wrong fixes. |
| Regex anchoring | `regex.go` prepends `^` and appends `$` | Supported | `converted_lslint_regex_rules_are_full_string_matches` | None | Keep exact wrapping | Prevents partial-match false negatives/positives. |
| Regex negation | `regex_test.go` case `![0-9]+` | Supported | `converted_lslint_regex_negation_matches_upstream_semantics` | None | Keep fast/full path aligned | Stops agents from accepting explicitly forbidden stems. |
| Regex directory substitutions `${0}`, `${1}` | `regex_test.go`, issue 307 case | Supported | `converted_lslint_regex_directory_substitutions_match_upstream_semantics` | None | Consider per-dir regex cache if hot | Enables path-aware generated-file naming rules. |
| Multiple regex rules with `\|` | rule parser splits on ` | ` | Supported | `converted_lslint_multiple_regex_rules_preserve_or_semantics` | None | Keep split behavior stable | Lets agents satisfy one of several generated-name shapes. |
| Regex alternation inside one pattern | `regex.go` preserves raw pattern | Supported | `converted_lslint_raw_regex_alternation_keeps_upstream_anchor_semantics` | None | Keep fast/full path aligned | Avoids changing user-authored regex meaning. |
| `exists` | `internal/rule/exists_test.go` | Supported | direct-count and invalid-parser tests | None | Keep direct-count coverage | Core structural nudge for missing/extra files. |
| Bare `exists` | `exists.go` default min 1 | Supported | `converted_lslint_bare_exists_and_directory_exists_are_direct_counts` | None | Keep parity fixture | Reports missing expected artifacts. |
| `exists:0` | `exists_test.go` | Supported | core parity fixture and direct-count tests | None | Keep parity fixture | Blocks unwanted generated clutter. |
| `exists:N` | `exists_test.go` | Supported | `converted_lslint_rules_cover_core_parity_surface` | None | Keep parity fixture | Exact-count feedback for required file classes. |
| `exists:N-M` | `exists_test.go` | Supported | `converted_lslint_rules_cover_core_parity_surface` | None | Keep parity fixture | Useful for bounded project artifacts. |
| Invalid exists syntax | `exists_test.go` syntax/range cases | Supported | `invalid_lslint_exists_syntax_returns_clear_errors` | None | Keep clear migration errors | Avoids silent wrong nudges from malformed config. |
| Directory `exists` through `.dir` | `internal/linter/linter_test.go` | Supported | `converted_lslint_dir_exists_rules_validate_scope_presence` and root `.dir` fixtures | None | Keep root and child `.dir` parity fixtures | Directory-count nudges must be local and exact. |
| Direct-child-only exists semantics | `linter.go` validates counts at index dir only | Supported | `direct_child_count_constraints_do_not_recurse` | None | Keep fixture | Prevents descendant files from satisfying local contracts. |
| Targeted file/directory runs | `cmd/ls_lint/main.go`, `linter_test.go` path cases | Supported with explicit mode split | Native `assura check <path>` recursion plus `--ls-lint-target-semantics` parity fixture | None | Keep LS-Lint target-mode tests separate from native recursive feedback tests | Agents need fast changed-path checks. |
| Wildcard extensions `.*`, `.*.js`, `.*.*.go` | LS-Lint basics and `linter_test.go` | Supported | wildcard precedence and extension-combination tests | None | Keep upstream-order candidate tests | Critical for test/story/generated suffixes. |
| Sub-extension precedence `.test.ts` over `.ts` | LS-Lint extension combination loop | Supported | `converted_lslint_wildcard_extension_precedence_uses_most_specific_rule` | None | Keep fast/full parity | Prevents broad rules from overriding special files. |
| `.dir` scope and override | `config.GetConfig`, `linter.validateDir` | Supported | `converted_lslint_dir_rule_validates_scoped_directory_itself` and realistic fixtures | None | Add more override fixtures as rules grow | Directory rules guide project shape changes. |
| Glob directory scopes `*`, `**` | `internal/glob/glob.go`, `linter_test.go` | Supported | `converted_lslint_directory_pattern_scopes_*` | None | Keep matcher tests | Key for monorepos and generated package trees. |
| Brace directory scopes `{a,b}` | `internal/glob/glob.go`, `linter_test.go` | Supported | `converted_lslint_directory_pattern_scopes_*` | None | Keep matcher tests | Reduces config duplication in agent-facing policies. |
| Glob and brace ignore patterns | `glob.IgnoreIndex` | Supported | `converted_lslint_glob_and_brace_ignore_patterns_exclude_matches` | None | Keep converted-ignore fixture | Prevents noisy generated artifacts in feedback. |
| Multiple `--config` merge behavior | `cmd/ls_lint/main.go` `maps.Copy` and ignore union | Supported in migration conversion | `test_convert_multiple_ls_lint_configs_merges_like_config_flags`; `cli_migrate_accepts_multiple_lslint_configs_in_merge_order` | None | Keep unit and CLI smoke coverage | Lets teams compose baseline/project policy. |
| `--workdir` | LS-Lint flag | Supported by Assura checked path/project-root discovery | `assura check <project>` live comparison | None | No extra rule work | Agents can run checks from arbitrary workspace paths. |
| `--error-output-format json` | LS-Lint flag | Supported by Assura `--format json` | live comparison and JSON report tests | None | Keep report schema stable | JSON is the agent nudge transport. |
| `--warn` | LS-Lint flag | Supported | `check_warn_reports_violations_but_exits_successfully` | None | Keep CLI exit-code test | Advisory nudges need non-blocking mode. |
| Exact filename/directory `exists` | Live LS-Lint 2.3 reports zero for exact filename keys | Assura extension | `converted_assura_extended_exists_requires_files_and_directories` | None after counterexample closure | Keep converted exact `exists` support and label it as Assura-extended notation | Core agent policy for required files such as package-level `AGENTS.md`. |
| Non-`exists` scalar path keys | Live LS-Lint 2.3 treats `src: kebab-case` as a no-op | Supported no-op | `converted_lslint_non_dot_scalar_naming_keys_match_upstream_noop` | None | Keep scalar path naming separate from Assura's exact `exists` extension | Avoids false agent nudges from compatibility migration. |

## Live Comparison

Fixture shape:

```text
.ls-lint.yml
Bad-Dir/
123.num.js
baz.js
gen/swu1/data/data.js
google/test/test.js
missing/
present-zero/
src/
src/a/a/not_kebab.png
src/c/c/not_pascal.png
src/c/c/packages/not-snake-case.png
```

The `.ls-lint.yml` combined regex negation, raw regex alternation, regex
directory substitution, canonical case aliases, `.dir` self validation,
missing-scope `exists`, brace scopes, recursive glob scopes, nested overrides,
and multi-extension matching.

LS-Lint command:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --error-output-format json
```

LS-Lint exit: `1`. LS-Lint and Assura reported the same failing paths in the
combined fixture:

```text
Bad-Dir
123.num.js
baz.js
gen/swu1/data/data.js
google/test/test.js
missing
present-zero
src
src/a/a/not_kebab.png
src/c/c/not_pascal.png
src/c/c/packages/not-snake-case.png
```

Assura commands:

```bash
cargo run --quiet -- migrate .ls-lint.yml --output .assura/config.yml
cargo run --quiet -- check . --format json
```

Assura exit: `1`. Message text and JSON shape intentionally differ, but the
rule outcomes matched.

## Product Implications

The key compatibility gaps found by the audit were implementation gaps, not
claim-wording problems. They are now covered by product code and tests:

- LS-Lint regex negation and path substitutions now work in migrated configs.
- LS-Lint raw regex alternation remains a regex, including the upstream
  anchoring behavior for patterns such as `regex:foo|bar`.
- LS-Lint canonical rule names such as `camelcase`, `snakecase`, and
  `kebabcase` work alongside Assura's display aliases.
- LS-Lint `.dir` rules are represented as self-directory rules, so a scoped
  directory is validated as the scope itself instead of being treated as a
  direct-child policy.
- LS-Lint `exists` rules still fire for missing configured scopes, matching
  upstream's final exists validation pass.
- LS-Lint wildcard and brace directory scopes now behave as validation scopes,
  not required literal child directories.
- LS-Lint extension-combination semantics now prevent `.js` from catching
  `*.test.js`, while `.test.js` and `.*.js` still work.
- Advisory `--warn` mode is available for non-blocking agent nudges.
- Multi-config LS-Lint merge behavior is supported by migration conversion.

For the agentic hot path, the next optimization work should focus on caching
path-substituted regexes and compiled pattern-scope matchers if profiling shows
they matter. Correctness is now the gating requirement; speedups must preserve
the matrix above.
