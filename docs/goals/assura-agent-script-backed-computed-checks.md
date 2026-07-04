---
id: goal-assura-agent-script-backed-computed-checks
type: goal
title: Assura agent script backed computed checks
status: completed
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-extension-api-clarification.md
---

# Assura Agent Script Backed Computed Checks

## Objective

Enable computed checks such as rollups, scores, confidence adjustments, and
domain-specific derived validations through a controlled script-backed
extension surface before building native computed fields.

## Scope

- Define a narrow computed-check contract with input data, output findings,
  severity, stable rule IDs, and reproducible metadata.
- Support project-local scripts only through an explicit configured allowlist.
- Feed computed findings into normal reports, doctor, agent output, hooks, and
  merge gates.
- Provide fixtures for successful checks, failing checks, missing scripts,
  invalid output, timeout, and nonzero exit behavior.
- Document the boundary between first-party checks, internal Rust APIs, and
  deferred plugin APIs.

## Non-Goals

- No arbitrary unconfigured script execution.
- No marketplace plugin system.
- No native formula language until script-backed checks prove the contracts.

## Definition Of Done

- A project can configure a computed check and receive normal Assura findings.
- Invalid or unsafe script behavior fails with clear diagnostics.
- Agent output includes concise next actions without dumping large script
  payloads.
- Website docs explain computed checks as an advanced, explicit project feature.

## Validation Commands

```bash
cargo fmt --check
cargo test computed_checks --quiet
cargo test --test agent_surface_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Reviewer Blocking Criteria

Block if scripts run implicitly, if findings bypass severity/rule/message
contracts, if output is not deterministic enough for CI, or if the feature is
presented as a general plugin API.
