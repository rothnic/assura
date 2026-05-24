# Execute LS-Lint realistic parity and core performance goal

## Goal

Execute `docs/goals/assura-ls-lint-realistic-parity-core-performance.md`
end-to-end: realistic LS-Lint parity fixtures, production `jwalk` traversal,
performance hotspot investigation, incremental/cache research, stored
performance evidence, website reporting, validation, push, and draft PR.

## Requirements

- Use branch `codex/ls-lint-realistic-parity-core-performance` from an
  up-to-date `master`.
- Keep progress in the goal document before and after major phases.
- Define and implement the good-enough comparison contract before relying on
  benchmarks or compatibility claims.
- Reuse realistic fixtures across tests and benches; materialize pinned
  external fixture sources through a manifest rather than vendoring full
  third-party repositories.
- Switch production `assura check` traversal from `walkdir` to `jwalk` while
  preserving deterministic, sorted report output and exclusion semantics.
- Profile current-product hotspots and either implement obvious wins or record
  exact deferred next steps.
- Produce machine-readable performance results and chart-ready history, plus a
  website or preview surface that links to the data.
- Consolidate notation source truth before implementing future notation
  extensions.
- Run required validation commands or document exact blockers.
- End with pushed branch and draft PR URL.

## Initial Slice

1. Add the good-enough comparison contract and pinned fixture manifest/harness
   scaffolding.
2. Add realistic reusable generated fixture families and tests that prove the
   compatibility matrix against valid and invalid shapes.
3. Migrate production traversal to `jwalk` with focused deterministic-output
   and exclusion-pruning regression coverage.

## Acceptance

- The final release checklist in the goal document is complete.
- Every demonstration criterion in the goal maps to independently verified
  evidence: file artifact, test, benchmark output, website build, command
  output, pushed branch, and draft PR.
- Any unmet criterion is represented as a documented blocker with exact
  command output and next action.
