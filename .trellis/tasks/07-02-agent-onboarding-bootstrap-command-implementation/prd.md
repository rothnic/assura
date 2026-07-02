# Agent Onboarding Bootstrap Command Implementation

## Goal

Implement the first executable slice of the agent-ready onboarding program:
a local Assura CLI onboarding flow that can enter a new or existing project,
apply a broad non-domain-specific baseline, generate the onboarding packet, run
verification, and tell the coding agent which specialization questions to ask
next.

## What I Already Know

- Parent goal: `docs/goals/assura-agent-ready-project-onboarding-program.md`.
- Child goal: `docs/goals/assura-agent-onboarding-bootstrap-command.md`.
- The increment excludes `docs/goals/assura-performance-polish-program.md`.
- The current roadmap names Agent-Ready Project Onboarding as the active P0
  adoption lane and keeps Performance Polish as a separate lane.
- Support policy currently marks `assura agent integration` as an experimental
  local lifecycle surface that must delegate to shared Assura contracts.
- The stable agent feedback API remains `assura check --format agent` with
  optional `--agent codex`; per-agent formats and per-agent feedback CLIs are
  unsupported.
- Baseline self-check is clean: `cargo run --quiet -- check --format json .`
  reports zero violations across 1403 files.

## Requirements

- Add an installed CLI onboarding surface for agent-ready project setup.
- Preserve command-surface truth: newly implemented CLI behavior may be
  documented as supported/experimental, but future remote wrapper and later
  specialization behavior must stay clearly marked as future/planned until it
  exists.
- Run inspect/apply/verify phases without asking upfront specialization
  questions.
- Detect basic project type and agent harness with confidence, using broad
  low-risk defaults when confidence is low.
- Create or merge a broad baseline on empty and existing repositories without
  destructive overwrites.
- Generate `.assura/onboarding/summary.md`, `questions.md`, `agent-next.md`,
  and `doctor.json`.
- Generate `agent-next.md` with the parent goal's specialization questions and
  a clear instruction not to invent project conventions.
- Final output must report installed, detected, verified, inactive, and next
  action sections.
- Reuse existing `assura init`, `assura check`, and `assura agent integration`
  behavior where practical instead of embedding duplicate validation logic.

## Acceptance Criteria

- [ ] A brand-new temp repo can run the onboarding command and receive a broad
      baseline, generated packet, and passing verification.
- [ ] An existing repo can run the onboarding command without overwriting
      existing user-authored files.
- [ ] `agent-next.md` includes all specialization questions from the parent
      goal and tells agents not to invent conventions.
- [ ] `doctor.json` reports checked versus unchecked or inactive capabilities
      clearly enough for a coding agent.
- [ ] The final text or JSON output includes installed, detected, verified,
      inactive, and next-action sections.
- [ ] Command-surface tests prove unsupported future command claims are not
      presented as current support.

## Definition Of Done

- Focused integration tests cover new empty repo and existing repo behavior.
- Existing command-surface tests pass.
- The repo self-check, target-state, docs, evidence, and whitespace gates pass.
- Independent review checks that the first run starts broad, verifies, and
  then asks instead of guessing conventions.
- Public docs or website pages touched by the slice distinguish implemented
  behavior from roadmap-only behavior.

## Technical Approach

Implement the first slice as a local CLI subcommand under the existing
`assura agent` command family, because the support policy already treats agent
integration as a local lifecycle surface and because the stable top-level
feedback API remains `assura check --format agent`. The subcommand should own
local project onboarding behavior; the remote bootstrap wrapper remains a
documented future install-and-delegate convenience, not product logic.

Start with a conservative broad baseline and generated onboarding packet.
Leave project-specific rules, content models, domain packs, and the later
answer-consuming specialization flow to later child goals.

## Decision (ADR-lite)

Context: The parent goal calls for a future one-action bootstrap, but the repo
must not document unsupported remote install scripts or agent-specific command
surfaces as current truth.

Decision: Implement the installed local CLI onboarding surface first and keep
remote bootstrap and later specialization as planned contracts. Reuse existing
Assura check and agent integration contracts for verification and host-agent
status.

Consequences: This produces a useful local first-run flow now, avoids remote
script/product-logic risk, and keeps domain-specific behavior out of the core
baseline. Later child goals can expand dynamic contracts, doctor/explain,
content activation, and website onboarding without changing this boundary.

## Out Of Scope

- Remote bootstrap script implementation.
- Domain-specific proposal/SBIR behavior.
- Broad content model initialization beyond generated handoff text.
- Requirements/evidence traceability and computed checks.
- Performance-polish work.
- Automatic mutation of unsupported host-agent configuration.

## Technical Notes

- Roadmap source: `.trellis/spec/assura/roadmap.md`.
- Support policy source: `docs/support-policy.md`.
- Agent feedback direction lock: `.trellis/spec/assura/codex-agent-feedback.md`.
- Existing agent integration implementation:
  `src/cli/agent_integration.rs` and `src/cli/agent_integration_bundle.rs`.
- Existing init and command-surface tests: `tests/cli_command_surface_tests.rs`.
- Goal-specified focused test target:
  `tests/project_intelligence_onboarding.rs`.
