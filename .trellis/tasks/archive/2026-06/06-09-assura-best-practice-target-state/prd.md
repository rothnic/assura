# Define Assura best-practice target state

## Goal

Define an evidence-backed target state for Assura as a modern Rust CLI/workspace
and agent-driven repository, compare the current repository against that target,
and produce a prioritized cleanup and quality-rule plan. The goal is not just to
remove obvious clutter; it is to make "best practice" concrete enough that
Assura can enforce reusable parts of it through configuration or future rule
extensions.

## What I Already Know

- The earlier deslopify task completed a narrower containment slice, but it did
  not fully prove that the whole repository had been compared against a
  senior-engineer target-state rubric.
- The current repo already has meaningful structure controls in
  `.assura/config.yml`: root allowlists, docs/workflow scopes, command-surface
  documentation constraints, module topology constraints, and scoped quality
  gates.
- Current Rust source is broad: `src/` has about 43k lines across validation,
  config, constraints, CLI, markdown, maturity, intelligence, LS-Lint
  compatibility, performance-reporting, and experimental/internal surfaces.
- Current test coverage is non-trivial and includes CLI, config, constraints,
  custom constraints, LS-Lint parity/coverage, markdown, maturity, performance
  report contracts, policy-language completeness, and real-project agent
  feedback tests.
- The June 8 deslopify analysis documents classify several broad module
  families as current, supported-evidence, experimental-internal, or contained
  rather than safe to delete immediately.

## Assumptions

- This task should finish the missing analysis and target definition before
  broad remediation starts.
- Remediation should be split into follow-up PRs by risk: documentation and
  config tightening first, then code/module/test restructuring, then generalized
  Assura rule implementation.
- Deterministic detection should be preferred over subjective review, but the
  audit must explicitly mark issues that still require human architectural
  judgment.
- Existing public support policy remains authoritative unless this task finds a
  documented contradiction that should be corrected.

## Requirements

- Research current Rust project organization, Cargo workspace/manifest, testing,
  documentation, API, CI, release, and hygiene practices from primary or
  authoritative sources.
- Define an Assura-specific target-state rubric covering Rust layout, workspace
  metadata, module boundaries, public surface support, tests, documentation,
  build/release flow, CI/local gates, performance evidence, dependency hygiene,
  and agent workflow.
- Audit every major repository area against the rubric, not only the areas
  already suspected from prior work.
- For each misalignment, classify whether the fix belongs in repo cleanup,
  `.assura/config.yml`, a generalized Assura rule, an external language tool, or
  human review.
- Identify deterministic detection methods that can be rerun later and decide
  which ones are candidates for Assura's quality suite.
- Preserve clean-workspace discipline: no uncommitted changes should be carried
  into a separate task.

## Acceptance Criteria

- [ ] A durable analysis document defines the modern Rust plus agent-driven
      target state and cites the sources used.
- [ ] The analysis compares Assura's current state against every target-state
      category and records aligned, misaligned, and uncertain areas.
- [ ] The plan contains a prioritized backlog with P0/P1/P2 slices and a
      deterministic detector proposal for each issue where feasible.
- [ ] The plan maps each detector to one of: existing Assura config, config
      tightening, new generalized Assura rule, external tool, or human review.
- [ ] The task context JSONL files point future implement/check agents at the
      relevant specs, research, and analysis files.
- [ ] Required docs/workflow validation passes for the planning artifacts.

## Definition of Done

- Planning artifacts are committed on the task branch.
- Validation commands for documentation/workflow changes pass:
  `python3 -B ./.trellis/scripts/workflow_gate.py --platform codex`,
  `cargo run --quiet -- check --format json .`,
  `node --run verify:evidence`, and `git diff --check`.
- If the work becomes a PR, a review agent reviews the task before PR creation.

## Out of Scope

- Rewriting or deleting broad Rust module families before the target-state audit
  proves the intended destination.
- Adding broad `.assura/config.yml` exclusions to hide structure pressure.
- Declaring the whole deslopify goal complete without current-state evidence,
  validation commands, and follow-up remediation routing.

## Technical Notes

- Primary analysis artifact:
  `docs/analysis/2026-06-09-assura-best-practice-target-state.md`.
- Initial research artifact:
  `.trellis/tasks/06-09-assura-best-practice-target-state/research/rust-agent-repo-best-practices.md`.
- Existing context:
  `docs/analysis/2026-06-08-deslopify-completion-audit.md`,
  `docs/analysis/2026-06-08-deslopify-dead-path-classification.md`,
  and `docs/analysis/2026-06-08-deslopify-hygiene-evaluation.md`.
