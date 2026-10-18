# Bounded Windows deadline diagnosis

## What current evidence proves

- At `ba4804bb6abdadc1ccc12c41397dd2402a914171`, Windows stable CI ran all nine `agent_trajectory_cli` tests; eight passed and the shallow-clone/multiple-worktree scenario failed after 22.03 seconds with `trajectory refresh deadline exceeded` at the shared `json_success` assertion.
- The error is returned in `inspect_at` after `git::discover` and `git::collect`, with a single 2,000 ms default `Instant` deadline. Discovery runs serial metadata/ref/status Git processes. Collection then runs history, pending-count, tree comparison, and pending diff processes when their inputs permit them.
- The test performs three separate CLI inspections: shallow clone, source worktree, secondary worktree. It also creates and removes a worktree. The runner log does not identify which inspection or Git phase consumed the budget.
- Separate test execution is allowed by the existing nine-test suite; concurrency contention is plausible but unproven. The other high-cost status fixture explicitly requests 10,000 ms, so its budget does not establish a product default override for this test.

## Discriminating observation

Add opt-in stderr-only diagnostics controlled by a test-scoped environment label. For each Git child, record a static call-site phase, elapsed milliseconds and coarse outcome. On inspection completion or deadline failure, record the label, total elapsed time and last Git phase. Do not print repository paths, refs, output, or change JSON/stdout. The ordinary invocation emits no diagnostic output. Instrument only the three existing inspections in the failing integration test, preserving every assertion and fixture.

The next normal Windows CI run will distinguish:

- one slow child (high child elapsed or timeout outcome) from cumulative serial process cost (many bounded child durations, total crossing the deadline);
- a slow shallow-clone/worktree phase from another inspection label;
- a process-start/cleanup concern from expensive Git work (by locating elapsed time in the child and whether it returned `TimedOut`);
- runner/test contention as a possibility only if child and total timings shift broadly; timing alone will not establish causality.

## Stop and decision rules

- Do not retry the unchanged failing command or infer runner noise.
- Do not raise the 2,000 ms default, skip tests, weaken coverage assertions, add a new workflow, or claim that diagnostic timings are product performance evidence.
- If evidence isolates avoidable duplicate subprocess work, make one narrow shared-collector correction and preserve the current negative controls. If the evidence instead implicates fixture setup or test parallelism, propose only an evidence-backed fixture adjustment that retains all scenario coverage. If the diagnostics only show general machine contention with no supported correction, retain the failure and request a bounded reproduction approach from the owner.
- Linux/macOS results can check logging behavior and output compatibility but cannot close the Windows platform proof.
