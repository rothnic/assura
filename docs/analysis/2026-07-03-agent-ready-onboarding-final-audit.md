---
title: Agent-Ready Onboarding Final Audit
status: completed
date: 2026-07-03
---

# Agent-Ready Onboarding Final Audit

This audit closes the implementation portion of
`docs/goals/assura-agent-ready-project-onboarding-program.md`. Performance
polish remains a separate lane.

## Requirements Evidence

| Requirement | Current evidence | Verdict |
| --- | --- | --- |
| One broad agent-ready bootstrap exists | `assura agent onboard` support row, command surface, and onboarding tests | Complete |
| Generated baseline distinguishes checked from unchecked | Onboarding report `verified`/`inactive`, `doctor.json`, doctor/explain tests | Complete |
| Agent handoff tells agents not to invent conventions | Generated `.assura/onboarding/agent-next.md` and onboarding tests | Complete |
| AGENTS.md and SKILL.md are enforceable without bloat | `extensions.agent_guidance`, `tests/agents_md.rs`, `tests/skill_contract.rs` | Complete |
| Search and references work before perfect modeling | `assura content search`, raw fallback, repository references, context pack tests | Complete |
| Content models and source-document custody are intentional | `agent-project` and `document-project` content templates, source manifest tests | Complete |
| Nudge, warn, and gate lifecycle is explicit | `lifecycle_profiles`, `.assura/onboarding/lifecycle.md`, lifecycle tests | Complete |
| Agent-facing next actions are ranked | onboarding next actions and `assura content agent-query next-actions` tests | Complete |
| Document-project, traceability, computed checks, and proposal/SBIR pack are separate layers | child goals 9-12, support policy, compatibility matrix, proposal pack tests | Complete |
| Domain-specific behavior stays out of core presets | support policy, website guide, document-project tests, proposal pack opt-in tests | Complete |
| Future remote bootstrap remains truthful | website guide and support policy mark remote bootstrap as future; installed CLI owns current onboarding behavior | Complete for this increment |

## Child Goal Status

All twelve child goals in the parent execution sequence are marked completed
and have implementation evidence in their goal files or parent progress log.

## Validation Scope

Final closure should include:

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo fmt --check`
- `cargo check --workspace --all-targets --all-features --quiet`
- focused onboarding, traceability, computed-check, content-query, doctor, and
  website/target-state tests
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

## Reviewer Notes

Review should block if public docs imply unsupported remote bootstrap behavior,
if public roadmap status contradicts the completed parent goal, if the website
guide loses the checked-versus-unchecked mental model, or if proposal/SBIR
behavior appears in a generic preset.

Independent review `019f2738-76c6-75a3-ac6a-fac247b8ae32` found stale
"future" wording in the parent goal. The wording was corrected, and the same
review agent reported no remaining blockers.
