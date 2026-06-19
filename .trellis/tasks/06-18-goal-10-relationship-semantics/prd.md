# Goal 10 Relationship Semantics Hardening

## Objective

Complete `docs/goals/assura-goal-10-relationship-semantics-hardening.md` on
branch `codex/goal-10-relationship-semantics`, then review, PR, address
comments, merge, and continue from a clean workspace.

## Current Gap

Canonical relationship notation exists, but relationship-heavy policies are not
yet proven across the edge cases users will hit first: ambiguous providers,
overlapping provider kinds, missing counterparts, same-name captures in
separate scopes, and diagnostics that identify the declaring structure entry.

## Scope

- Revalidate that Goal 10 is not already achieved and record the result in a
  review artifact or progress log.
- Add focused fixtures/tests for relationship happy paths and failure modes.
- Improve relationship diagnostics so users can identify the producer, missing
  provider or counterpart, need/provider kind, and declaring structure entry.
- Keep provider and counterpart declarations where the artifacts live in the
  tree.
- Update public examples and docs touched by relationship semantics.
- Preserve rejection of removed alpha notation and avoid compatibility shims.
- Record bounded performance evidence for relationship notation changes.

## Non-Goals

- No dependency graph validation claim.
- No arbitrary shell validators.
- No broad performance optimization beyond evidence needed for this behavior.
- No compatibility support for removed alpha relationship notation.

## Definition Of Done

- Required Goal 10 tests and fixtures cover supported relationship semantics,
  including ambiguous/invalid cases.
- Existing relationship passing cases still pass.
- Diagnostics are first-time-user actionable and identify the declaring
  structure entry.
- Public docs describe relationship behavior as Assura-native notation.
- Independent review checks false positives/negatives, provider alternatives,
  removed notation, and docs/examples consistency.
- PR includes the goal link, review evidence, validation evidence, addressed
  comments, and is merged.

## Validation

```bash
cargo fmt --all -- --check
cargo test structure_notation --quiet
cargo test --test structure_config_notation_tests --quiet
cargo run --quiet -- performance-report --output target/performance/current.json
cargo run --quiet -- check --format json .
git diff --check
```

Add narrower commands while iterating when useful, but do not treat them as a
replacement for the required Goal 10 gates.
