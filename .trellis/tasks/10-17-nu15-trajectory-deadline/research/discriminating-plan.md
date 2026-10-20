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

When `ASSURA_TRAJECTORY_DIAGNOSTIC_TEST_OUTPUT=1` is set in the test process, the existing labeled scenario forwards only its captured diagnostic stderr to the test harness. Windows-only CI probes use `--nocapture` to expose successful timings after the unchanged full suite. These extra runs are diagnostic only; they preserve the original full-suite failure and every scenario assertion.

The correction turn's tool output reported these local macOS results: nine trajectory tests passed in 8.10 seconds with inspection totals 154/182/189 ms; five process-runner deadline/cleanup tests passed; one bounded-label test passed; and the exact all-features CI probe passed 1/1 with inspection totals 156/171/190 ms. The source and workflow edits were complete before those commands, and only this plan file changed before commit `f0140f4187a98e476c9cdd49dd1b5ba1be479150`.

The focused current-candidate Mac proof was later captured durably at `/Users/nroth/workspace/.assura-review-20261017/nu15-exact-2104b0f9/`. Its manifest identifies source candidate `2104b0f991b9715fe097851ec474cace78ddf187`, tree `40516bb95c74162e850e8c2cbd43cb1815ad1105`, rustc/cargo 1.94.1, the warm target, private TMPDIR, and one Cargo job. It records passing 9/9 trajectory tests, 5/5 runner timeout/cancellation/descendant cleanup controls, 1/1 bounded-label test, and the exact all-features scenario 1/1 with visible inspection totals 151/212/186ms. Per-command argv, stdout, stderr, exit status and hashes are retained. This Mac evidence verifies the diagnostic path but cannot resolve the Windows failure.

After receipt reconciliation, the candidate merged current `refs/remotes/origin/master` (`778128a89d086e946ab77b0e68ad6b73a3af71a4`) at `d962e70dc6074c34bd8658f609828b9e1da17325`. The merge contains only NU04 documentation/task files. The diagnostic implementation/test/workflow blobs are byte-identical to `f0140f4187a98e476c9cdd49dd1b5ba1be479150`: `src/cli/agent_trajectory.rs` `1325afc84e12a3a9eaeaa64e9ce71cdcfa92abf3`; `src/cli/agent_trajectory_git.rs` `491c451f5bfa634950ce95ecdc68bdc3de9931ee`; `src/cli/agent_trajectory_git_exec.rs` `def659253e2408a7e34df2db5e3268bfad375b60`; `tests/agent_trajectory_cli.rs` `03e91c394373902afc005d6ae2c9853b485177aa`; `.github/workflows/ci.yml` `f5c0433c7af905b2ec48162d96874df2098b6276`. No tests were rerun for this docs-only base merge.

## Windows run 39096924402: diagnostic result, not cause

The Windows stable Test Suite job `117585228725` on diagnostic candidate `b900a9b2acd5db1386c1d25e246ec09047bfd1c9` passed both the unchanged `cargo test --all-features` suite and the serial exact scenario probe. The raw job log is `/Users/nroth/workspace/.assura-review-20261017/nu15-windows-diagnostic-job117585228725.log` (SHA-256 `8465fb34ba1dc5f0c59ef294b397993f238714ff8cfaee29cb2294932f7626e7`); the typed summary is `/Users/nroth/workspace/.assura-review-20261017/nu15-windows-diagnostic-summary.json`. In the full suite, the nine-test `agent_trajectory_cli` target used default parallelism and passed in 9.64 seconds. The subsequent serial exact scenario passed in 2.50 seconds and reported labeled inspection totals of 531ms, 590ms, and 610ms. Discovery accounted for 378/451/464ms and collection for 139/126/133ms; the 40 child lifecycle-wall records were mostly 37–49ms. Git for Windows was 2.56.0.windows.2, the same version recorded in the original failed run. The complete PR358 rollup is retained at `/Users/nroth/workspace/.assura-review-20261017/nu15-diagnostic-pr358-checks.json` (30 checks succeeded).

These serial timings show no slow child or inspection close to the 2,000ms budget, but they do not expose timings while the other eight integration tests run concurrently. The original post-merge failure ran the same nine-test target with default parallelism, passed eight tests and failed the shallow-clone/worktree scenario after 22.03 seconds; its log contains no inspection label or Git phase, so the failing call remains unknown. A later passing run does not classify that earlier failure as noise and does not prove a product fix.

## One bounded next discriminator

The approved next Windows probe is a serial/parallel comparison of the existing full nine-test `agent_trajectory_cli` target, both with `ASSURA_TRAJECTORY_DIAGNOSTIC_TEST_OUTPUT=1` and `--nocapture`, after the unchanged required full suite:

1. Serial: `cargo test --all-features --test agent_trajectory_cli -- --test-threads=1 --nocapture`.
2. Default parallelism: `cargo test --all-features --test agent_trajectory_cli -- --nocapture`.

The comparison is diagnostic only. If inspection/child timings inflate broadly or cross the deadline only in the parallel run, that implicates sibling-test/platform contention and warrants a separate fixture/concurrency review; it does not identify a specific external process or justify changing the product timeout. If the same call sites are slow in both modes, investigate those call sites and repeated child outcomes before considering a narrow source correction. If both pass comfortably under 2 seconds, retain the named hold: **original failing phase not observed; failure not reproduced on this candidate**. Do not close the bead, claim runner noise, or claim a product fix from a passing pair. Preserve the full suite, all nine fixtures/assertions, the 2,000ms budget, and cleanup behavior.
