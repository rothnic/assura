# NU16 watch signal listener lifetime

## Goal

Retain platform shutdown listeners for the lifetime of the continuous watch
runtime, so cancellation of a `select!` receive future does not drop and
recreate the underlying listener between filesystem events and synchronous
batch handling.

## Authority

- Owner: `nu16-watch-signal-lifetime` for this scoped candidate.
- Coordinator: `/root` is the sole Beads/native delivery writer, pusher, PR and
  integration writer.
- Authorized by parent goal `.trellis/tasks/10-17-assura-beads-execution/prd.md`
  and dispatched implementation scope. Do not register/inspect provider
  delivery, mutate Beads, push, or create a PR.
- Base: `refs/remotes/origin/master`.
- Task/acceptance authority: this document's `## Authority` and
  `## Acceptance` sections.

## Evidence and source boundary

The source research and independent design/proof review are in
`/Users/nroth/workspace/.assura-review-20261017/nu03/`:
`signal-listener-lifetime-research.md`, `signal-listener-design-review.md`,
`watch-listener-bead-description.md`, and `watch-stop-environment-triage-16fd.md`.
The original hosted Mac `watch_stop10s` timeout remains unexplained. Tokio's
listener-subscription loss is a source-supported mechanism, not proven as the
cause of that failure. Preserve this distinction in tests and reporting.

## Approved behavior

- Construct private platform listener(s) once for each watch runtime, before
  its initial event when straightforward.
- Keep the listener alive across both existing `select!` sites and all
  synchronous batch validation/emission between them.
- Borrow cancel-safe `recv` operations from the retained listener in both
  shutdown branches. Preserve Unix SIGINT, Windows Ctrl-C and Ctrl-Break,
  contextual setup errors, and the existing Ctrl-Break `BrokenPipe` mapping.
- A retained signal is observed at the next async select. This does not promise
  interruption of synchronous work.

## Deterministic test protocol

The deterministic regression must target the same narrow private listener
construction/receive seam used in production, without a generic watch-loop
rewrite. A private fake source models only broadcast to current subscribers:

1. On the retained path, start a borrowed `recv`, let the filesystem side win
   and cancel that receive future, represent synchronous handling, publish a
   signal, then prove the next borrowed receive on that same listener observes
   it. Assert one listener construction/subscription and delivery.
2. Negative control: drop the listener, publish while no listener exists,
   recreate it, and prove that signal is absent. Assert the recreate/subscriber
   counts. Do not call this proof of Tokio driver scheduling.
3. Keep the current owned-child integration's real Unix SIGINT / Windows
   Ctrl-Break success, unchanged ten-second deadline, no-runtime-artifact
   assertion, and guaranteed child cleanup.
4. Add Windows Ctrl-C coverage only after proving a child-owned console control
   route that cannot broadcast `GenerateConsoleCtrlEvent(CTRL_C_EVENT)` into the
   harness console. Do not use an unsafe process-group broadcast or claim fake
   coverage as OS coverage.

No sleeps, timeout increases, blind retries, production debug hooks, or
unbounded harnesses. A public-API child timing race cannot prove Tokio broadcast
occurred during the unsubscribed interval. Run the final Unix/Windows matrix
after the coordinator releases the resource and test-method hold.

## Scope and constraints

Expected implementation ownership is limited to `src/cli/watch.rs`,
`src/cli/watch_signal.rs`, focused tests adjacent to the existing watch signal
tests, and `tests/watch_cli.rs` only for justified platform coverage. Do not
add public APIs, dependencies, modules, debug flags, retries, sleeps, timeout
changes, or weaken gates. Do not edit implementation or launch Cargo/build/test
commands until `/root` explicitly releases the resource/method hold.

## Acceptance

- One listener is registered for each watch runtime and retained across both
  select sites and synchronous batch handling; each site uses its borrowed
  receive operation and setup errors retain context.
- Deterministic retained-path positive and dropped/recreated negative controls
  exercise the production seam and assert lifecycle counts. Their proof claim
  is limited to retention/cancellation wiring.
- Existing real owned-child success, ten-second bound, no-artifact assertion,
  and cleanup remain intact. Windows Ctrl-C is covered only with safe owned
  console control delivery; otherwise record the unsupported proof gap.
- The original Mac timeout remains classified `unknown` unless new causal
  evidence proves otherwise. No regression proof is described as establishing
  its historical cause.
