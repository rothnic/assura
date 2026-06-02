---
id: goal-assura-roadmap-07-extension-and-plugin-foundation
type: goal
title: Assura roadmap 07 extension and plugin foundation
status: active
created: 2026-06-01
owners:
  - assura-maintainers
related:
  - docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md
  - .trellis/spec/assura/index.md
  - .agents/skills/custom/constraint-development/SKILL.md
---

# Goal 07: Extension And Plugin Foundation

## Objective

Create a constrained extension foundation so Assura can support custom
validation rules without fragmenting the core CLI or weakening safety.

This is a two-week team chunk for core architecture, rule authors, docs, and
security-minded reviewers.

## Scope

- Define extension boundaries for custom constraints, rule metadata, severity,
  diagnostics, and test fixtures.
- Decide what is stable now versus explicitly experimental.
- Provide one first-party custom constraint example that validates a real
  structure rule not covered by built-in fields.
- Add docs for developing, testing, and reviewing custom constraints.
- Ensure extensions cannot bypass path exclusions or produce unstable output.
- Keep core `assura check` as the execution surface.

## Non-Goals

- No untrusted remote plugin execution.
- No marketplace.
- No dynamic network loading.
- No agent-specific plugin entrypoints.

## Definition Of Done

- Extension API boundaries are documented with examples.
- At least one custom constraint has passing and failing fixtures.
- Diagnostics from custom rules match core report shape.
- Safety constraints around filesystem access and output determinism are tested
  or documented.
- Docs explain when to add a built-in rule versus a custom constraint.

## Required Validation

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo run --quiet -- check --format json .
git diff --check
```

## Review Tasks

- R0: Confirm extension work is still within Assura's structure-first roadmap.
- R1: Review API boundaries and stability labels.
- R2: Review custom constraint fixtures and diagnostics.
- R3: Reproduce extension examples from docs.
- R4: Review safety notes for untrusted paths, generated output, and
  deterministic ordering.
- R5: Confirm no new command surface bypasses `assura check`.

## Reviewer Blocking Criteria

Block the PR if extensions require network loading, if custom diagnostics do
not fit the stable report model, or if examples encourage ad hoc scripts instead
of repo-reviewed constraints.

## Progress Log

| Date | Event | Evidence |
| --- | --- | --- |
| 2026-06-02 | Started Goal 07 from updated `master` after Goal 06 merged; moved the active Trellis task to `codex/phase-01-goal-07-extension-plugin-foundation`. | `gh pr view 23 --json state,mergedAt,mergeCommit,url`; `git switch -c codex/phase-01-goal-07-extension-plugin-foundation`. |
| 2026-06-02 | Added the first constrained extension slice: `extensions.custom_constraints`, first-party `paired_file_exists`, config validation, normal `assura check` diagnostics, exclusion-aware execution, fixtures, and docs. | `src/config/config/extensions.rs`; `src/cli/check/custom_constraints.rs`; `tests/custom_constraints_tests.rs`; `crates/assura-check-cli/tests/compiled_config_cli.rs`; `website/src/content/docs/examples/custom-constraints.md`. |
