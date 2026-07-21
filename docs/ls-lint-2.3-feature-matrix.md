---
title: LS-Lint 2.3 Feature Matrix
status: active
---

# LS-Lint 2.3 Feature Matrix

This matrix is the checked source of truth for Assura's LS-Lint 2.3 config
migration claim. It covers config semantics only. Assura does not claim LS-Lint
CLI drop-in parity, exact JSON output parity, `--workdir`, `--debug`, or other
LS-Lint command-line flags.

## Matrix

| LS-Lint 2.3 feature | Converter support | Assura native support | Parity evidence | Docs reference |
| --- | --- | --- | --- | --- |
| `ls` mapping | Converted into `structure ./` | Structure-first config | `lslint_migration_rejects_unsupported_yaml_shapes` | This document; `docs/compatibility-and-surface.md` |
| `ignore` sequence | Converted into `exclude` and merged across configs | Exclude patterns | `native_lslint_golden_multi_config_merge_and_ignore_match_assura` | This document |
| Extension rules | Converted to file naming patterns | `files.naming_patterns` | `native_lslint_golden_extension_subextension_and_wildcard_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| Wildcard extension `.*` | Converted to `*.*` pattern | Glob naming/count patterns | `native_lslint_golden_extension_subextension_and_wildcard_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| Subextensions such as `.test.js`, `.d.ts` | Converted to most-specific glob patterns | Glob naming patterns | `native_lslint_golden_subextension_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| `.dir` rules | Converted to directory self-rules and directory counts | `self_directory` / `directories.exists` | `native_lslint_golden_directory_scopes_globs_braces_and_dir_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| Nested directory scopes | Converted to nested structure children | Structure children | `converted_ls_lint_rules_cover_core_parity_surface` | `docs/ls-lint-capability-comparison.md` |
| Dot-directory scopes | Dot-prefixed mappings such as `.agents:` convert as directory scopes; dot-prefixed scalar rules remain extensions | Structure children | `lslint_migration_accepts_dot_directory_scopes` | `website/src/content/docs/guides/ls-lint-migration.md` |
| Glob directory scopes | Preserved as matcher-backed scopes | Compiled scope patterns | `native_lslint_golden_directory_scopes_globs_braces_and_dir_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| Brace/alternative directory scopes | Preserved as matcher-backed scopes | Compiled scope patterns | `native_lslint_golden_directory_scopes_globs_braces_and_dir_rules_match_assura` | `docs/ls-lint-capability-comparison.md` |
| Multiple rules with ` | ` | Strictly split on LS-Lint separator | Naming alternatives and count checks | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| Invalid `|` separators outside `regex:` | Rejected | N/A | `lslint_migration_rejects_unknown_rules_and_invalid_syntax` | This document |
| Naming rules and aliases | LS-Lint 2.3 names and aliases accepted; unknown names rejected | Check-time case validators | `converted_lslint_case_rule_edges_cover_upstream_examples`; `converted_lslint_canonical_case_aliases_match_upstream_names` | This document |
| `regex:` rules | Converted with LS-Lint anchoring | Regex naming checks | `converted_lslint_regex_rules_are_full_string_matches` | `docs/ls-lint-capability-comparison.md` |
| Regex alternation inside `regex:` | Preserved inside the regex rule | Regex naming checks | `converted_lslint_raw_regex_alternation_keeps_upstream_anchor_semantics` | This document |
| Regex negation | Converted to negated regex naming | Regex naming checks | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| Regex directory substitutions `${0}`, `${1}` | Preserved for check-time substitution | Path-aware regex naming | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| `exists` | Converted to direct count checks | `files.exists` / `directories.exists` | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | `docs/ls-lint-capability-comparison.md` |
| `exists:0` | Converted to exact zero direct count | Direct count checks | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| `exists:N` | Converted to exact direct count | Direct count checks | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| `exists:N-M` | Converted to direct count range | Direct count checks | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura` | This document |
| File existence checks | Extension/subextension `exists` rules and scalar exact-file `exists` keys convert to direct file count checks | `files.exists` | `native_lslint_golden_regex_negation_substitutions_and_exists_match_assura`; `native_lslint_golden_scalar_rules_match_default_target_semantics` | `.trellis/spec/assura/structure-enforcement.md` |
| Directory existence checks | `.dir` `exists` rules and trailing-slash scalar `exists` keys convert to directory count checks | `directories.exists` | `converted_lslint_dir_exists_rules_validate_scope_presence`; `native_lslint_golden_scalar_rules_match_default_target_semantics` | `.trellis/spec/assura/structure-enforcement.md` |
| Scalar non-dot naming keys | Validated for LS-Lint rule syntax and otherwise ignored to match LS-Lint's no-op naming behavior | No emitted naming rule | `native_lslint_golden_scalar_rules_match_default_target_semantics`; `converted_lslint_non_dot_scalar_invalid_rules_are_rejected` | This document |
| Multiple config merge behavior | Replaces top-level `ls` keys in later configs; concatenates and deduplicates `ignore` | Converter entrypoint for multiple docs | `native_lslint_golden_multi_config_merge_and_ignore_match_assura`; `native_lslint_golden_multi_config_top_level_keys_replace_previous_rules` | This document |
| Explicit target-path semantics | Supported as Assura validation mode, not CLI parity | `--ls-lint-target-semantics` | `check_can_use_explicit_lslint_target_semantics_without_losing_native_recursion` | This document |

## Rejection Contract

`assura migrate` must fail before writing output for unknown LS-Lint rule names,
invalid `exists` syntax, invalid regex syntax, invalid rule separators, unknown
top-level YAML keys, non-string rule keys, non-string rules, and invalid
`ignore` or `ls` shapes. These failures are tested in
`ls_lint_rule_coverage_tests`.
