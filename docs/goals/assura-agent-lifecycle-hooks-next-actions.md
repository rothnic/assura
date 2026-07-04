---
id: goal-assura-agent-lifecycle-hooks-next-actions
type: goal
title: Assura agent lifecycle hooks and next actions
status: completed
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-integration-lifecycle.md
  - ./assura-beta-structure-severity-contract.md
---

# Assura Agent Lifecycle Hooks And Next Actions

## Objective

Formalize nudge, warn, and gate lifecycle behavior for agent-ready projects and
make agent-facing output recommend the next best fixes.

## Scope

- Define advisory, warning, and merge-gate modes for checks and onboarding.
- Provide hook profiles for agent working loops, pre-commit warning, and
  pre-push or CI gate behavior.
- Reuse supported agent integration lifecycle surfaces where possible.
- Add ranked next-action output with priorities, human-readable action text,
  affected paths, and follow-up surfaces.
- Ensure hook installation is explicit, reviewable, and reversible.
- Keep agent nudges bounded and cache-friendly.

## Non-Goals

- No hidden mutation of host-agent config.
- No mandatory daemon for baseline hooks.
- No unbounded context injection.

## Definition Of Done

- Agent-ready onboarding installs or recommends lifecycle profiles with clear
  nudge/warn/gate semantics.
- Agent output can answer what to fix next without dumping the whole report.
- Pre-commit and CI examples use the same rule/severity model with different
  blocking behavior.
- Tests prove advisory mode exits successfully while gate mode blocks on
  configured errors.

## Validation Commands

```bash
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if lifecycle modes are implicit, if hook profiles hide side effects, if
agent output lacks ranked next actions, or if advisory mode can block normal
draft work unexpectedly.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-03 | Completed lifecycle hooks and next actions for the agent-ready onboarding slice. `assura agent onboard` now emits explicit nudge, warn, and gate lifecycle profiles over existing `assura agent nudge` and `assura check --format agent` commands; generated onboarding packets include `.assura/onboarding/lifecycle.md`; ranked next actions now carry priority, action, reason, affected paths, and follow-up commands. Advisory `--warn` behavior remains non-blocking while gate mode blocks on configured medium+ findings. | `.trellis/tasks/07-02-agent-lifecycle-hooks-next-actions/prd.md`; `src/cli/agent_lifecycle.rs`; `src/cli/agent_onboarding.rs`; `src/cli/agent_onboarding_report.rs`; `src/cli/agent_onboarding_templates.rs`; `tests/project_intelligence_onboarding.rs`; `tests/real_project_agentic_feedback_tests.rs`; independent review agent `019f264c-49b2-7f01-b4e3-14010041d3f2`; `cargo fmt --check`; `cargo test --test agent_surface_cli --quiet`; `cargo test --test real_project_agentic_feedback_tests --quiet`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo run --quiet -- check --format agent --agent codex .`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
