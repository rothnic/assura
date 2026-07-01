---
id: goal-assura-ls-lint-performance-reassessment
type: goal
title: Assura LS-Lint performance reassessment
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ./assura-performance-floor-and-fixture-gate.md
  - ../analysis/
  - ../../benches/history/current.json
  - ../../website/public/data/performance/current.json
---

# Assura LS-Lint Performance Reassessment

## Objective

Reassess Assura's end-to-end CLI and warm-session performance against native
LS-Lint after the post-beta daemon, document graph, Markdown, agent, and editor
work lands, and remove any unexplained case where Assura is slower on accepted
fixture rows.

## Current Gap

The beta performance evidence shows the no-slower headline gate passing for
accepted realistic-equivalent fixture rows, but the broader question remains:
why does any Rust Assura path have a CLI floor that can look worse than a Go
LS-Lint path, and which part of startup, config loading, walking, matching,
rule evaluation, reporting, or benchmark harness overhead owns that cost?

## Scope

- Audit current `benches/history/current.json`,
  `website/public/data/performance/current.json`, benchmark manifests, fixture
  definitions, and CI gates.
- Re-run native LS-Lint and Assura comparisons on every accepted fixture row
  with cold and warm measurements.
- Attribute overhead by phase: process startup, config loading, traversal,
  glob/pattern matching, rule evaluation, report formatting, and harness
  overhead.
- Fail CI for any accepted LS-Lint-equivalent fixture where Assura is slower
  than native LS-Lint unless the fixture is explicitly reclassified with
  reviewer-approved evidence.
- Identify and fix root causes without making normal CLI usage cumbersome.
- Update performance docs and checked benchmark history with the final
  explanation.

## Non-Goals

- No hiding slower rows behind aggregate speedups.
- No benchmark-only CLI path that users cannot actually run.
- No performance claim based on warm daemon state when comparing cold one-shot
  CLI behavior.
- No changing fixture definitions to pass gates without review.

## Definition Of Done

- Every accepted LS-Lint-equivalent fixture row is no slower than native
  LS-Lint in the checked comparison.
- Any previous CLI-floor confusion is explained with phase-level evidence.
- Performance gates fail merges for slower accepted rows.
- Website performance data, benchmark history, and docs agree.
- Independent review confirms the comparison is fair, reproducible, and not
  hiding slow paths behind aggregate or warm-only numbers.

## Validation Commands

```bash
cargo bench
cargo xtask performance-report
cargo xtask target-state
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm accepted fixture rows are compared row-by-row against native
  LS-Lint.
- R2: Confirm phase-level attribution explains CLI-floor cost.
- R3: Confirm CI gates fail slower accepted rows.
- R4: Confirm benchmark docs match checked artifacts.

## Reviewer Blocking Criteria

Block if any accepted fixture row is slower than LS-Lint, if aggregate speedups
hide row failures, if warm daemon measurements are used as cold CLI proof, if
fixture rows are reclassified without evidence, or if the remediation makes the
normal CLI cumbersome.
