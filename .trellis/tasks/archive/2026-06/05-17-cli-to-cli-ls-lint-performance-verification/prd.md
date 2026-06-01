# CLI-to-CLI LS-Lint Performance Verification

## Goal

Implement the first measurable slice of
`docs/goals/assura-cli-to-cli-ls-lint-performance-verification.md` by making
`assura performance-report` distinguish CLI subprocess rows from in-process
Assura diagnostics and by attaching machine-readable fixture metadata to every
row.

## Requirements

- Emit clearly named row families for `assura-cli`, `ls-lint-cli`,
  `assura-in-process`, `assura:phase:*`, `traversal:*`, and future
  `strategy:*` rows.
- Measure Assura CLI rows by executing an Assura binary as a subprocess against
  the materialized fixture tree and prepared config, not by calling the
  in-process checker API.
- Continue preparing LS-Lint once before measured loops and execute the cached
  LS-Lint binary inside the measured loop.
- Include fixture metadata in machine-readable report output: fixture id,
  cohort, source type, checked file count, ignored file count, directory count,
  rule count, rule surface summary, native LS-Lint parity, Assura config path,
  LS-Lint config path, config generation method/shared config id, and expected
  exit statuses.
- Label synthetic stress and traversal-only rows as diagnostics so website and
  PR evidence cannot accidentally treat them as headline rows.
- Preserve existing performance-report behavior and output compatibility where
  practical while adding explicit fields.

## Initial Slice

This task starts with report schema and measurement plumbing only. Website
layout, visual review, final architecture decision, and PR body updates remain
open until the generated data contract is correct.

## Verification

- `cargo fmt --all -- --check`
- `git diff --check`
- focused tests for performance-report schema and row families
- `cargo run --quiet -- performance-report --output <artifact> --iterations 1`
- later full goal gates from the goal document before final completion
