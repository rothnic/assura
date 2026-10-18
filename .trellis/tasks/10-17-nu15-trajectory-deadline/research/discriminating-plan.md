# Bounded Windows deadline diagnosis

## What current evidence proves

- At `ba4804bb6abdadc1ccc12c41397dd2402a914171`, Windows stable CI ran all nine `agent_trajectory_cli` tests; eight passed and the shallow-clone/multiple-worktree scenario failed after 22.03 seconds with `trajectory refresh deadline exceeded` at the shared `json_success` assertion.
- The error is returned in `inspect_at` after `git::discover` and `git::collect`, with a single 2,000 ms default `Instant` deadline. Discovery runs serial metadata/ref/status Git processes. Collection then runs history, pending-count, tree comparison, and pending diff processes when their inputs permit them.
- The test performs three separate CLI inspections: shallow clone, source worktree, secondary worktree. It also creates and removes a worktree. The runner log does not identify which inspection or Git phase consumed the budget.
- Separate test execution is allowed by the existing nine-test suite; concurrency contention is plausible but unproven. The other high-cost status fixture explicitly requests 10,000 ms, so its budget does not establish a product default override for this test.

## Discriminating observation

Add opt-in stderr-only diagnostics controlled by a bounded test label read once per inspection. For each Git child, record a static call-site phase, elapsed milliseconds and coarse outcome. Child timings are inclusive process-lifecycle wall time across command setup, spawn, child execution, output handling, and termination/cleanup; they do not isolate Git execution or process startup/cleanup. On each inspection completion or deadline failure, record the label, total elapsed time, discovery/collection stage times, and last Git phase. Keep a fixed upper bound on event/stage buffers. Do not print repository paths, refs, output, or change JSON/stdout. The ordinary invocation emits no diagnostic output. Instrument only the three existing inspections in the failing integration test, preserving every assertion and fixture. Emit buffered events only after the measured inspection completes so diagnostic output itself does not consume the refresh deadline.

The next Windows CI run will distinguish:

- one slow child (high child elapsed or timeout outcome) from cumulative serial process cost (many bounded child durations, total crossing the deadline);
- a slow shallow-clone/worktree phase from another inspection label;
- per-child wall time alone cannot attribute delay to Git, Windows process startup/cleanup, or test-runner contention;
- runner/test contention only as a possibility if child and total timings shift broadly; timing alone will not establish causality.

## Stop and decision rules

- Do not retry the unchanged failing command or infer runner noise.
- Do not raise the 2,000 ms default, skip tests, weaken coverage assertions, add a new workflow, or claim that diagnostic timings are product performance evidence.
- If evidence isolates avoidable duplicate subprocess work, make one narrow shared-collector correction and preserve the current negative controls. If the evidence instead implicates fixture setup or test parallelism, propose only an evidence-backed fixture adjustment that retains all scenario coverage. If the diagnostics only show general machine contention with no supported correction, retain the failure and request a bounded reproduction approach from the owner.
- Linux/macOS results can check logging behavior and output compatibility but cannot close the Windows platform proof.

## Independent review disposition

The review at `nu15-diagnostic-independent-review.md` accepted the scoped diagnostic probe but required three corrections before CI use:

- F1 accepted: Git-child timings are labeled process-lifecycle wall time and no longer claim to distinguish Git work from Windows setup, output handling, termination, cleanup, or runner contention.
- F2 accepted: both successful completion and deadline failure emit the total inspection duration plus buffered discovery/collection stage durations.
- F3 accepted: the environment label is read once per inspection, only bounded safe labels (at most 32 ASCII characters) are accepted, and event/stage buffers have explicit limits. With diagnostics inactive, Git calls perform no timer reads or environment lookups.

The first local serial run at diagnostic candidate `8ff1fce427b7aea2d19e52db5587fef8eef44778` passed all nine `agent_trajectory_cli` tests, but successful subprocess stderr was captured by the Rust test harness and did not appear in the command log. The targeted integration assertions verified default silence and the opt-in stage/phase records. This Mac result is not Windows evidence.

When `ASSURA_TRAJECTORY_DIAGNOSTIC_TEST_OUTPUT=1` is set in the test process, the existing labeled scenario forwards only its captured diagnostic stderr to the test harness. A Windows-only exact-test CI step can use `--nocapture` to expose successful timings after the unchanged full suite. The extra run is diagnostic only; it preserves the original full-suite failure and every scenario assertion.

After the accepted corrections, the focused Mac invocation `ASSURA_TRAJECTORY_DIAGNOSTIC_TEST_OUTPUT=1 CARGO_TARGET_DIR=/Users/nroth/workspace/.assura-review-20261017/nu15/target CARGO_BUILD_JOBS=2 TMPDIR=/Users/nroth/workspace/.assura-review-20261017/nu15/tmp cargo test -p assura --test agent_trajectory_cli -- --test-threads=1 --nocapture` passed all nine tests in 8.10 seconds. The labeled scenario emitted successful records for all three inspections, with total elapsed times 154 ms (`shallow_clone`), 182 ms (`primary_worktree`), and 189 ms (`secondary_worktree`); the ordinary command path still checks for silence. The focused process-runner tests `cargo test -p assura --lib agent_trajectory::git::exec::tests:: -- --test-threads=1` passed 5/5, including aggregate-deadline, timeout, cancellation, and descendant cleanup controls. These are macOS-only instrumentation and control results; they do not explain or resolve the Windows failure.

The exact command proposed for the Windows CI step was also exercised locally with `--all-features`: the named scenario passed 1/1 and emitted successful inspection totals of 156 ms, 171 ms, and 190 ms. It remains macOS-only evidence.
