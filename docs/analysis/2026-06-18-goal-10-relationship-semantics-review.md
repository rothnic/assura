---
id: analysis-2026-06-18-goal-10-relationship-semantics-review
type: analysis
title: Goal 10 relationship semantics review
status: active
created: 2026-06-18
owners:
  - assura-maintainers
related:
  - docs/goals/assura-goal-10-relationship-semantics-hardening.md
  - .trellis/spec/assura/config-notation.md
  - tests/structure_config_notation_tests.rs
  - src/config/config/structure_notation/relationships.rs
  - src/cli/check/custom_constraints.rs
---

# Goal 10 Relationship Semantics Review

## Revalidation

Result: `valid`.

Live repo inspection confirmed Goal 10 was still needed before implementation.
The canonical notation existed and basic relationship tests covered a captured
source/test counterpart and package docs providers, but the checked surfaces did
not yet prove these Goal 10 edge cases:

- ambiguous or duplicate provider declarations;
- overlapping file and section provider alternatives;
- missing counterpart/provider diagnostics that name the declaring structure
  entry;
- same-name captures reused in separate scopes without cross-wiring
  counterpart requirements;
- provider-only artifacts staying out of the producer set.

## Behavior Decisions

- Captured paths with `provides:` only are provider artifacts, not implicit
  producers.
- Required captured children inside a captured directory are ordinary structure
  requirements, not relationship counterparts for the directory itself.
- When a producer has same-scope captured counterparts, Assura pairs those
  local counterparts before considering cross-tree counterparts.
- If a producer still has multiple counterpart candidates after that
  disambiguation, config loading fails instead of silently choosing one.
- Duplicate provider alternatives for the same need, capture set, path,
  section, and kind fail config loading as ambiguous.
- Relationship diagnostics now include the producer path, source pattern,
  declaring source structure entry, expected counterpart/provider path, provider
  kind, and provider declaration.

## Evidence

- `cargo test structure_notation --quiet` passed after the relationship
  compiler tests were updated.
- `cargo test --test structure_config_notation_tests --quiet` passed after the
  runtime diagnostics and relationship fixture tests were updated.
- `cargo run --quiet -- performance-report --output
  target/performance/current.json` passed and produced a bounded local report
  with 392 result rows. The relationship changes add constant-size metadata to
  normalized relationship constraints and keep provider lookup within the
  existing relationship validation pass; no broad traversal or optimization
  behavior was changed.
- `cargo run --quiet -- check --format json .` passed after relationship
  runtime helpers were split into `src/cli/check/custom_constraints/` to keep
  the self-enforced line-count policy green.
- Independent review found no blockers. Follow-up tests now cover provider-only
  captured entries not becoming counterpart producers and ambiguous cross-tree
  counterpart candidates failing config loading.
