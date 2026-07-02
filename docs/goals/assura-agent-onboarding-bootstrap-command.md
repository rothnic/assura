---
id: goal-assura-agent-onboarding-bootstrap-command
type: goal
title: Assura agent onboarding bootstrap command
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-project-preset-dynamic-contracts.md
  - ./assura-agent-doctor-explain-feedback.md
---

# Assura Agent Onboarding Bootstrap Command

## Objective

Create the first-run agent onboarding flow that installs or verifies Assura,
applies a broad safe baseline, installs supported harness integration, verifies
setup, and tells the agent exactly what to ask next.

## Scope

- Add the installed CLI onboarding surface for agent-ready project setup.
- Define the remote bootstrap wrapper as install-and-delegate only.
- Implement inspect/apply/verify phases with confidence-aware project-type and
  agent-harness detection.
- Generate the onboarding packet under `.assura/onboarding/`.
- Generate `agent-next.md` with the specialization questions and instructions
  not to invent project conventions.
- Add an apply/verify flow that can run on a new empty repo and on an existing
  repo without destructive overwrites.
- Add a later specialization entrypoint that consumes saved answers or runs an
  interactive equivalent.

## Non-Goals

- No domain-specific proposal pack implementation.
- No unsupported host-agent config mutation.
- No remote script that owns product behavior.
- No broad content model or traceability implementation beyond generated
  onboarding handoff.

## Definition Of Done

- A brand-new temp repo can complete the agent onboarding flow with a broad
  baseline, generated packet, and no destructive overwrites.
- An existing repo can merge the broad baseline and report conflicts or manual
  decisions clearly.
- The generated `agent-next.md` includes the specialization questions from the
  parent goal.
- The final output reports installed, detected, verified, inactive, and next
  action sections.
- Independent review confirms the first run starts broad, verifies, and then
  asks.

## Validation Commands

```bash
cargo fmt --check
cargo test --test project_intelligence_onboarding --quiet
cargo test --test cli_command_surface_tests init --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if the bootstrap asks too many questions before creating a baseline, if
the remote wrapper contains product logic, if existing files are overwritten
without review, or if generated next steps let agents guess project
conventions.
