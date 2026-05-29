# Define LS-Lint rule coverage goal

## Goal

Create a referenceable Assura goal file that scopes a detailed LS-Lint rules and test coverage audit, with special focus on regex, exists, wildcard extension, directory pattern, and migration parity behavior needed for the agentic feedback roadmap.

## What I already know

* The user wants a goal file they can reference by name/path, not copy-paste prose.
* The goal must review LS-Lint supported rules in detail, especially regex and exists behavior.
* The goal must determine whether upstream LS-Lint has rule tests that Assura does not currently cover.
* Assura already has LS-Lint parity/performance history and tests under `tests/ls_lint_parity_regression_tests.rs`, `tests/realistic_lslint_fixtures.rs`, and `src/config/ls_compat.rs`.
* Current Assura coverage now includes extension rules, `.dir`, multiple naming alternatives, ignore, direct-child exists counts, exact filename exists as an Assura extension, regex negation/substitution, LS-Lint extension-combination precedence, repeated config merging, `--warn`, and glob/brace directory scopes.

## Requirements

* Research current LS-Lint 2.3 docs and upstream source/tests.
* Compare upstream rule semantics against Assura config, migration, and tests.
* Define concrete audit outputs and acceptance criteria in a new `docs/goals/` goal file.
* Make the new goal path explicit in final output.

## Acceptance Criteria

* [x] Goal file exists under `docs/goals/` with frontmatter, objective, scope, requirements, validation commands, and evidence artifacts.
* [x] Goal requires an upstream LS-Lint rule/test matrix.
* [x] Goal requires identifying upstream LS-Lint tests not currently mirrored by Assura.
* [x] Goal separates supported LS-Lint parity, Assura extensions, and any newly discovered gaps that must be fixed rather than claimed away.
* [x] Goal ties coverage completeness to the agentic hot-path roadmap.

## Out of Scope

* Publishing packages in this task.

## Technical Notes

* Active spec: `.trellis/spec/assura/index.md`.
* Dormant template specs under `.trellis/spec/backend` and `.trellis/spec/frontend` are not active Assura guidance.
