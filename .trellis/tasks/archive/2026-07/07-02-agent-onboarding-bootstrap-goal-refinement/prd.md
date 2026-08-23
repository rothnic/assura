# Agent Onboarding Bootstrap Goal Refinement

## Goal

Refine the agent-ready project onboarding goal with the desired first-run
bootstrap product shape: install if needed, apply a broad baseline, install
agent harness integration, verify, and tell the agent exactly what to ask the
user next.

## What I Already Know

- The broader parent goal already exists at
  `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- The user wants a one-command bootstrap experience for new projects, with the
  remote script only installing/delegating to the real CLI.
- The first run should avoid asking many questions up front. It should apply a
  safe broad baseline first, then produce a short question list for
  specialization.
- The generated onboarding packet should include `.assura/onboarding/` files
  for humans and agents.
- Assura self-check rejects future unsupported command syntax if goal docs make
  it look like a current supported command, so planned surfaces must be worded
  carefully.

## Requirements

- Add bootstrap phases: install, inspect, apply baseline, install harness
  integration, apply project-type pack, verify, and produce agent next steps.
- Capture the generated onboarding packet and `agent-next.md` contract.
- Capture broad default baseline, project-type packs, harness adapters, hook
  profile behavior, specialization flow, and ideal UX.
- Split the non-performance backlog into individual child goal files so the
  next agent can execute the program goal-by-goal.
- Include website onboarding as a child goal because the onboarding experience
  must improve alongside the CLI/product backlog.
- Preserve the distinction between planned command surfaces and currently
  supported commands.

## Acceptance Criteria

- [x] Agent-ready onboarding goal includes the single-command bootstrap
      product shape.
- [x] The generated onboarding packet is explicit.
- [x] The agent next-step question list is explicit.
- [x] The broad baseline remains low-risk and non-specialized.
- [x] Child goal docs cover every non-performance item from the review.
- [x] Website onboarding improvements are represented as a goal.
- [x] The planned CLI surfaces do not fail Assura command-surface self-checks.

## Out Of Scope

- Implementing the onboarding command.
- Implementing remote install scripts.
- Changing support policy for planned commands.

## Technical Notes

- Goal file: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goals live under `docs/goals/assura-agent-*.md` plus the website
  onboarding goal.
- Roadmap source: `.trellis/spec/assura/roadmap.md`.
- Validation should include Assura self-check because command-surface docs
  constraints are intentionally active on goal files.
