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
  - .trellis/tasks/05-11-structure-check-benchmark-attribution
  - .trellis/spec/assura/structure-enforcement.md
  - src/config/ls_compat.rs
  - benches/profiling.rs
  - tests/ls_lint_parity_regression_tests.rs
---

# LS-Lint parity performance regression audit

## Summary

Assura's structure-first checker now has focused regression coverage for the
LS-Lint 2.3 basics that matter to the current compatibility layer:
extensions, wildcard extensions, `.dir`, nested directory rules, OR syntax,
`exists` counts, ignore/exclude behavior, and direct-child count semantics.

The first audit found one narrow correctness issue in `convert_ls_lint_to_config`:
direct child `exists` rules such as `README.md: exists:1` and
`docs/: exists:1` were parsed as child directory rules. That made Assura report
missing directories instead of direct file or direct directory count checks.
The fix maps pure direct-child `exists` rules into the current directory's
`files.exists` or `directories.exists` bundles.

The follow-up benchmark-attribution task corrected source-truth wording:
extension and `.dir` `exists` are LS-Lint 2.3 parity, while exact filename
`exists` remains an Assura compatibility extension.

Live LS-Lint 2.3 comparison was available through:

```bash
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version
```

The tool reported `ls-lint v2.3.0`.

## Source references

- LS-Lint 2.3 basics: <https://ls-lint.org/2.3/configuration/the-basics.html>
- LS-Lint 2.3 rules: <https://ls-lint.org/2.3/configuration/the-rules.html>
- LS-Lint 2.3 announcement: <https://ls-lint.org/blog/announcements/v2.3.0.html>
- LS-Lint v2.3.0 `exists` source:
  <https://raw.githubusercontent.com/loeffel-io/ls-lint/v2.3.0/internal/rule/exists.go>
- LS-Lint v2.3.0 linter source:
  <https://raw.githubusercontent.com/loeffel-io/ls-lint/v2.3.0/internal/linter/linter.go>

Relevant upstream facts:

- `ls` defines rules for extensions, sub-extensions, and directories while
  `ignore` excludes files and directories completely.
- `.dir` applies directory-name rules to the current directory and descendants.
- `.*` and `.*.js` wildcard extension rules are supported.
- `|` expresses multiple acceptable rules.
- `exists` allows or disallows `N` or `N-M` files for a given extension, works
  for `.dir` directory rules, and applies only to the directory itself.
- In v2.3.0 source, file `exists` counts are attached to extension rule keys;
  exact filename keys are not a native exact-file matching path.

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
| Extension and `.dir` `exists` | Supported as direct child count checks. | `converted_ls_lint_rules_cover_core_parity_surface`; existing count tests. | Covered |
| Exact filename `exists` | Supported as an Assura compatibility extension, not native LS-Lint 2.3 parity. | `converted_exact_file_exists_is_a_file_count_not_required_directory`; `converted_missing_exact_file_exists_reports_count_not_required_directory`. | Extension |
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

Exact filename verification command:

```bash
tmpdir=$(mktemp -d)
cd "$tmpdir"
printf 'ls:\n  README.md: exists:1\n' > .ls-lint.yml
printf '# Readme\n' > README.md
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint --version
npm exec --yes --package @ls-lint/ls-lint@2.3.0 -- ls-lint
```

Observed output:

```text
ls-lint v2.3.0
go go1.24.1 X:nocoverageredesign
. failed for `README.md` rules: exists:1 (found 0)
exit=1
```

## Regression fixture coverage

New fixture file:
`tests/ls_lint_parity_regression_tests.rs`.

| Test | Purpose |
| --- | --- |
| `converted_ls_lint_rules_cover_core_parity_surface` | Positive parity fixture for extension, `.dir`, nested rules, OR, range `exists`, and ignore behavior. |
| `converted_ls_lint_rules_report_expected_failures` | Negative parity fixture for naming, directory naming, `exists:0`, and `exists:N-M`. |
| `converted_exact_file_exists_is_a_file_count_not_required_directory` | Regression for the converter bug where exact file exists became a required directory. |
| `converted_missing_exact_file_exists_reports_count_not_required_directory` | Proves the exact filename compatibility extension reports `exists_count` when missing, not `required_directory`. |
| `direct_child_count_constraints_do_not_recurse` | Proves direct `exists` counts do not include nested descendants. |
| `direct_content_policy_is_not_inherited_recursively` | Proves closed-world direct-content policy does not leak recursively. |
| `ls_lint_parity_audit_performance_shapes` | Ignored historical manual fixture superseded by Criterion `structure_check/...` profiling groups. |

