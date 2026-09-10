# Preserved maturity execution train progress history

The entries below were moved verbatim from `progress.md` only to keep the live
checkpoint under the repository's 1000-line structure limit. They remain
historical evidence and are not current routing instructions.

## Iteration 21 — 2026-09-06 — Q07 cache recovery attribution repair

- Independent review found that the original corrupt-cache test added a naming
  violation before its recovery assertion, allowing snapshot invalidation to
  explain the fresh result. The revised test repeats against an unchanged
  project, asserts a successful fresh validation and a valid rewritten cache
  record, then separately checks the later naming violation.
- Context-health review: the known Trellis `cargo xtask pr` active-task routing
  discrepancy remains documented rather than waived; Q07 will retain its
  nonzero result if it recurs. The nested launcher package test is bounded and
  useful; no reusable-skill gap is evident.

## Iteration 20 — 2026-09-06 — Q07 launcher error contract

- Q07 started in isolated current-master worktree `goal/q07-error-contract` at `c9ad106`. A focused red test proved the primary launcher silently treated a present, non-executable companion as absent because spawn errors were discarded with `.ok()?`.
- The smallest repair now propagates a path-bearing OS error and runtime exit `1`; the exact focused test passed. The first `--exact` invocation selected zero tests because its nested test path was incomplete and is explicitly excluded from proof. Remaining Q07 work is the report-output failure contract, selected cache fallback audit, packaged-launcher coverage, and required integration/release gates.

## Iteration 35 — 2026-09-06 — R01 hosted watch-regression refinement

- PR #173's first hosted matrix disproved the initial external-config rescan filter: macOS coalesced unrelated external-config activity with Assura runtime-output paths, emitting a false full rescan in both explicit-config and requested-directory watch contracts. The failure is preserved in R01 evidence; no gate was waived and no merge occurred.
- A new focused mixed-event unit test was RED before the correction and is green after it. The refinement ignores only rescan events made entirely of external/config-sibling and Assura-runtime-output paths; excluded in-scope paths and pathless rescans remain observable safety fallbacks. Independent review added a direct pathless-rescan regression; all 16 `cli::watch` units and 14 watch integration tests pass locally.
- Context health: the only repeat was platform-specific watcher event coalescing, now captured by a compact unit contract beside the classifier. Existing watch evidence and test patterns are sufficient; no new skill is warranted. Next: independent review of the new SHA, then a fresh hosted matrix.

## Iteration 36 — 2026-09-06 — R01 requested-scope sequence correction

- The corrected R01 hosted matrix passed Linux but macOS exposed a remaining test-sequencing defect: a helper intentionally accepts a successful, pathless full-rescan event for an out-of-scope FSEvent, then the test incorrectly expected the subsequent in-scope mutation to retain sequence 2. The helper now returns the next expected sequence (2 when quiet, 3 after the accepted safety fallback); it does not weaken either event’s content contract.
- The focused requested-directory test, complete 14-test watch integration suite, 16 `cli::watch` units, formatting, diff, and structure gates pass locally. Next: independent review of this exact test-contract repair, then a fresh hosted matrix.

## Iteration 37 — 2026-09-06 — R01 external-config rescan integration

- PR #173 merged the independently reviewed external-config rescan repair as `b3002b1`; the exact commit was fetched and verified reachable from `origin/master`. Its full hosted matrix passed, including macOS/Windows/Linux suites, MSRV, performance, release/installer/adoption, documentation, security, coverage, and evidence gates.
- The correction keeps watcher safety explicit: unrelated external config and Assura runtime-output noise no longer produces false rescans, while pathless and excluded in-scope rescans remain observable fallbacks. No tag, release, deployment, or public action was taken. Next independent ready card: A02.

## Iteration 38 — 2026-09-06 — A02 explicit local-policy preflight

- A02 is active in isolated current-master worktree `goal/a02-local-patterns`. Focused VPS integration proof covers `init --recipe-file` with a spaced path, SHA-256 provenance in `.assura/onboarding/profile-selection.json`, successful `agent onboard --recipe-file`, and a conflicting local rule with path/existing/incoming diagnostics.
- A new red contract showed onboarding was materializing its baseline before detecting a local-policy conflict, changing project config despite the conflict. The implementation now preflights a local recipe against an existing config before baseline materialization; the four-test focused suite is green and the local Assura structure gate has zero violations. This is not card completion: invalid-result, idempotence, bundled fixture, documentation, broader-gate, and review evidence remain.
- Context health: local disk recovered to 1.7 GiB; VPS remains the compile/test authority and has ample capacity. The only repeated operational friction is explicit source synchronization between the isolated local worktree and the VPS clone; existing evidence commands remain sufficient and no new reusable skill is warranted. Next: add the remaining A02 merge and idempotence contracts.
