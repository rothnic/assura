# Agent Proposal SBIR Domain Pack

## Objective

Add an optional proposal/SBIR domain pack that composes with the existing
agent-ready and document-project onboarding templates without making proposal
behavior part of the core preset.

## Problem

Assura now has generic agent onboarding, document-project/source-custody
modeling, requirements traceability, and script-backed computed checks. Proposal
teams still need an opinionated, use-case-specific starter pack for SBIR-style
work: requirements, evidence, scoring, review findings, final package readiness,
and agent next actions. The current onboarding docs explicitly describe
proposal/SBIR behavior as future optional domain work.

Agents also need clearer progressive-disclosure guidance in generated
`AGENTS.md` and skill files. File length violations for agent and skill guidance
should tell agents how to resolve the issue by moving detail into routed skills
or referenced docs instead of only reporting a structural failure.

## Requirements

- Add an explicit opt-in proposal/SBIR content template or domain pack.
- Preserve the generic `agent-project` and `document-project` baselines as
  domain-neutral defaults.
- Model proposal requirements, evidence, source documents, scorecards, review
  findings, final package readiness, and submission checklist records with
  deterministic Assura checks.
- Use existing first-party extensions, especially requirements traceability and
  script-backed computed checks, instead of adding proposal behavior to core.
- Generate agent-facing next actions that prioritize missing evidence,
  unresolved source documents, incomplete review findings, and final package
  gaps.
- Add a progressive-disclosure routing reference to generated `AGENTS.md`.
- Add an optional opinionated rule that validates an AGENTS use-case routing
  table mapping "when doing X" to required skills.
- Add analogous skill-file guidance requiring skills to route to referenced docs
  when detailed instructions would exceed length limits.
- Improve file-length diagnostics for agent and skill files with remediation
  messages tailored to progressive disclosure.

## Non-Goals

- Do not make proposal/SBIR checks part of the default or broad document-project
  onboarding path.
- Do not require PDF or DOCX parsing for final package validation.
- Do not let scorecards replace human proposal judgment.
- Do not add per-agent feedback commands; keep `assura check --format agent` as
  the stable agent feedback API.

## Acceptance Criteria

- A generated proposal/SBIR starter project validates successfully with
  `assura check`.
- Negative tests prove proposal-specific missing evidence, incomplete review, or
  package gaps produce deterministic findings.
- Generated `AGENTS.md` includes use-case-oriented skill routing guidance.
- AGENTS/skill file length violations include actionable progressive-disclosure
  remediation text.
- Docs and website reference the optional proposal/SBIR pack and the
  progressive-disclosure guidance without implying domain behavior is core.
