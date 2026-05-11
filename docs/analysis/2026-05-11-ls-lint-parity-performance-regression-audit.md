---
id: analysis-2026-05-11-ls-lint-parity-performance-regression-audit
type: analysis
title: LS-Lint parity performance regression audit
status: active
created: 2026-05-11
updated: 2026-05-11
owners:
  - assura-maintainers
related:
  - .trellis/tasks/05-11-ls-lint-parity-performance-regression-audit
  - .trellis/spec/assura/structure-enforcement.md
  - src/config/ls_compat.rs
  - tests/ls_lint_parity_regression_tests.rs
---

# LS-Lint parity performance regression audit

## Summary

Assura's structure-first checker now has focused regression coverage for the
LS-Lint 2.3 basics that matter to the current compatibility layer:
extensions, wildcard extensions, `.dir`, nested directory rules, OR syntax,
`exists` counts, ignore/exclude behavior, and direct-child count semantics.

The audit found one narrow correctness issue in `convert_ls_lint_to_config`:
direct child `exists` rules such as `README.md: exists:1` and
`docs/: exists:1` were parsed as child directory rules. That made Assura report
missing directories instead of direct file or direct directory count checks.
The fix maps pure direct-child `exists` rules into the current directory's
`files.exists` or `directories.exists` bundles.

