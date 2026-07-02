---
id: goal-assura-agent-doctor-explain-feedback
type: goal
title: Assura agent doctor explain feedback
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-onboarding-bootstrap-command.md
---

# Assura Agent Doctor Explain Feedback

## Objective

Prevent false green-check confidence by adding project-level doctor output and
path-level explain output that show what is active, inactive, inherited,
skipped, broken, and worth fixing next.

## Scope

- Add a project doctor surface with text, JSON, and agent-oriented output.
- Report configured checks, inactive models, empty collections, zero search
  chunks, unresolved references, binary custody status, inherited-rule risks,
  and recommended preset gaps.
- Add path explanation for applied scopes, inherited rules, disabled
  inheritance, skipped checks, binary/read behavior, suppressions, severity,
  and rule applicability.
- Include checked versus unchecked sections in onboarding verification.
- Feed ranked next actions into agent-facing output.

## Non-Goals

- No natural-language query engine.
- No hidden mutation from doctor or explain.
- No treating inactive capabilities as violations unless the selected mode
  says they block.

## Definition Of Done

- A clean configured check can still report inactive models, empty collections,
  and missing recommended preset capabilities.
- Explain output makes inheritance and skips understandable for Markdown,
  source, generated, binary, and skill paths.
- Agent output includes compact next actions with follow-up surfaces.
- Tests prove doctor catches draft models that are not wired into config.

## Validation Commands

```bash
cargo fmt --check
cargo test doctor --quiet
cargo test explain --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if "no violations" can still be mistaken for "fully onboarded", if
explain omits inherited or skipped rules, or if doctor output is not usable by
agents.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Started child goal 3 after closing child goal 2. Revalidated that the goal remains valid because top-level project doctor and path explain surfaces do not yet exist, while current onboarding/check output can still leave inactive, unwired, inherited, skipped, and next-action state implicit. | `.trellis/tasks/07-02-agent-doctor-explain-feedback/prd.md`; `.trellis/spec/assura/roadmap.md`; `docs/support-policy.md`; `docs/goals/assura-agent-ready-project-onboarding-program.md`; `python3 ./.trellis/scripts/task.py validate 07-02-agent-doctor-explain-feedback`. |
