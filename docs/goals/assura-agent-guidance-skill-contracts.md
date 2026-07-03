---
id: goal-assura-agent-guidance-skill-contracts
type: goal
title: Assura agent guidance and skill contracts
status: completed
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

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Completed child goal 4. Added first-party `extensions.agent_guidance` checks for required `AGENTS.md` sections, duplicate heading anchors, project-local skill index links, required `SKILL.md` frontmatter and sections, and concise guidance entrypoints. The generated agent-ready baseline now enables the checks with advisory defaults, public docs show the expected guidance shape, and Assura self-policy tracks the new config surface plus compiled artifact coverage. | `src/cli/check/agent_guidance.rs`; `src/cli/check/agent_guidance/`; `src/config/config/extensions/agent_guidance.rs`; `src/cli/agent_onboarding_templates.rs`; `tests/agents_md.rs`; `tests/skill_contract.rs`; `crates/assura-check-cli/tests/compiled_agent_guidance_cli.rs`; independent review `McClintock`; `cargo fmt --check`; `cargo test agents_md --quiet`; `cargo test skill_contract --quiet`; `cargo test -p assura-check-cli --test compiled_agent_guidance_cli --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
| 2026-07-02 | Revalidated and started child goal 4. Existing self-check and generated onboarding contracts cover file/folder shape for `AGENTS.md` and `.agents/skills/*`, but do not yet enforce semantic AGENTS/SKILL routing contracts such as required sections, skill frontmatter, skill index links, or progressive-disclosure guidance. | `.trellis/tasks/07-02-agent-guidance-skill-contracts/prd.md`; `.assura/config.yml`; `tests/project_intelligence_onboarding.rs`; `cargo run --quiet -- check --format json .` reported 0 violations across 1421 files. |
