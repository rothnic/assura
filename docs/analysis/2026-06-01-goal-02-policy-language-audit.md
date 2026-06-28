---
title: Goal 02 Policy Language Audit
date: 2026-06-01
status: active
---

# Goal 02 Policy Language Audit

This record maps the Goal 02 policy surface to current source, tests, docs, and
adoption use cases. The goal is to keep the public `assura check` policy
language useful for realistic repositories without overclaiming LS-Lint parity.

## Source Files Audited

| Source | Purpose |
| --- | --- |
| `src/config/config.rs` | Public config root and directory node model. |
| `src/config/config/bundles.rs` | File, directory, markdown, and legacy exists bundles. |
| `src/config/ls_compat.rs` | LS-Lint conversion boundary. |
| `src/cli/check/configured_structure.rs` | Required path and missing-scope checks. |
| `src/cli/check/direct_contents.rs` | Closed-world direct child policy and count checks. |
| `src/cli/check/validators.rs` | File and directory naming, extension, size, and docs checks. |
| `src/cli/check/markdown.rs` | Markdown frontmatter, heading depth, and section checks. |
| `src/cli/check/report.rs` | Violation schema and corrective context. |

## Supported Policy Matrix

| Field | Enforcement | Passing evidence | Failing evidence | Adoption use case |
| --- | --- | --- | --- | --- |
| `structure` | Directory-scoped policy tree. | `check_passes_valid_structure`; `check_passes_realistic_multi_package_policy_matrix` | Existing CLI failure tests under `tests/cli_check_tests.rs` | One file defines repo shape. |
| `exclude` | Prunes paths before validation and direct counts. | `check_exists_counts_ignore_excluded_children`; generated-output fixture in `check_passes_realistic_multi_package_policy_matrix` | `ignored_generated_heavy_repo` invalid variant still catches non-excluded drift | Keep generated/build output out of source policy. |
| `DirectoryNode.required` | Configured directories must exist by default. | Existing valid structure tests | Missing `package-core` in `check_policy_diagnostics_include_corrective_context` | Prevent silent removal of owned scopes. |
| `DirectoryNode.inherit` | Child scopes inherit or reset parent bundles. | `.assura/` reset in policy matrix | Existing inheritance tests in `src/config/inheritance.rs` | Let generated/config dirs avoid parent rules. |
| `self_directory.naming` / `self_directory.exists` | Rules for the configured directory itself, primarily emitted by LS-Lint `.dir` migration. | `converted_ls_lint_rules_cover_core_parity_surface`; `explicit_directory_exists_rule_still_requires_matching_directory` | `converted_lslint_dir_rule_diagnostic_mentions_self_directory_context` | Preserve LS-Lint `.dir` semantics without confusing direct child directory policy. |
| `files.naming` | Direct file naming convention. | `check_passes_valid_structure` | `check_fails_bad_file_naming` | Keep source files consistently named. |
| `files.naming_patterns` | Pattern-specific naming. | `check_passes_realistic_multi_package_policy_matrix` | `monorepo_policy` invalid fixture | Mixed extension naming in web and Rust projects. |
| `files.max_lines` | Direct file line limit. | `check_passes_realistic_multi_package_policy_matrix` | Existing max-line CLI coverage in `tests/cli_check_tests.rs` | Keep agent-edited files small. |
| `files.max_size` | Direct file byte-size limit. | `check_passes_realistic_multi_package_policy_matrix` | Existing size validation coverage in validator tests | Block accidental large checked-in artifacts. |
| `files.require_docs` | Rust files require rustdoc. | `check_passes_realistic_multi_package_policy_matrix` | Existing `require_docs` validator coverage | Keep public Rust modules documented. |
| `files.extensions` | Direct files must match allowed extensions. | Extension CLI tests and policy matrix | Extension failure tests in `tests/cli_check_tests.rs` | Ban accidental file families in source scopes. |
| `files.severity` | Assigns violation severity. | Policy matrix config | `check_policy_diagnostics_include_corrective_context` | Route critical vs advisory issues. |
| `files.required` | Exact required direct files. | Policy matrix valid fixture | Missing `AGENTS.md` in policy diagnostics test | Ensure package-local instruction files exist. |
| `files.allowed_names` | Exact direct allow list. | Policy matrix valid fixture | Existing unexpected file tests | Keep root or package scopes closed-world. |
| `files.allowed_patterns` | Pattern allow list. | `Cargo.lock` in policy matrix | Existing allowed-pattern tests | Allow generated lockfiles without broadening root policy. |
| `files.forbidden_patterns` | Pattern deny list. | Policy matrix valid fixture | `draft-plan.md` in policy diagnostics test | Ban drafts, scratch files, or generated outputs. |
| `files.allow_extra` | Rejects undeclared direct files. | Policy matrix valid fixture | `scratch.txt` in policy diagnostics test | Prevent well-named stray files. |
| `files.exists` | Direct file count checks. | `package-core/README.md` in policy matrix | Missing `package-core` count in policy diagnostics test | Require or ban direct file classes. |
| `directories.naming` | Direct child directory naming. | Policy matrix `src` scope | `check_policy_diagnostics_include_corrective_context` covers direct child directory naming drift | Keep workspace directory names predictable. |
| `directories.severity` | Assigns directory violation severity. | Policy matrix config | Critical directory violations in policy diagnostics test | Promote root-shape drift above low-priority style. |
| `directories.required` | Exact required direct directories. | `src` and `docs` in policy matrix | Existing required-directory tests | Ensure source/docs scopes exist. |
| `directories.allowed_names` | Exact directory allow list. | Policy matrix root dirs | Existing unexpected directory tests | Keep root clean. |
| `directories.allowed_patterns` | Pattern directory allow list. | `package-core` in policy matrix | Existing LS-Lint parity direct policy fixture | Support package/workspace families. |
| `directories.forbidden_patterns` | Pattern directory deny list. | Policy matrix valid fixture | `tmp-cache` in policy diagnostics test | Ban scratch/cache directories. |
| `directories.allow_extra` | Rejects undeclared direct dirs. | Policy matrix valid fixture | `scratch/` in policy diagnostics test | Prevent new root scopes without review. |
| `directories.exists` | Direct directory count checks. | `package-core` count in policy matrix | Missing package count in policy diagnostics test | Enforce bounded workspace layout. |
| `markdown.require_frontmatter` | Markdown requires YAML frontmatter. | `docs/decision.md` in policy matrix | Invalid docs fixture in policy diagnostics test | Keep docs queryable and typed. |
| `markdown.required_fields` | Superseded. Typed frontmatter fields now belong to content runtime models and collections. | Historical Goal 02 policy matrix only | Content runtime model tests cover required typed fields | Avoid duplicate docs metadata policy surfaces. |
| `markdown.max_heading_depth` | Limits heading depth. | Existing heading-depth CLI test | Existing heading-depth CLI test | Keep docs structure shallow. |
| `markdown.required_sections` | Requires heading text. | `Summary` in policy matrix | Missing section in policy diagnostics test | Make review docs complete. |
| `exists.files` / `exists.directories` | Legacy required path lists. | `Cargo.toml` and `src` in policy matrix | Existing required path tests | Backward-compatible exact existence checks. |