- Independent Luna review and corrections, focused proof and required
  Unix/Windows readiness gates are completed before integration by `/root`.

## Current phase

Implementation and focused verification are complete; the candidate awaits
independent review and the coordinator's readiness decision. The production
change constructs one private platform listener before the initial event,
retains it across synchronous work, and borrows its `recv` operation at both
select sites. A private `ShutdownSignal` seam supports two deterministic model
controls: cancellation followed by delivery on the same retained subscription,
and the dropped/recreated negative control. These tests establish lifecycle
wiring only.

Validation was run against working-tree source based on `c10d12f4` with
`CARGO_TARGET_DIR=/Users/nroth/workspace/.nu16-watch-build.qlMSs0/target`,
`TMPDIR=/Users/nroth/workspace/.nu16-watch-build.qlMSs0`, one Cargo job, and one
Rust test thread:

- The existing `watch_stops_cleanly_without_runtime_artifacts` child test
  passed: 1 passed, 0 failed, 0 ignored, 15 filtered out; 1.31 seconds. Its
  original ten-second bound, success assertion, no-artifact assertion, and
  child cleanup are unchanged.
- The two lifecycle controls passed: 2 passed, 0 failed, 0 ignored, 554
  filtered out.
- Scoped `cargo clippy --lib --test watch_cli -- -D warnings` passed with no
  warnings.

Durable launch metadata, raw outputs, atomic exits and teardown records are in
`/Users/nroth/workspace/.assura-review-20261017/nu16/run-watch-stop-20261018T113819Z`,
`run-lifecycle-tests-20261018T113919Z`, and
`run-scoped-clippy-20261018T114058Z`. An earlier invocation lost its session
handle; its PID is gone and its dedicated target has no holders, but it has no
durable exit status and remains unknown/uncredited. Its old process-group ID
was unavailable and is not inferred.

Safe real Windows Ctrl-C coverage remains a separately tracked follow-up. This
candidate retains both Windows listeners and reviews the Ctrl-C path, while the
existing real owned-child integration continues to exercise Ctrl-Break. No new
console-control harness was built or broadcast. Windows execution/compilation
and the broader readiness matrix remain unrun. The original Mac timeout also
remains causally unproven. Do not describe either model controls or this Mac
owned-child pass as proving the historic timeout's cause. No full NU16 PR gate,
push, or integration was run.

## Coordinator checkpoint at current base

This checkpoint supersedes the earlier readiness statements above. Independent
implementation review at01058222 and current-base rereviews at a9ba6990 and
05198dea found no remaining source issue. The candidate is based on reviewed
master31c043eb; the two watch source files remain identical to01058222. Parent
task child lists were merged by union with original stage records retained.

One local `cargo xtask pr` completed with observed tool and child exit0 at
a9ba6990 using Rust1.94.1, two build jobs/test threads, a private short APFS
TMPDIR and checkout-local target. Its captured raw output contains1471 passed,
0 failed,1 manual ignore across110 harnesses;8 empty harnesses and the ignore
receive no test credit. Clippy and the48-page website build passed. All2262
tracked inputs, tree, HEAD and clean state matched before/after. Proof and
actual outer handle3762 are under
`/Users/nroth/workspace/.assura-review-20261017/nu16/full-pr-a9ba6990-20261018T1320/`.
The root census reader initially assumed an optional interrupted-receipt field
was always present; three read-only triages supported a state-specific reader
correction. The original normal-complete receipt and raw output were unchanged;
no producer change or test rerun was made. The historical lost handle remains
unknown and is not repaired by this later successful invocation.

The current-base merge changes only reviewed DX03 skill/helper and task
metadata. A manifest compares869 Rust/Cargo/xtask/test/website/config inputs
unchanged from the tested source; full PR proof is reused with scoped Luna
review, rather than repeated. Exact raw census differences from older W02 and
SP04 candidates reconcile to their different source tests. Records are under
`current-base-31c-20261018T133629Z/` and `census-reconciliation-a9ba6990.md` in
the same external evidence root.

The exact final PR head still requires hosted gates, including Windows
compilation and the existing real owned-child Ctrl-Break test. Safe real
Windows Ctrl-C proof remains NU17. No new timeout/retry or platform broadcaster
was added, and the historical Mac timeout's cause remains unproven. This is
engineering validation only; no push, merge or release is claimed here.
