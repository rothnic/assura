---
id: goal-assura-beta-structure-severity-contract
type: goal
title: Assura beta structure severity contract
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-beta-code-agnostic-capabilities-program.md
  - ../../.assura/config.yml
  - ../../.trellis/spec/assura/config-notation.md
  - ../../.trellis/spec/assura/structure-enforcement.md
  - ../support-policy.md
---

# Assura Beta Structure Severity Contract

## Objective

Make structure validation beta-grade for code-agnostic repository quality
checks. Users and agents should get deterministic rule IDs, severity, paths,
clear messages, and remediation hints from the same `assura check` contract.

## Current Gap

Assura already validates repository structure and can emit JSON and agent
feedback. The beta surface still needs a clear severity and messaging contract
that can be reused by CLI output, hooks, daemon diagnostics, editor diagnostics,
and agent nudges without each surface inventing its own wording.

## Scope

- Define supported severity levels and default behavior for structure rules.
- Ensure structure findings include stable rule IDs, severity, path, expected
  shape, actual state, and concise remediation.
- Keep text, JSON, YAML, and agent output consistent.
- Add config examples for warning-only rules and blocking errors.
- Preserve LS-Lint-equivalent behavior where Assura is acting as an LS-Lint
  compatibility path.
- Add tests for severity propagation, text output, JSON output, and agent
  feedback output.

## Non-Goals

- No markdown lint implementation in this goal.
- No daemon protocol in this goal.
- No remote policy registry.

## Definition Of Done

- Structure validation findings have stable severity and rule-message fields.
- Agent feedback can render concise nudges without broad context.
- Warning-only rules can be reported without failing the check when configured.
- Blocking rules still produce correct nonzero exit behavior.
- Docs explain severity and message fields for humans and agents.

## Validation Commands

```bash
cargo fmt --check
cargo test structure --quiet
cargo test agent_feedback --quiet
cargo run --quiet -- check --format json .
cargo run --quiet -- check --format agent --agent codex .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm severity is a shared finding contract, not per-output wording.
- R2: Confirm warning-only behavior cannot hide configured blocking errors.
- R3: Confirm messages are concise enough for agent nudges.
- R4: Confirm LS-Lint-compatible configs keep parity semantics.

## Reviewer Blocking Criteria

Block if severity differs across output formats, agent nudges need to parse
human prose, warning rules can suppress hard errors accidentally, or LS-Lint
compatibility changes without regression evidence.