## Accepted But Not Public Enforcement Fields

| Field | Current state | Product guidance |
| --- | --- | --- |
| `patterns` | Accepted by the config model and used by older library resolver APIs, but not the public `assura check` structure policy surface. | Do not document it as a user-facing check field; use `structure` scopes. |
| `ls` | Accepted by the config model for compatibility conversion and tests, but `assura check` compiles rule state from `structure`. | Use `assura migrate` to convert LS-Lint config into `structure`; do not hand-write `ls` as the check policy. |
| `markdown.check_links` | Accepted by the config model and inherited, but not enforced by current `assura check`. | Keep out of examples until link checking is implemented and tested. |

## LS-Lint Boundary

Native LS-Lint parity remains separate from Assura compatibility extensions:

| Rule family | Status | Evidence |
| --- | --- | --- |
| Extension, wildcard extension, subextension, `.dir`, OR syntax, regex, ignore, glob/brace scopes | Native LS-Lint parity target | `converted_ls_lint_rules_cover_core_parity_surface`; `realistic_fixture_manifest_is_pinned_and_complete` |
| `.md: exists:1-2` and `.dir: exists:1` | Native direct count parity | `converted_ls_lint_rules_cover_core_parity_surface`; `explicit_directory_exists_rule_still_requires_matching_directory` |
| `README.md: exists:1` and `docs/: exists:1` | Assura compatibility extension | `converted_exact_file_exists_is_a_file_count_not_required_directory`; `converted_missing_exact_file_exists_reports_count_not_required_directory`; docs label the behavior as Assura-only |
| Non-`exists` scalar path keys | LS-Lint-compatible no-op | Existing LS-Lint parity regression coverage |

## Diagnostic Contract

Every `StructureViolation` now serializes:

- `path`
- `rule`
- `message`
- `severity`
- `corrective_context`

The focused regression is
`check_policy_diagnostics_include_corrective_context`, which exercises missing
required files, unexpected and forbidden direct files, file naming drift,
unexpected and forbidden direct directories, direct child directory naming drift,
direct count failures, missing generic Markdown frontmatter, and missing
required sections. Typed frontmatter field requirements are now covered by
content runtime model validation instead of structure Markdown diagnostics. The
migrated `.dir` diagnostic boundary is covered by
`converted_lslint_dir_rule_diagnostic_mentions_self_directory_context`.

## Goal 02 Evidence Commands

Focused commands used during this slice:

```bash
cargo test --test cli_check_tests policy --quiet
```

Full Goal 02 validation still requires the commands listed in
`docs/goals/assura-goal-02-policy-language-completeness.md` before PR closure.
