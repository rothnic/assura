# Project Intelligence Usability Goals

## Goal

Evaluate what remains after the Project Intelligence Runtime program and create
a new ordered set of Assura goal docs that move the runtime from implemented
foundation to usable product workflow.

## What I Already Know

- Project Intelligence Runtime is completed locally in
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- The current supported runtime surface includes `assura content` query
  commands, `assura content agent-context`, `assura content agent-query`, and
  `assura fix markdown --dry-run --format json`.
- The current docs explicitly mark daemon/editor session, LSP, and MCP surfaces
  as planned.
- `assura watch` remains experimental until watch-mode tests and docs exist.
- The current support policy keeps semantic search and baseline code-symbol
  evidence as candidate context only, not validation truth.

## Requirements

- Identify the usability gaps that remain after the runtime foundation.
- Create a master usability program goal under `docs/goals/`.
- Create ordered successor goals that are executable independently and do not
  duplicate completed Project Intelligence Runtime successors.
- Update the Assura roadmap so agents can discover the new iteration and first
  recommended goal.
- Include proof gates, validation commands, review tasks, and reviewer blocking
  criteria in each goal.

## Acceptance Criteria

- [x] New goal docs exist for the usability program and its successors.
- [x] The master goal explains remaining gaps and the intended sequence.
- [x] Each successor has objective, current gap, scope, non-goals, definition of
  done, validation commands, review tasks, and blocking criteria.
- [x] Roadmap routes future work to the new usability iteration without
  reopening completed runtime work.
- [x] Assura self-check, docs/evidence gates, and whitespace checks pass.

## Definition Of Done

- The repo contains a clear, ordered goal set for turning project intelligence
  into a usable product workflow.
- Future agents can pick the first goal from the roadmap without needing this
  conversation.
- No goal requires hosted infrastructure, external semantic/code providers, or
  per-agent command families for core usability.

## Out Of Scope

- Implementing the new goals in this task.
- Changing CLI behavior beyond documentation/planning updates.
- Claiming daemon, LSP, MCP, or watch mode support before implementation and
  tests exist.

## Technical Notes

- Relevant master goal:
  `docs/goals/assura-project-intelligence-runtime-program.md`.
- Relevant archived audit:
  `.trellis/tasks/archive/2026-06/06-29-06-29-project-intelligence-runtime-completion-audit/prd.md`.
- Relevant support docs:
  `docs/support-policy.md`,
  `website/src/content/docs/product/agent-editor-surfaces.md`, and
  `website/src/content/docs/reference/agent-feedback.md`.
- Live probes confirmed `assura content` and `assura fix markdown --dry-run`
  surfaces exist, while daemon/editor, LSP, MCP, and watch remain planned or
  experimental.

## Decision

Use a new post-runtime iteration instead of extending the completed runtime
program. The completed runtime goal proves core local contracts; the new
iteration should prove usability through adoption, real-repo evidence,
persistent sessions, transport wrappers, safer repair workflows, and release
hardening.

## Gap Evaluation

| Gap | Evidence | New goal |
| --- | --- | --- |
| First-run project-intelligence adoption is not a single guided path | Runtime commands exist, but docs require users to connect init, models, Markdown, queries, safe fixes, and agent envelopes themselves. | `docs/goals/assura-project-intelligence-adoption-blueprint.md` |
| Realistic non-Assura proof is not packaged as a user-facing adoption artifact | Existing proof is strong on fixtures and Assura-local evidence. | `docs/goals/assura-project-intelligence-real-repo-proof.md` |
| Repeated agent/editor workflows lack a promoted warm session | Docs still describe warm sessions as future integration and `assura watch` as experimental. | `docs/goals/assura-project-intelligence-persistent-session.md` |
| Editor and agent transports are planned, not implemented support surfaces | Product docs classify daemon/editor session, LSP, and MCP as planned. | `docs/goals/assura-project-intelligence-editor-agent-transports.md` |
| Safe fixes stop at dry-run for the current supported contract | Support policy marks `assura fix markdown --dry-run --format json` as experimental safe-fix contract. | `docs/goals/assura-project-intelligence-safe-fix-workflow.md` |
| Release surfaces do not yet package project-intelligence usability as a promoted slice | Support and release docs classify surfaces but do not yet lock the full usability program. | `docs/goals/assura-project-intelligence-release-hardening.md` |

## Validation Evidence

- 2026-06-29: `git diff --check` passed.
- 2026-06-29: `cargo run --quiet -- check --format json .` passed with 1089
  files and 269 directories checked and no violations.
- 2026-06-29: `cargo xtask evidence` passed.
- 2026-06-29: `cargo xtask docs` passed and built 36 pages.
