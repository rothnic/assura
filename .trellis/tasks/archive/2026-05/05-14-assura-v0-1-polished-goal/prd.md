# Ship Assura v0.1 polished onboarding release

## Goal

Create and execute a durable long-running Codex goal for the Assura v0.1
polished onboarding release. The release must keep the CLI, LS-Lint
compatibility evidence, performance evidence, and website onboarding truthful
to the current product.

## What I Already Know

- The user requested a v0.1 polished goal file; use
  `docs/goals/assura-v0-1-polished.md` so the file stem satisfies the
  existing kebab-case docs rule without a one-off `allowed_names` exception.
- The goal is for shipping a truthful pre-1.0 developer onboarding release.
- The goal excludes the advanced agent-nudge system but must define the next
  goal for Codex/agent nudge MVP and quality measurement.
- `.assura/config.yml` restricts allowed project shape, so new versioned
  project areas must be represented there instead of handled through ad hoc
  exceptions.

## Requirements

- Create `docs/goals/assura-v0-1-polished.md`.
- Include objective, current repo truth, acceptance criteria, test matrix,
  benchmark matrix, website onboarding requirements, known gaps, non-goals,
  validation commands, progress log, final release checklist, and next-goal
  definition.
- Keep the document truthful to current repo state inspected during this task.
- Keep the new directory compatible with `.assura/config.yml`.
- Implement the currently required release work when feasible in this branch:
  CLI command truth, LS-Lint compatibility tests, performance benchmarks,
  website onboarding, and validation evidence.

## Acceptance Criteria

- [x] Goal file exists at `docs/goals/assura-v0-1-polished.md`.
- [x] Goal file contains the requested sections and clarifications.
- [x] `.assura/config.yml` allows `docs/goals/` markdown files.
- [x] `cargo run --quiet -- check --format json .` succeeds or blocker is
  documented.
- [x] CLI commands exposed in onboarding are implemented or removed from docs.
- [x] LS-Lint scope required-directory behavior is covered by regression tests.
- [x] Current-product LS-Lint comparison benchmark exists and has local
  baseline evidence.
- [x] Website onboarding uses supported commands and builds.

## Out of Scope

- Implementing the advanced agent-nudge runtime.
- Claiming full LS-Lint parity for unsupported features.
- Preserving stale V1/V2 onboarding terminology as current product guidance.

## Technical Notes

- Relevant current source files inspected: `src/cli/commands.rs`,
  `src/cli/args.rs`, `src/cli/check.rs`, `src/config/ls_compat.rs`,
  `tests/ls_lint_parity_regression_tests.rs`, `benches/README.md`,
  `benches/ls_lint_comparison.rs`, website guide files, and Codex integration
  skeleton files.
- Local platform build troubleshooting is captured in the reusable project
  skill at `.agents/skills/assura-local-build/SKILL.md` instead of being
  embedded only in this goal.
