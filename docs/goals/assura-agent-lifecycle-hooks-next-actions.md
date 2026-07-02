---
id: goal-assura-agent-lifecycle-hooks-next-actions
type: goal
title: Assura agent lifecycle hooks and next actions
status: planned
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
