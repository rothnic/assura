---
id: goal-assura-agent-integration-lifecycle
type: goal
title: Assura agent integration lifecycle
status: completed
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-beta-agent-nudge-integrations.md
  - ./assura-agent-daemon-awareness.md
---

# Assura Agent Integration Lifecycle

## Objective

Turn the current Codex, OpenCode, Claude, and Pi agent recipes into installable
and diagnosable local integration lifecycle surfaces over the shared Assura
nudge, daemon, and agent-feedback contracts.

## Current Gap

`v0.2.0` provides bounded `assura agent nudge` payloads and concrete wrapper
recipes, but Assura does not yet own install/update/remove/doctor workflows for
those integrations.

## Scope

- Define a shared integration manifest model for supported host agents.
- Add install, update, remove, status, and doctor flows for Codex, OpenCode,
  Claude, and Pi where local host conventions are known.
- Keep generated hooks/plugins thin: they call `assura agent nudge`,
  `assura check --format agent`, and daemon commands rather than embedding
  validation logic.
- Prove event placement for session-start, before-tool, after-tool, file-read,
  and recovery flows.
- Add context-budget and cache-safety tests for injected nudges.
- Document manual fallback steps when host-agent paths or permissions are
  unavailable.

## Non-Goals

- No per-agent validation engines.
- No one CLI command per private validator.
- No hosted orchestration service.
- No claim that Assura manages the full agent workflow.

## Definition Of Done

- Each target agent has install/update/remove/status/doctor documentation and
  at least one tested local fixture or dry-run installer path.
- Generated integration files are reproducible and reviewable.
- Nudges remain bounded by configured issue/path limits and never require
  hidden remote services.
- Independent review confirms all adapters reuse shared Assura contracts.

## Validation Commands

```bash
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test daemon_cli_tests --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
cargo xtask target-state
git diff --check
```

## Reviewer Blocking Criteria

Block if any integration embeds independent validation logic, emits unbounded
context, hides install-time side effects, lacks uninstall/doctor behavior, or
revives deprecated per-agent feedback commands or formats.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Completed the agent integration lifecycle child for this beta increment. `assura agent integration install|update|remove|status|doctor` now generates reviewable local bundles under `.assura/integrations/<agent>/` for Codex, OpenCode, Claude, and Pi, with a stable manifest, wrapper script, and README. Generated wrappers delegate to `assura agent nudge`, `assura check --format agent`, and `assura daemon` commands, and nudge placement now covers session-start, before-tool, after-tool, file-read, and recovery events. | `.trellis/tasks/07-02-agent-integration-lifecycle/prd.md`; `src/cli/agent_integration.rs`; `src/cli/agent_integration_bundle.rs`; `tests/agent_surface_cli.rs`; `.assura/command-surface.yml`; `docs/support-policy.md`; `docs/compatibility-and-surface.md`; `website/src/content/docs/reference/agent-feedback.md`; `cargo test --test agent_surface_cli --quiet`; `cargo xtask target-state`. |
