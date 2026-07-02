# Agent Doctor Explain Feedback

## Goal

Implement the third child goal of the agent-ready onboarding program: add
project-level doctor output and path-level explain output so agents can tell
the difference between configured checks that passed and capabilities that are
inactive, skipped, inherited, missing, or worth fixing next.

## What I Already Know

- Parent goal: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goal: `docs/goals/assura-agent-doctor-explain-feedback.md`.
- Child goal 1 is complete: `assura agent onboard` generates the broad
  onboarding baseline and `.assura/onboarding/` packet.
- Child goal 2 is complete: the generated agent-project baseline uses reusable
  dynamic contracts and validates repeated skills plus packages/docs/examples/
  fixtures without enumerating every child.
- Current roadmap epic is Agent-Ready Project Onboarding, the highest general
  adoption priority.
- Current support policy marks `assura agent onboard` experimental and says it
  creates `.assura/onboarding/`, runs local verification, and reports inactive
  capabilities.
- Existing doctor-like surfaces are scoped to daemon and agent integration
  lifecycle; this task is for top-level project doctor/explain behavior.
- Command-surface truth matters: do not advertise hidden mutation, remote
  bootstrap behavior, or domain-specific packs as implemented.

## Revalidation Result

`valid`: the goal is still needed. A clean `assura check` can prove configured
structure validation passes, but there is not yet a top-level project doctor or
path explain command that reports inactive models, empty collections, missing
recommended preset capabilities, inherited rule effects, skipped checks, binary
read behavior, and ranked agent next actions.

## Requirements

- Add a project doctor surface with text, JSON, and agent-oriented output.
- Report configured checks, inactive capabilities, empty or unwired project
  intelligence surfaces, unresolved-reference/search/content gaps when they can
  be determined locally, binary custody/read-exclusion status, and recommended
  preset gaps.
- Add a path explanation surface for applied scopes, inherited rules, skipped
  checks, binary/read behavior, suppressions, severity, and rule applicability.
- Keep doctor/explain read-only. They must not create files, mutate config, or
  treat inactive capabilities as violations by default.
- Feed compact ranked next actions into agent-oriented output.
- Include checked-versus-unchecked doctor sections in onboarding verification
  output without implying later child goals are complete.
- Preserve the stable `assura check --format agent` direction lock; do not add
  one agent-specific command or format per harness.
- Prove that draft model files not wired into config are surfaced by doctor.

## Acceptance Criteria

- [ ] `assura doctor <path> --format json` emits a deterministic schema with
      configured, inactive, gaps, and next-action sections.
- [ ] `assura doctor <path> --format text` is understandable without implying
      "no violations" means "fully onboarded".
- [ ] `assura doctor <path> --format agent` emits compact agent-oriented JSON
      with ranked next actions and follow-up commands.
- [ ] `assura explain <path> --format json` shows applied/inherited/skipped
      structure checks for Markdown, source, generated, binary, and skill
      paths where the information is available locally.
- [ ] Onboarding verification includes checked versus unchecked doctor
      sections backed by the same project-doctor logic.
- [ ] Tests prove doctor catches model files that exist but are not wired into
      config.

## Definition Of Done

- Focused doctor and explain tests pass.
- `cargo fmt --check`, `cargo test doctor --quiet`,
  `cargo test explain --quiet`, repo self-check, `cargo xtask target-state`,
  `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check` pass.
- Independent review checks that green `assura check` output cannot be mistaken
  for full onboarding completeness and that doctor/explain output is usable by
  agents.
- Website/onboarding proof is deferred to the website child goal, but this
  task must leave machine-readable surfaces that the website can document.

## Out Of Scope

- Natural-language query engine.
- Hidden mutation, automatic setup, or safe-fix application.
- Domain-specific proposal/SBIR checks.
- Lifecycle hook mode implementation.
- Performance backlog work.

## Technical Notes

- Existing CLI command dispatch lives in `src/cli/full_entry.rs`,
  `src/cli/commands.rs`, `src/cli/args.rs`, and command-specific `*_args.rs`
  modules.
- Existing onboarding output and `.assura/onboarding/doctor.json` generation
  live in `src/cli/agent_onboarding.rs` and
  `src/cli/agent_onboarding_templates.rs`.
- Existing daemon and integration doctor patterns live in
  `src/cli/daemon_*`, `src/cli/agent_integration_bundle.rs`, and tests in
  `tests/daemon_cli_tests.rs` and `tests/agent_surface_cli.rs`.
- Existing structure check report, rule plan, suppression, direct-content,
  Markdown, binary/read, and repository-reference logic should be reused rather
  than duplicated.
- Existing onboarding tests in `tests/project_intelligence_onboarding.rs` are
  the first place to prove checked-versus-unchecked handoff remains honest.
