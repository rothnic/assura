---
title: Goal 07 extension plugin foundation review
date: 2026-06-02
status: evidence
---

# Goal 07 Extension Plugin Foundation Review

Goal file: `docs/goals/assura-goal-07-extension-and-plugin-foundation.md`.

## Scope Review

- Active Trellis task:
  `.trellis/tasks/06-01-roadmap-phase-01-execution`.
- Branch: `codex/phase-01-goal-07-extension-plugin-foundation`.
- Public surfaces changed: `.assura/config.yml` schema, `assura check`
  diagnostics, compiled config artifacts, and custom-constraints docs.
- Public CLI surface preserved: custom constraints run through `assura check`.
- Agent feedback surface preserved: stable feedback remains
  `assura check --format agent`, with Codex delivery only through
  `--agent codex`.
- Unsupported extension behavior remains unsupported: no remote plugin loading,
  marketplace, shell hooks, package feedback CLIs, per-agent CLI entrypoints, or
  per-agent `--format` values.

## Evidence Inventory

| Evidence | Location | Checked In | Reproduction Command |
| --- | --- | --- | --- |
| Extension config schema | `src/config/config/extensions.rs` | Yes | `cargo test custom_constraint --quiet` |
| Runtime custom constraint executor | `src/cli/check/custom_constraints.rs` | Yes | `cargo test --test custom_constraints_tests --quiet` |
| Fast-path guard | `src/cli/check/ls_fast_plan.rs` | Yes | `cargo test --all-targets --quiet` |
| Compiled artifact preservation | `src/cli/check/compiled_artifact.rs`, `crates/assura-check-cli/src/compiled.rs` | Yes | `cargo test --test compiled_config_cli --package assura-check-cli --quiet` |
| Passing/failing/exclusion fixtures | `tests/custom_constraints_tests.rs` | Yes | `cargo test --test custom_constraints_tests --quiet` |
| User docs | `website/src/content/docs/examples/custom-constraints.md` | Yes | `node --run verify:docs` |
| Iteration ledger and Trellis routing | `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`, `.trellis/spec/assura/roadmap.md` | Yes | `node --run verify:evidence` |

## Validation Commands

| Command | Status | Notes |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Passed | Re-ran after the target-template safety cleanup. |
| `cargo test --test custom_constraints_tests --quiet` | Passed | Covers missing target, existing target, excluded source, and root source-parent behavior. |
| `cargo test custom_constraint --quiet` | Passed | Covers config parsing and custom executor unit tests across harnesses. |
| `cargo test --all-targets --quiet` | Passed | Full Rust test suite, including benchmark-style harnesses. |
| `cargo test --test compiled_config_cli --package assura-check-cli --quiet` | Passed | Proves compiled artifacts preserve and execute `extensions.custom_constraints`. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed | No warnings after config/check/docs changes. |
| `cargo run --quiet -- check --format json .` | Passed | Assura self-check reports zero violations. |
| `node --run verify:docs` | Passed | Website static build completed with 28 pages. |
| `node --run verify:evidence` | Passed | Goal metadata, Trellis state, evidence docs, links, and stale-surface checks pass. |
| `node --run verify:fast` | Passed | Consolidated fast gate passed after implementation. |
| `git diff --check` | Passed | No whitespace errors. |

## Review Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| R0. Scope and source-of-truth review | Complete locally | Goal 06 is completed in the Iteration 01 ledger; Goal 07 is active in the goal file, Trellis task, and roadmap spec. |
| R1. API boundary and stability review | Complete locally | `extensions.custom_constraints` is documented as experimental and first-party; unsupported plugin surfaces remain rejected in docs and policy. |
| R2. Fixture and diagnostics review | Complete locally | `tests/custom_constraints_tests.rs` proves normal `StructureViolation` shape with `custom:<id>` rule names, configured severity, and root-level source handling. |
| R3. Docs reproduction review | Complete locally | `node --run verify:docs` builds the updated custom-constraints page. |
| R4. Safety review | Complete locally | Runtime walks only checked paths, prunes exclusions, sorts sources, rejects parent/prefix target escapes, and disables LS-Lint fast-only plans when custom constraints exist. |
| R5. Command-surface review | Complete locally | No new CLI commands or per-agent formats were added; custom constraints execute through `assura check`. |

## Review Feedback Closure

| Source | Finding | Decision | Evidence |
| --- | --- | --- | --- |
| Local review | `compiled_artifact.rs` and `cli_check_tests.rs` exceeded Assura line limits after the first implementation pass. | Fixed | Moved custom tests to `tests/custom_constraints_tests.rs`, simplified compiled artifact extension storage, and reran `cargo run --quiet -- check --format json .` with zero violations. |
| Local review | Target template safety allowed Windows prefix components in `is_safe_relative_path`. | Fixed | Rejected prefix components and reran `cargo test --test custom_constraints_tests --quiet`. |
| Review agent Nietzsche (`019e865e-6cc8-77d3-a9cb-b907f41b7425`) | `assura-check-compile-config` accepted extension-bearing configs, but `assura-check-compiled` used the fast-only artifact path and rejected them at runtime. | Fixed | Switched `assura-check-compiled` to the fallback-capable artifact runner and added `compiled_config_cli_supports_custom_constraint_artifacts`. |
| Review agent Nietzsche (`019e865e-6cc8-77d3-a9cb-b907f41b7425`) | Goal/review evidence pointed at stale test paths and overclaimed compiled artifact coverage before the compiled CLI smoke existed. | Fixed | Updated Goal 07 progress log and this review record to reference `tests/custom_constraints_tests.rs` and `crates/assura-check-cli/tests/compiled_config_cli.rs`. |
| Gemini Code Assist | Root-level sources made `{source_parent}/...` expand to a leading slash, which target safety rejected before the paired-file check could report the missing target. | Fixed | `expand_target_template` now collapses `{source_parent}/` when the parent is empty; `target_template_drops_empty_source_parent_prefix` and `check_custom_paired_file_constraint_handles_root_source_parent` cover helper and CLI behavior. |

## Handoff

- PR: https://github.com/rothnic/assura/pull/24.
- Next goal after completion:
  `/goal docs/goals/assura-goal-08-release-readiness-and-ecosystem.md`
- Known risks: `extensions.custom_constraints` is intentionally first-party
  and experimental. The first rule covers source/target file pairing only; broad
  third-party plugin APIs remain out of scope.
