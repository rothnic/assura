# Release Contract Rule First Slice

## Goal

Implement the first reusable Assura release-contract rule slice so a project can
declare expected release artifacts and have `assura check` report drift between
the contract, release workflow assets, install scripts, docs, and checksum
sidecars.

This follows the revalidated `docs/goals/assura-rule-release-sync.md` boundary:
build configurable product validation for other repositories, without moving or
weakening Assura's repo-local `cargo xtask target-state` governance.

## What I Already Know

- Goal 13 already made Assura's own release and performance evidence checks
  deterministic through `cargo xtask target-state`.
- The remaining gap is a reusable configured rule family, not another
  hard-coded Assura release check.
- The first slice should prefer an explicit configured contract over broad prose
  inference.
- Required coverage includes passing and failing fixtures for docs, workflow
  assets, install URLs, and checksum sidecar drift.
- JSON diagnostics need actionable file and contract context.

## Assumptions

- The minimal notation can live under the existing Assura config model rather
  than introducing a plugin API.
- The first slice may parse configured files with conservative text/YAML
  scanning if the parser is deterministic, fixture-backed, and not presented as
  broad natural-language inference.
- Package metadata drift can be deferred unless the existing architecture makes
  it cheap to include without diluting workflow/docs/checksum coverage.

## Requirements

- Define a minimal explicit release-contract config surface.
- Validate that configured docs/installers do not mention unsupported artifact
  names or unsupported release URL branches.
- Validate that configured workflow files publish every contracted artifact.
- Validate checksum sidecar expectations in workflow and docs/installers when
  the contract requires sidecars.
- Add independent fixtures that do not depend on Assura's own release docs.
- Add CLI integration coverage proving the rule runs through `assura check`.
- Preserve all existing target-state checks.

## Acceptance Criteria

- [ ] Passing fixture covers all configured artifacts, workflow uploads, docs,
  installer URLs, and checksum sidecars.
- [ ] Failing fixture covers docs mentioning an asset absent from the workflow
  contract.
- [ ] Failing fixture covers installer URLs pointing at unsupported branch or
  asset names.
- [ ] Failing fixture covers a workflow upload missing a required `.sha256`
  sidecar.
- [ ] `assura check --format json` includes release-contract rule identifiers
  and file/contract context.
- [ ] `cargo xtask target-state` remains green without removing Goal 13 checks.

## Definition Of Done

- Implementation and fixtures are committed on this task branch.
- Required validation passes:
  - `cargo fmt --all -- --check`
  - `cargo test --all-targets --quiet`
  - `cargo xtask target-state`
  - `cargo run --quiet -- check --format json .`
  - `git diff --check`
- A review agent has reviewed the implementation against the goal's blocking
  criteria before PR creation.
- A GitHub PR is opened, hosted checks and review comments are addressed, and
  the PR is merged before moving to the next goal.

## Out Of Scope

- Release publishing automation.
- GitHub API or remote release inspection.
- Broad natural-language parsing of arbitrary release prose.
- Removing or replacing current `cargo xtask target-state` checks.
- Full package metadata policy unless it falls out naturally from the minimal
  rule architecture.

## Technical Notes

- Canonical goal: `docs/goals/assura-rule-release-sync.md`.
- Config notation constraints: `.trellis/spec/assura/config-notation.md`.
- Structure validation expectations: `.trellis/spec/assura/structure-enforcement.md`.
- Roadmap context: `.trellis/spec/assura/roadmap.md`.
