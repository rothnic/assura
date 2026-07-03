---
id: goal-assura-website-agent-onboarding-experience
type: goal
title: Assura website agent onboarding experience
status: completed
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-onboarding-bootstrap-command.md
---

# Assura Website Agent Onboarding Experience

## Objective

Improve the website onboarding experience so users and agents understand the
first-run flow, the broad baseline, checked versus unchecked capabilities, and
the next questions before project specialization.

## Scope

- Rewrite onboarding pages around the agent-ready first-run journey.
- Explain the install-and-delegate remote wrapper without claiming unsupported
  commands as current.
- Show the generated onboarding packet and `agent-next.md` purpose.
- Teach "checked versus unchecked" as a core concept.
- Show default broad baseline, project-type packs, adapter behavior, hook
  lifecycle, doctor/explain, and specialization flow.
- Keep current supported command docs truthful and separate from planned
  surfaces.
- Add website tests or target-state checks that prevent unsupported onboarding
  claims from drifting into current docs.

## Non-Goals

- No marketing-only landing page.
- No unsupported command examples as current quickstart steps.
- No performance claim changes.

## Definition Of Done

- The website has a dedicated agent-ready onboarding path from the landing or
  getting-started docs to a page that explains broad baseline, verify, and
  specialize.
- The onboarding page includes sections for first-run phases, generated
  onboarding packet, checked versus unchecked capabilities, agent-next
  questions, project-type packs, adapter behavior, hook lifecycle, and
  specialization.
- Current supported commands and planned surfaces are visually and textually
  distinguished; unsupported planned commands are not shown as quickstart
  commands.
- The page includes example output or structured snippets for installed,
  detected, verified, inactive, and user choices needed.
- The website shows how this path relates to agent-project, document-project,
  and optional domain packs without making performance polish part of the
  onboarding increment.
- Screenshot or rendered-page evidence is captured for the revised onboarding
  page at desktop and mobile widths.
- Planned surfaces are labeled as roadmap until implementation lands.
- Website build and target-state checks pass.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
pnpm --dir website build
git diff --check
```

## Reviewer Blocking Criteria

Block if the website tells users to run unsupported commands, hides unchecked
capabilities behind a green check, or fails to show the agent next-step
handoff.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-03 | Completed the dedicated website onboarding path. The website now links from Getting Started and the home page to an Agent-Ready Onboarding guide that labels `assura agent onboard` as an experimental local surface, separates roadmap bootstrap behavior from current commands, shows installed/detected/content/lifecycle/verified/inactive/next-action output, explains checked versus unchecked state, documents the onboarding packet and `agent-next.md`, and routes agent-project, document-project, optional domain packs, adapter bundles, nudge/warn/gate lifecycle, doctor/explain, and specialization flow. Added target-state checks for the guide, sidebar, entry links, output markers, experimental-local wording, supported command markers, and forbidden unsupported commands. | `website/src/content/docs/guides/agent-ready-onboarding.md`; `website/astro.config.mjs`; `website/src/content/docs/index.mdx`; `website/src/content/docs/guides/getting-started.md`; `xtask/src/main.rs`; `.trellis/tasks/archive/2026-07/07-02-website-agent-onboarding-experience/evidence/rendered-proof.md`; independent review agents `019f265b-c003-73f0-94cf-ca51c030afc6` and `019f265c-2524-7731-9959-973e3c4b33ca`; `cargo fmt --check`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `pnpm --dir website build`; `cargo check --workspace --all-targets --all-features --quiet`; `git diff --check`. |
