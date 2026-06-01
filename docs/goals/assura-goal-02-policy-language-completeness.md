---
id: goal-assura-roadmap-02-policy-language-completeness
type: goal
title: Assura roadmap 02 policy language completeness
status: completed
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/structure-enforcement.md
  - docs/goals/assura-ls-lint-rule-coverage-audit.md
---

# Goal 02: Policy Language Completeness

## Objective

Make the structure-first policy language complete enough for realistic
repository shape contracts while clearly separating LS-Lint-compatible behavior
from Assura-specific extensions.

This is a two-week team chunk for config, migration, docs, and test owners.

## Scope

- Audit supported file, directory, markdown, naming, ignore, direct-content, and
  existence/count rules.
- Close known LS-Lint migration gaps or document unsupported cases with tests.
- Add realistic fixtures for direct files, direct directories, required content,
  forbidden content, markdown frontmatter, and generated-output exclusions.
- Improve diagnostics so users can distinguish naming drift, unexpected
  contents, missing required paths, and count failures.
- Update config reference docs and examples with exact command output.
- Produce an analysis record that maps policy features to real adoption use
  cases.

## Non-Goals

- No dependency graph validation.
- No plugin API.
- No broad performance rearchitecture unless a correctness fix requires it.

## Definition Of Done

- Every supported config field has at least one passing and one failing test
  where practical.
- Migration tests cover native LS-Lint parity and Assura compatibility
  extensions separately.
- User docs label exact direct `exists:1` behavior as Assura behavior when it is
  not native LS-Lint parity.
- Error messages include stable path, rule, severity, and corrective context.
- The policy reference is sufficient to write a real multi-package project
  policy without reading source code.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo test ls_lint --quiet
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm the policy audit uses current config structs and migration code.
- R1: Review every new or changed policy contract against good/base/bad cases.
- R2: Review tests for both false positives and false negatives.
- R3: Reproduce fixture reports and compare documented output to actual CLI
  output.
- R4: Review docs as a first-time user writing `.assura/config.yml`.
- R5: Confirm the PR links each closed gap to tests and docs.

## Reviewer Blocking Criteria

Block the PR if docs claim LS-Lint parity for Assura-only behavior, if migration
silently changes semantics, or if a new config field lacks failure coverage.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-01 | Started Goal 02 from the Iteration 01 execution branch after Goal 01 merged; clarified the master roadmap artifact as an iteration ledger and moved active status to Goal 02. | `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`; `.trellis/spec/assura/roadmap.md`; `bash scripts/verify.sh fast`. |
| 2026-06-01 | Added corrective diagnostic context to structure violations, added a realistic multi-package policy fixture with passing and failing coverage, expanded configuration and LS-Lint migration docs, and produced the policy language audit. | `cargo test --test policy_language_completeness_tests --quiet`; `cargo test --test cli_command_surface_tests agent --quiet`; `docs/analysis/2026-06-01-goal-02-policy-language-audit.md`. |
| 2026-06-01 | Addressed review findings for LS-Lint `.dir`/`self_directory` wording and naming-drift test coverage, then reran the full Goal 02 gates. | `cargo fmt --all -- --check`; `cargo test --test policy_language_completeness_tests --quiet`; `cargo test --test cli_command_surface_tests agent --quiet`; `cargo test --all-targets --quiet`; `cargo test ls_lint --quiet`; `cargo run --quiet -- check --format json .`; `git diff --check`; `node --run verify:docs`; `bash scripts/verify.sh fast`. |
| 2026-06-01 | Addressed PR #19 Gemini review comments by replacing per-violation `format!` allocations in text feedback render loops with direct writes while preserving the `fast_cli.rs` line budget. | `cargo fmt --all -- --check`; `cargo test --test cli_command_surface_tests agent --quiet`; `cargo test --test policy_language_completeness_tests --quiet`; `cargo test --all-targets --quiet`; `cargo test ls_lint --quiet`; `cargo run --quiet -- check --format json .`; `node --run verify:docs`; `bash scripts/verify.sh fast`; `wc -l src/cli/check/fast_cli.rs`; `git diff --check`. |
