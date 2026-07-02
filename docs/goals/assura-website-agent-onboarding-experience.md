---
id: goal-assura-website-agent-onboarding-experience
type: goal
title: Assura website agent onboarding experience
status: planned
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
