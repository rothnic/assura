---
id: goal-assura-ls-lint-counterexample-closure
title: Close LS-Lint Counterexample Compatibility And Performance Gaps
status: completed
created: 2026-05-26
analysis:
  - docs/analysis/2026-05-26-ls-lint-counterexample-challenge.md
  - docs/analysis/2026-05-26-ls-lint-rule-coverage-audit.md
---

# Goal: Close LS-Lint Counterexample Compatibility And Performance Gaps

## Objective

Make Assura meet the full LS-Lint compatibility, Assura-extended LS-Lint
notation, and real-time feedback claims for the counterexamples found in
`docs/analysis/2026-05-26-ls-lint-counterexample-challenge.md`.

This goal must improve implementation and evidence. Do not close it by reducing
claims, labeling LS-Lint-valid configs unsupported, or moving the gaps into
roadmap language.

## Required Work

1. Preserve and verify Assura-extended exact `exists` notation.
   - Keep support for scalar exact file and directory requirements such as
     `README.md: exists:1`, `src/: exists:1`, and package-scoped
     `AGENTS.md: exists:1`.
   - Document and test this as an Assura extension to LS-Lint notation, not as
     upstream LS-Lint behavior.

2. Correct LS-Lint targeted path semantics without losing native recursion.
   - Add tests for full-tree, targeted file, and targeted directory runs.
   - Provide an explicit LS-Lint target-semantics mode that matches
     `@ls-lint/ls-lint@2.3.0`.
   - Keep recursive targeted-directory checks as Assura's native real-time
     feedback behavior.

3. Correct root `.dir` `exists` behavior.
   - Match LS-Lint for root `.dir: exists:1` and `.dir: exists:0`.
   - Keep root, child, glob-scope, full-engine, fast-path, and compiled-path
     behavior aligned.

4. Remove multipart-extension performance explosion.
   - Replace exponential `lslint_extension_candidates` behavior with a linear or
     bounded lookup strategy.
   - Cover long multipart extension rules such as
     `.a.b.c.d.e.f.g.h.i.j.k.js`.

5. Expand performance coverage.
   - Add the multipart-extension fixture and the many-configured-scope fixture
     to the release evidence path.
   - Use pinned native LS-Lint from `@ls-lint/ls-lint@2.3.0`.
   - Compare rebuilt release binaries, including `assura-check`.

## Acceptance Checks

Run and record:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --quiet
cargo build --release -p assura --bins -p assura-check-cli
target/release/assura performance-report \
  --output target/performance/ls-lint-counterexample-closure.json \
  --history target/performance/ls-lint-counterexample-closure.jsonl \
  --website-dir target/performance/ls-lint-counterexample-closure-website \
  --iterations 5
cargo run --quiet -- check --format json .
```

Evidence must show:

- Assura-extended exact `exists` configs require files/directories as intended
  and are labeled separately from upstream LS-Lint behavior.
- Explicit LS-Lint target-semantics checks match LS-Lint results while native
  targeted-directory checks still recurse for agent feedback.
- The multipart-extension fixture no longer makes Assura materially slower than
  native LS-Lint.
- The many-scope fixture is included in performance evidence and is not slower
  than native LS-Lint.
- Existing LS-Lint rule coverage and parity tests continue to pass.

## Review Criteria

- Review the implementation against upstream LS-Lint behavior, not only local
  assumptions.
- Confirm that Assura-native features and LS-Lint compatibility are separated
  cleanly when their semantics differ.
- Check that agent hot-path use cases are covered by tests for changed files,
  changed directories, and full-project checks.
- Verify that performance improvements are measured with release binaries and
  native LS-Lint, not cargo/debug runs.

## Progress Log

- 2026-05-26: Started implementation pass. Reconciled the goal with the
  intentional Assura exact `exists` extension, added targeted regression
  coverage for root `.dir exists`, exact file/directory requirements, and
  explicit LS-Lint target semantics, and began performance-regression closure.
- 2026-05-26: Added counterexample fixtures to the performance-report evidence
  path and updated audit wording so exact scalar `exists` is treated as a
  required Assura extension, not a claim to soften or remove.
- 2026-05-26: Closed the measured counterexample regressions with bounded
  multipart-extension matching, indexed fast-scope lookup, configured-structure
  pruning, and a parallel large-scope fast walk. Rebuilt `assura`,
  `assura-full`, and `assura-check-cli`, then regenerated
  `target/performance/ls-lint-counterexample-closure.json`.
- 2026-05-26: Independent review found stale performance evidence and skipped
  warm rows for long fixture IDs. Fixed both by rebuilding all release binaries
  after the final source changes and shortening hot-daemon Unix socket names.
  Current evidence shows `warm_claim_summary.two_x_claim_verdict=complete`,
  while the broader local macOS cold `claim_summary` remains `not-complete` and
  should stay separate from the real-time editor-session claim.
- 2026-05-26: Rebuilt the release binaries after module splitting and
  regenerated `target/performance/ls-lint-counterexample-closure.json`.
  Current counterexample rows show `multipart_extension_regression` at
  `10.162 ms` for `assura-cli` versus `11.946 ms` for native LS-Lint, and
  `many_configured_scopes_regression` at `29.503 ms` for `assura-cli` versus
  `43.064 ms` for native LS-Lint. Warm/editor-session evidence remains complete
  across all eight realistic-equivalent fixtures with `27.189x` aggregate
  speedup; local macOS cold aggregate remains explicitly non-complete at
  `1.316x`.
- 2026-05-26: Follow-up independent review found two more compatibility
  blockers: non-`exists` scalar path keys were still converted into child
  file-naming policy, and LS-Lint target mode still enforced direct child
  counts for targeted directories. Added regression coverage and fixed both
  while preserving Assura's exact `exists` extension and native recursive
  changed-directory feedback.
- 2026-05-26: Rebuilt release binaries again after the review fixes and
  regenerated `target/performance/ls-lint-counterexample-closure.json` with no
  source files newer than `target/release/assura`.
