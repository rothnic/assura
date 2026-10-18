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

The coordinator accepted the independent design and starts bounded TDD. Keep
one small private listener/receive seam; do not make the watch engine generic
for test injection. Explicitly distinguish deterministic lifecycle-model
controls from a direct unchanged-production or OS-driver reproduction. Review
both actual call sites to ensure they retain and borrow the same listener.

Safe real Windows Ctrl-C coverage is a pre-existing proof gap: the current
process-group setup disables Ctrl-C, and a console-wide broadcast could affect
the harness. Preserve the existing real Ctrl-Break integration, compile and
review the retained Ctrl-C path, and record the isolated-console proof as a
separate follow-up. Do not build or broadcast a new console-control harness in
this candidate. The original Mac timeout remains causally unproven.

The root admits focused unit/integration validation with one Cargo job and
explicit test threads under a fresh resource gate; a separate SP04 full PR
uses two jobs. No benchmark or full NU16 PR run is admitted yet. Stop on an
unexpected gate failure and preserve the exact process handle and receipt.

