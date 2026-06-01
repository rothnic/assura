# Implement PR 11 Performance Amendment

## Goal

Complete the performance amendment for PR #11 by replacing the current
"serial jwalk measured" endpoint with a measured production traversal strategy
that includes a real parallel `jwalk` path, preserves `assura check`
correctness, and fixes benchmark validity so LS-Lint warm measurements do not
mostly measure repeated npm/package startup.

## What I Already Know

- PR #11 is `https://github.com/rothnic/assura/pull/11`.
- The active branch is `codex/ls-lint-realistic-parity-core-performance`.
- The current remote head fetched locally is
  `6c86b03a67815e85ff02c684227d01dc64998777`.
- Local checkout is on the PR branch and tracks
  `origin/codex/ls-lint-realistic-parity-core-performance`.
- `.codex/config.toml` is an unrelated local modification and must not be
  bundled into product changes unless explicitly requested.
- The existing amendment document is the source of truth for this continuation:
  `docs/goals/assura-ls-lint-realistic-parity-core-performance-amendment.md`.
- `assura check` currently uses `jwalk::Parallelism::Serial` in
  `src/cli/check.rs`.
- `assura performance-report` currently reports top-level Assura/LS-Lint rows,
  phase rows, a walkdir traversal row, and a single jwalk traversal row.
- The current LS-Lint report path invokes `npm exec --package ... ls-lint` in
  each measured iteration, which can skew warm LS-Lint comparisons toward npm
  startup and package resolution overhead.
- This session is running with `CODEX_SANDBOX_NETWORK_DISABLED=1`; git network
  and `.git` metadata writes require escalation in the current process.

## Requirements

- Implement a production traversal strategy that includes a measured parallel
  `jwalk` path.
- Preserve deterministic text/JSON output, stable relative paths, exclusion
  pruning, fail-fast behavior, sorted violation output, and all existing
  parity/CLI behavior.
- Keep fail-fast deterministic. If parallel validation conflicts with
  deterministic fail-fast, use a deterministic serial fail-fast path and a
  parallel normal path.
- Avoid shared mutable hot-path state during parallel validation where
  practical; prefer immutable check context plus independently collected
  per-entry results.
- Keep regex and glob compilation outside per-entry hot paths.
- Preserve or improve exclusion pruning before expensive validation work.
- Extend `assura performance-report` so report rows distinguish walkdir, serial
  jwalk, parallel jwalk, top-level Assura/LS-Lint, and Assura phase timings.
- Fix warm LS-Lint measurement by resolving/installing LS-Lint once and
  executing the cached binary directly in measured loops, or clearly separating
  cold npm invocation metrics from warm tool metrics.
- Fix performance history append behavior so appending does not require reading
  the entire existing history file.
- Ensure website performance data writes the intended history/current data
  without accidentally replacing history with only the latest run.
- Preserve symlink behavior when materializing external fixtures.

## Acceptance Criteria

- [ ] Production `assura check` includes a real parallel `jwalk` path or an
      adaptive strategy that can select parallel `jwalk`.
- [ ] Deterministic output, pruning, and fail-fast semantics are covered by
      focused traversal regression tests.
- [ ] Performance report includes separate rows for walkdir, serial jwalk, and
      parallel jwalk.
- [ ] LS-Lint warm measurements avoid repeated `npm exec` package resolution in
      the measured loop, or cold npm timing is explicitly labeled separately.
- [ ] Performance results show improvement over the current PR baseline for
      rule-heavy validation and at least one traversal-heavy realistic fixture,
      with no silent stable-baseline regressions.
- [ ] Existing LS-Lint parity and realistic fixture tests continue to pass.
- [ ] Required verification commands from the amendment are run or blockers are
      recorded with exact error text and next action.
- [ ] The PR body is updated with final before/after numbers, chosen traversal
      strategy, and links to machine-readable performance artifacts or checked
      in data.

## Out Of Scope

- Implementing a full incremental cache.
- Expanding Assura notation beyond the PR #11 LS-Lint parity/performance goal.
- Changing unrelated Codex sandbox or hook configuration unless explicitly
  handled as environment setup outside the product commit.

## Technical Notes

- Primary goal docs:
  - `docs/goals/assura-ls-lint-realistic-parity-core-performance.md`
  - `docs/goals/assura-ls-lint-realistic-parity-core-performance-amendment.md`
- Performance context:
  - `docs/analysis/2026-05-15-performance-architecture-statement.md`
  - `src/cli/check.rs`
  - `src/cli/performance_report/mod.rs`
  - `src/cli/performance_report/traversal.rs`
  - `src/cli/performance_report/io.rs`
- Relevant specs:
  - `.trellis/spec/assura/index.md`
  - `.trellis/spec/assura/structure-enforcement.md`
  - `.trellis/spec/assura/tooling-stabilization.md`

