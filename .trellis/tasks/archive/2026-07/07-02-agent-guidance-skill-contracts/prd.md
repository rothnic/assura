# Agent Guidance Skill Contracts

## Goal

Implement the fourth child goal of the agent-ready onboarding program: make
`AGENTS.md`, project-local `SKILL.md` files, the skill index, and skill folders
enforceable agent-routing surfaces without turning every early draft into a
hard merge gate.

## What I Already Know

- Parent goal: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goal: `docs/goals/assura-agent-guidance-skill-contracts.md`.
- Child goals 1-3 are complete: local `assura agent onboard`, reusable dynamic
  project contracts, and top-level `assura doctor` / `assura explain`.
- The current roadmap marks Agent-Ready Project Onboarding as the active P0
  adoption priority and keeps Performance Polish as a separate lane.
- Existing `.assura/config.yml` enforces root `AGENTS.md` existence and
  project-local skill folder structure for this repository.
- Existing onboarding tests prove generated configs can validate repeated
  skill directories without enumerating every skill name.
- Current checks do not yet enforce semantic AGENTS/SKILL routing contracts:
  required guidance sections, skill frontmatter, required skill sections, skill
  index link integrity, or progressive-disclosure guidance.
- Command-surface truth matters: future surfaces must stay marked as future or
  planned until implemented.

## Revalidation Result

`valid`: the goal is still needed. A clean Assura self-check proves configured
structure validation passes, and the generated agent-project baseline already
has reusable directory contracts. It does not prove that agents can rely on
`AGENTS.md` or `SKILL.md` as concise, linked, semantically complete routing
surfaces.

## Requirements

- Validate required `AGENTS.md` sections for agent routing:
  `Operating Rules`, `Process Docs vs Skills`, `Skills`, and `Anchors`.
- Validate stable heading anchors in `AGENTS.md`, including a clear skills
  index and links from each project-local skill entry to an existing
  `.agents/skills/<skill>/SKILL.md`.
- Validate `AGENTS.md` maximum size with advisory/warn defaults so drafts can
  continue while merge-ready projects get useful drift feedback.
- Validate `SKILL.md` frontmatter fields: `name`, `description`,
  `applies_when`, and optional `version`.
- Validate required `SKILL.md` sections: `Workflow`, `Read as needed`,
  `Outputs`, and `Guardrails`.
- Enforce progressive disclosure by warning when `SKILL.md` grows too long and
  by allowing longer material under `references/` or `docs/process/`.
- Keep reusable skill-directory structure rules intact: required `SKILL.md`,
  optional `references/`, `scripts/`, and `assets/`, and forbidden unexpected
  folders by default.
- Enable the new checks from the broad `agent-project` onboarding baseline
  with sensible advisory defaults.
- Make agent-facing output point to the drifted guidance surface and explain
  the fix without implying later child goals are complete.
- Add website or docs-facing onboarding proof showing the expected AGENTS/SKILL
  shape.

## Acceptance Criteria

- [x] Focused fixtures cover valid and invalid `AGENTS.md` section, anchor,
      and skill-index cases.
- [x] Focused fixtures cover valid and invalid `SKILL.md` frontmatter,
      required section, and progressive-disclosure cases.
- [x] Generated `assura agent onboard` config enables AGENTS/SKILL contract
      checks through the broad agent-project baseline.
- [x] Agent-facing output names the drifted guidance file and provides a
      concrete next action.
- [x] Website or docs onboarding material shows the expected agent guidance
      and skill shape.
- [x] Existing child-goal behavior remains true: reusable dynamic skill-folder
      contracts still work, and doctor/explain do not overclaim inactive
      future capabilities.

## Definition Of Done

- `cargo fmt --check`
- `cargo test agents_md --quiet`
- `cargo test skill_contract --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`
- Independent review checks that the contracts improve agent routing, avoid
  long duplicated skill entrypoints, and keep draft-mode checks reasonably
  non-blocking.

## Out Of Scope

- Global skill registry.
- Agent-specific validation engines or one command per agent harness.
- Raw search, frontmatter reference graph, content model activation, lifecycle
  hook modes, website onboarding overhaul, document-project preset,
  requirements/evidence traceability, computed checks, and domain-specific
  domain packs.
- Blocking every early draft by default.

## Technical Approach

Prefer extending the existing structure validation/reporting path with focused
content-oriented guidance checks rather than adding a separate validation
engine. Reuse current report severities, path normalization, JSON/agent output,
and generated onboarding config patterns.

## Technical Notes

- Existing onboarding template:
  `src/cli/agent_onboarding_templates.rs`.
- Existing onboarding tests:
  `tests/project_intelligence_onboarding.rs`.
- Existing structure planning and dynamic directory contracts:
  `src/cli/check/rule_plan.rs`, `src/cli/check/scope_patterns.rs`, and
  `src/config/config/structure_notation.rs`.
- Existing doctor/explain output:
  `src/cli/doctor.rs`, `src/cli/doctor_agent.rs`, and
  `src/cli/check/explain.rs`.
- Existing command surface and support truth:
  `.assura/command-surface.yml`, `docs/support-policy.md`, and
  `docs/compatibility-and-surface.md`.
