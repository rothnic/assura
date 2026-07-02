# Agent Project Preset And Dynamic Contracts

## Goal

Implement the second child goal of the agent-ready onboarding program: turn the
broad baseline used by `assura agent onboard` into a reusable
`agent-project` preset with dynamic contracts for repeated project-local
structures such as skills, references, scripts, assets, packages, examples,
fixtures, and docs sections.

## What I Already Know

- Parent goal: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goal: `docs/goals/assura-agent-project-preset-dynamic-contracts.md`.
- Child goal 1 is complete: `assura agent onboard` creates a local broad
  baseline and onboarding packet, merges existing config defaults, and reports
  checked versus inactive capabilities.
- The first implementation writes a concrete `.assura/config.yml` template in
  `src/cli/agent_onboarding_templates.rs`.
- This slice must keep the preset broad and non-domain-specific.
- Performance backlog work is out of scope.

## Requirements

- Add a reusable `agent-project` baseline contract that onboarding can apply
  without duplicating large config blocks per repeated directory.
- Support repeated skill directories through one dynamic contract, including
  required `SKILL.md` and allowed `references/`, `scripts/`, and `assets/`.
- Add fixture coverage for passing and failing repeated skill directories,
  including unexpected child folders and missing `SKILL.md`.
- Keep command-surface truth intact; do not advertise future doctor,
  specialization, or domain-pack behavior as implemented.
- Preserve broad safe defaults for root clutter, Markdown/docs organization,
  binary exclusions, and project-local skill routing without adding
  language- or domain-specific rules.
- Document the baseline as broad and safe.

## Acceptance Criteria

- [x] A generated or referenced `agent-project` config can validate multiple
      skill directories through one reusable contract.
- [x] Passing fixtures cover multiple valid skill directories without listing
      every skill by name.
- [x] Failing fixtures cover unexpected child folders and missing `SKILL.md`.
- [x] The onboarding command uses the reusable preset/contract path rather
      than embedding one-off per-skill config expansion.
- [x] Docs/support text explains that the baseline is broad and
      non-domain-specific.
- [x] Existing child-1 onboarding tests remain green.

## Definition Of Done

- Focused tests for dynamic directory contracts pass.
- `cargo fmt --check`, repo self-check, `cargo xtask target-state`,
  `cargo xtask docs`, `cargo xtask evidence`, and `git diff --check` pass.
- Independent review checks that users no longer need to enumerate every skill
  directory manually and that the preset stays broad.

## Out Of Scope

- Proposal/SBIR or other domain packs.
- Rust, Node, Python, or web-app-specific project rules.
- Performance benchmark changes.
- Doctor/explain, content activation, lifecycle hook, and website overhaul
  surfaces from later child goals.

## Technical Notes

- Existing onboarding template:
  `src/cli/agent_onboarding_templates.rs`.
- Existing config loading and structure validation paths:
  `src/config/` and `src/core/validator.rs`.
- Existing test home for onboarding:
  `tests/project_intelligence_onboarding.rs`.
- Goal-specific validation suggests `cargo test dynamic_directory --quiet`
  and `cargo test skill --quiet`; inspect current test naming before relying
  on those filters.
