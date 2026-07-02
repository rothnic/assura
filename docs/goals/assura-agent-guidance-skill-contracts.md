---
id: goal-assura-agent-guidance-skill-contracts
type: goal
title: Assura agent guidance and skill contracts
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-project-preset-dynamic-contracts.md
---

# Assura Agent Guidance And Skill Contracts

## Objective

Make `AGENTS.md`, `SKILL.md`, the skill index, and skill folders enforceable
agent-routing surfaces without turning every draft into a hard gate.

## Scope

- Validate required `AGENTS.md` sections: Operating rules, Process docs vs
  skills, Skills, and Anchors.
- Validate stable anchors, maximum size, project-local skill links, and
  separation between durable process docs and executable skills.
- Validate `SKILL.md` frontmatter fields and required sections: Workflow, Read
  as needed, Outputs, and Guardrails.
- Enforce concise progressive disclosure by routing longer material into
  `references/` or `docs/process/`.
- Validate that every use-case-oriented skill index entry points to an
  existing project-local skill.
- Apply advisory/warn/gate severity defaults appropriate for draft versus
  merge stages.

## Non-Goals

- No global skill registry.
- No agent-specific validation engines.
- No blocking every early draft by default.

## Definition Of Done

- Fixtures cover valid and invalid `AGENTS.md`, `SKILL.md`, skill index, and
  skill folder cases.
- The agent-project preset enables these checks with sensible advisory defaults.
- Agent-facing output explains which guidance surface drifted and how to fix
  it.
- Website onboarding shows the expected agent guidance shape.

## Validation Commands

```bash
cargo fmt --check
cargo test agents_md --quiet
cargo test skill_contract --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if agents can enter with stale or unlinkable guidance, if the contract
duplicates long workflow docs into `SKILL.md`, or if draft-mode checks are too
strict for normal onboarding.
