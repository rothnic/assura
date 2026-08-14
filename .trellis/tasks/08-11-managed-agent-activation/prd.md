# Managed Agent Activation For Four Hosts

## Goal

Make onboarding able to explicitly activate Assura feedback in Codex, Claude
Code, OpenCode, and Pi through one ownership-safe lifecycle.

## Public Contract

```text
assura agent onboard . --agent auto --activate --format json
```

Activation is explicit. Generated bundles remain thin adapters over stable
Assura commands and schemas.

## Requirements

- Detect the requested/active host without guessing when evidence is ambiguous.
- Install project-local bundles and patch only Assura-owned host configuration.
- Support install/activate, status, doctor, update, deactivate/remove, and dry-run.
- Preserve unmanaged files and report conflicts with concrete remediation.
- Record adapter contract/version, managed hashes, host event mapping, and logs.
- Prove at least one real supported event payload per host.

## Acceptance Criteria

- [x] Codex lifecycle and event fixture pass.
- [x] Claude Code lifecycle and event fixture pass.
- [x] OpenCode lifecycle and event fixture pass.
- [x] Pi lifecycle and event fixture pass.
- [x] Repeated activation is idempotent.
- [x] Update repairs stale managed files and rejects unmanaged drift.
- [x] Removal deletes only Assura-owned configuration and files.
- [x] Onboarding JSON distinguishes generated, activated, verified, and conflicted.

## Evidence

Validated on 2026-08-14 against the isolated VPS worktree snapshot:

- `cargo test --test agent_integration_cli`: 9 passed.
- `cargo test --test agent_surface_cli`: 18 passed.
- `cargo test --test project_intelligence_onboarding --test agent_onboarding_config_merge`: 16 passed.
- `cargo fmt --all -- --check`: passed.

## Validation

```bash
cargo test --test agent_integration_cli
cargo test --test agent_surface_cli
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
cargo run --quiet -- check --format json .
```

## Review Blocking Criteria

Block on whole-file host config replacement, false auto-detection, host-specific
policy logic, missing rollback, unbounded feedback, or undocumented event gaps.