Live LS-Lint 2.3 comparison was available through:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version
```

The tool reported `ls-lint v2.3.0`.

## Source references

- LS-Lint 2.3 basics: <https://ls-lint.org/2.3/configuration/the-basics.html>
- LS-Lint 2.3 rules: <https://ls-lint.org/2.3/configuration/the-rules.html>
- LS-Lint 2.3 announcement: <https://ls-lint.org/blog/announcements/v2.3.0.html>

Relevant upstream facts:

- `ls` defines rules for extensions, sub-extensions, and directories while
  `ignore` excludes files and directories completely.
- `.dir` applies directory-name rules to the current directory and descendants.
- `.*` and `.*.js` wildcard extension rules are supported.
- `|` expresses multiple acceptable rules.
- `exists` allows or disallows `N` or `N-M` files for a given extension, also
  works for directories, and applies only to the directory itself.

## Baseline health

Commands run from `/Users/nroth/workspace/assura` on branch
`codex/ls-lint-parity-performance-regression-audit`.

| Command | Result | Notes |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Pass | No formatting drift at task start. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass | Clean warning baseline. |
| `cargo test --all-targets --quiet` | Pass | 299 lib tests with 3 ignored, plus integration suites and bench harness tests. |
| `cargo run --quiet -- check --format json .` | Pass | `success: true`, `violations: []`, 352 files and 105 directories checked at task start. |

## Parity matrix

| LS-Lint 2.3 behavior | Assura behavior | Regression coverage | Status |
| --- | --- | --- | --- |
| Extension rules, for example `.ts: kebab-case` | Converted to `files.naming_patterns` such as `*.ts`. | `converted_ls_lint_rules_cover_core_parity_surface`; failing fixture in `converted_ls_lint_rules_report_expected_failures`. | Covered |
| Wildcard extension rules, for example `.*` and `.*.js` | Supported by extension matching and glob-style naming patterns. | Existing `check_supports_wildcard_extension_rules`; existing extension benchmark tests. | Covered |
| `.dir` directory naming | Converted to `directories.naming`; inherited recursively as naming policy. | `converted_ls_lint_rules_cover_core_parity_surface`; `check_validates_converted_ls_lint_dir_rules`. | Covered |
| Nested path rules, for example `src:` and `packages/core:` | Converted into nested `children` nodes. | `converted_ls_lint_rules_cover_core_parity_surface`; direct comparison fixture. | Covered with caveat |
| OR syntax | `split_naming_conventions` treats `|` as alternatives while preserving regex pipes. | `converted_ls_lint_rules_cover_core_parity_surface`; existing OR syntax tests. | Covered |
| `exists` | Supported as direct child count check. | `converted_exact_file_exists_is_a_file_count_not_required_directory`; existing count tests. | Covered |
| `exists:0` | Supported for direct files and directories. | `converted_ls_lint_rules_report_expected_failures`; existing count tests. | Covered |
| `exists:1` | Supported as exact direct count. | `converted_exact_file_exists_is_a_file_count_not_required_directory`; existing count tests. | Covered |
| `exists:N-M` | Supported as inclusive hyphen range. | `converted_ls_lint_rules_cover_core_parity_surface`; failing fixture with `.md: exists:1-2`. | Covered |
| Ignore/exclude behavior | Converted to `exclude`; `WalkDir` pruning and violation suppression respect exclusions. | `converted_ls_lint_rules_cover_core_parity_surface`; ignored invalid file fixture. | Covered |
| Direct child vs recursive semantics for `exists` | Count constraints inspect only direct children of the configured directory. | `direct_child_count_constraints_do_not_recurse`. | Covered |
| Direct-content closed-world policy inheritance | `allow_extra`, allowed, forbidden, and count checks are stripped when inherited into descendants. | `direct_content_policy_is_not_inherited_recursively`. | Covered |

## Live LS-Lint comparison

Two equivalent fixture families were generated outside the repo and run through
both `ls-lint v2.3.0` and `target/debug/assura check`.

| Fixture | LS-Lint result | Assura result | Notes |
| --- | --- | --- | --- |
| Valid extension, `.dir`, nested path, OR, range, and ignore fixture | Exit 0 | Exit 0, zero violations | Equivalent behavior for the core surface. |
| Invalid fixture with `BadName.ts`, `src/bad-dir`, `src/main-file.rs`, one `.log`, and three `.md` files | Exit 1 with five failures | Exit 1 with five violations: `file_naming`, `directory_naming`, `exists_count` | Equivalent pass/fail surface; rule names differ by product model. |

One important caveat: live LS-Lint 2.3 did not treat
`README.md: exists:1` as an exact filename existence check in the direct
comparison fixture; it reported zero matches even when `README.md` existed.
Assura's converter now handles exact direct file names as a useful
compatibility-extension behavior, but reports should not claim that exact file
exists is native LS-Lint parity.

## Regression fixture coverage

New fixture file:
`tests/ls_lint_parity_regression_tests.rs`.

| Test | Purpose |
| --- | --- |
| `converted_ls_lint_rules_cover_core_parity_surface` | Positive parity fixture for extension, `.dir`, nested rules, OR, range `exists`, and ignore behavior. |
| `converted_ls_lint_rules_report_expected_failures` | Negative parity fixture for naming, directory naming, `exists:0`, and `exists:N-M`. |
| `converted_exact_file_exists_is_a_file_count_not_required_directory` | Regression for the converter bug where exact file exists became a required directory. |
| `direct_child_count_constraints_do_not_recurse` | Proves direct `exists` counts do not include nested descendants. |
| `direct_content_policy_is_not_inherited_recursively` | Proves closed-world direct-content policy does not leak recursively. |
| `ls_lint_parity_audit_performance_shapes` | Ignored manual fixture for repeatable synthetic performance measurements. |

The ignored performance fixture can be run with:

```bash
cargo test --test ls_lint_parity_regression_tests -- --ignored --nocapture
```

## Performance results

The performance fixture uses `run_structure_check` against temporary
structure-first repositories. Results are single local measurements on
2026-05-11, useful for risk comparison but not a stable benchmark threshold.

| Scenario | Files checked | Dirs checked | Elapsed ms | Violations |
| --- | ---: | ---: | ---: | ---: |
| small | 192 | 8 | 5.626 | 0 |
| medium | 2,560 | 32 | 55.897 | 0 |
| large | 10,240 | 64 | 242.304 | 0 |
| deep tree | 80 | 80 | 16.516 | 0 |
| wide tree | 800 | 800 | 75.950 | 0 |
| many ignored/generated dirs | 1 | 1 | 0.645 | 0 |
| many direct-content checks | 160 | 160 | 59.523 | 0 |
| many wildcard/extension/path rules | 9,600 | 120 | 2,848.090 | 0 |

## Correctness gaps

Closed during this audit:

- Pure direct child `exists` rules in the converter now become direct count
  constraints instead of child directory requirements.

Remaining gaps:

- Glob and alternative directory patterns such as `packages/*`, `**`, and
  `{src,tests}` are documented LS-Lint basics, but Assura's structure-first
  conversion still models nested rules as literal `children` paths. This is
  acceptable for explicit paths such as `src` and `packages/core`, but it is
  not full LS-Lint directory pattern parity.
- Nested `children` conversion can imply required directory presence because
  configured structure nodes are validated as required. LS-Lint directory
  rules are lint scopes, not necessarily required directories. This audit did
  not rewrite that behavior.
- Regex directory substitutions such as `${0}` and regex negation are LS-Lint
  2.3 features outside this task's requested basics and remain unaudited.
- Exact file-name `exists` is now handled by Assura's converter, but live
  LS-Lint comparison did not confirm it as native LS-Lint behavior.

## Performance risks

- The rule-heavy fixture is the clear risk area: 9,600 files with 80
  naming-pattern entries took roughly 2.85 seconds locally. The implementation
  checks patterns sequentially per file, so this can scale poorly when many
  wildcard and extension rules are configured in broad directories.
- Many direct-content checks are acceptable at this size, but they require
  directory reads for each configured directory. This should be watched if
  future configs add hundreds or thousands of direct count checks.
- Ignored/generated directories are effectively pruned in this fixture. That
  is good evidence for current `exclude` behavior and supports keeping
  generated outputs out of modeled structure.
- Deep and wide trees did not expose an immediate regression, but the local
  fixture is a smoke measurement rather than a statistically rigorous
  benchmark.

## Recommended next implementation tasks

1. Add glob/alternative directory-pattern conversion for LS-Lint path scopes
   without treating every pattern as a required literal directory.
2. Split "configured validation scope" from "required directory" in
   structure-first checks so path-specific lint rules do not automatically
   require that path to exist unless an explicit `required` or `exists` rule is
   present.
3. Add a real Criterion benchmark for structure-first `assura check` with the
   rule-heavy scenario, then use it to guide pattern-indexing or compiled-rule
   improvements.
4. Decide whether exact filename `exists` is an Assura extension worth keeping,
   and document it separately from LS-Lint parity if retained.
5. Keep Windows CI restore, Codex runtime hook behavior, and hook
   blocking-policy decisions deferred to their existing owning work.