The current structure-first benchmark can be run with:

```bash
cargo bench --bench profiling structure_check -- --noplot
```

## Performance results

The Criterion `structure_check/full/run_structure_check/*` group now uses
`run_structure_check` against temporary structure-first repositories. Results
below are local Criterion medians from 2026-05-11 on branch
`codex/structure-check-benchmark-attribution`; they are attribution data, not
CI thresholds.

| Scenario | Median time | Throughput | Notes |
| --- | ---: | ---: | --- |
| small | 2.798 ms | 72.6 Kelem/s | No material change from pre-optimization run. |
| medium | 32.154 ms | 80.7 Kelem/s | Normal naming workload. |
| large | 121.30 ms | 85.0 Kelem/s | Traversal contributes about 20 ms in isolation. |
| deep tree | 9.158 ms | 17.8 Kelem/s | Depth is not the rule-heavy risk. |
| wide tree | 50.956 ms | 31.5 Kelem/s | Width is slower than sized trees but below rule-heavy cost. |
| many ignored/generated dirs | 355.08 us | 10.5 Melem/s | Exclusion pruning keeps generated output out of traversal. |
| many direct-content checks | 25.910 ms | 12.5 Kelem/s | Directory count reads are visible but bounded at this size. |
| many wildcard/extension/path rules | 511.22 ms | 19.0 Kelem/s | Improved from the pre-optimization 932.63 ms local median. |

Attribution group results:

| Cost slice | Median time | Finding |
| --- | ---: | --- |
| `config_load/large` | 63.979 us | Config parsing is not material for the regression. |
| `traversal/walkdir_large` | 19.981 ms | Traversal is visible but not the rule-heavy hotspot. |
| `traversal_pruned/walkdir_ignored_generated` | 125.64 us | Exclusion pruning is effective for ignored/generated dirs. |
| `direct_count_reads/many_direct_content_checks` | 13.330 ms | Direct count checks mostly cost directory reads. |
| `pattern_compile_each/many_wildcard_extension_path_rules` | 528.48 ms | Recompiling glob patterns per match was a dominant hotspot. |
| `pattern_precompiled/many_wildcard_extension_path_rules` | 96.958 ms | Reusing compiled glob patterns removes most matching overhead. |

The current structure-first `assura check` implementation uses
`walkdir::WalkDir` in `src/cli/check.rs`. Existing `jwalk` usage remains in
`benches/profiling.rs` and `benches/ls_lint_comparison.rs` around traversal
and the older `ConstraintEngine` benchmark path. This follow-up did not switch
the production structure-first walker to `jwalk`; the measured hotspot was
glob compilation, not traversal.

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
- Exact file-name `exists` is handled by Assura's converter as a compatibility
  extension, not as native LS-Lint 2.3 parity.

## Performance risks

- The rule-heavy fixture remains the clear risk area, but precompiling
  structure-first glob patterns reduced the local Criterion median from
  roughly 933 ms to roughly 511 ms.
- Pattern matching still scales with files times configured wildcard rules.
  If broader rule-heavy configs become common, the next optimization should
  index naming patterns by extension-like suffix before falling back to glob
  scans.
- Many direct-content checks are acceptable at this size, but they require
  directory reads for each configured directory. This should be watched if
  future configs add hundreds or thousands of direct count checks.
- Ignored/generated directories are effectively pruned in this fixture. That
  supports keeping generated outputs out of modeled structure.

## Recommended next implementation tasks

1. Add glob/alternative directory-pattern conversion for LS-Lint path scopes
   without treating every pattern as a required literal directory.
2. Split "configured validation scope" from "required directory" in
   structure-first checks so path-specific lint rules do not automatically
   require that path to exist unless an explicit `required` or `exists` rule is
   present.
3. Consider extension/suffix indexing for `files.naming_patterns` if future
   dogfooding configs remain rule-heavy after glob precompilation.
4. Keep exact filename `exists` documented as an Assura compatibility
   extension unless a future LS-Lint source review finds an official exact
   syntax.
5. Keep Windows CI restore, Codex runtime hook behavior, and hook
   blocking-policy decisions deferred to their existing owning work.
