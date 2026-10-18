# NU17: Native Windows Ctrl-C proof

## Objective

Prove actual Tokio watch listener Ctrl-C termination on Windows in a private
owned console, without signaling the test harness. Preserve production code,
existing Ctrl-Break/Unix tests, ten-second event-to-exit bound and watch artifacts.

## Authority

Nick authorized iterative Beads execution with independent GPT-6 Luna review,
corrections, applicable gates and root-owned merges. Bead ae-whr.2 is canonical.
Root owns native lifecycle, Beads, push/PR/merge and delivery closure. Worker owns
this candidate only. No release/deployment/publication or broad cleanup authority.

## Scope

Only tests/watch_cli_windows_ctrl_c.rs (new test-only integration target),
.github/workflows/ci.yml and this task/context/evidence. Existing tests/ structure
allows root snake_case files <=1000 lines. No new production module, dependency,
public API, production signal change, timeout relaxation, skip or retry.
Root approved this exact placement because watch_cli.rs has 251 lines headroom.

## Contract

Implement the independently reviewed v2 protocol and root corrections:
- Outer ignored exact test starts a bounded private CREATE_NEW_CONSOLE helper;
  kill-on-close noninheritable Job Object assignment succeeds before one-use GO.
- Clear inherited Ctrl-C ignore before child spawn; callback atomics use separate
  ACK and event sentinel (Ctrl-C is numeric zero). No callback I/O or locks.
- Watch inherits the owned console without a new process group. Before the sole
  zero-group Ctrl-C send, prove complete exact helper/watch console membership,
  outer PID absent, live child, and parent-side IsProcessInJob against the exact
  noninheritable owned job handle after a bounded watcher-PID handshake.
- One shared ten-second event deadline covers helper ACK and successful actual
  child exit. Job query success with ActiveProcesses zero, direct child reaping,
  bounded direct termination with exact-job fallback, bounded streams/reader
  joins and no watch artifact are mandatory; retain fixture until cleanup is
  established.
- Execute no-send control as an outer-test subcase; prove zero send/no ACK/live
  child then owned cleanup. Include it in one final validated receipt.
- Pure safety validator rejects foreign/incomplete memberships before sender;
  receipt validator rejects missing ACK/event/wrong exit/missing receipt.
- Required Windows CI step runs exactly one ignored outer test, captures original
  Cargo exit, rejects zero/multiple tests and missing/duplicate/invalid receipt.
  Keep existing broad test matrix and Ctrl-Break unchanged.

## Acceptance

Actual exact-PR-head Windows focused CI proves console/job/ACK/deadline/cleanup
and both positive and no-send subcases. Existing Ctrl-Break and required matrix
remain green. Meaningful negative controls pass. Independent Luna source and
final-readiness review findings are adjudicated/corrected and rereviewed before
root merge. Source-only/Unix checks do not count as Windows signal proof.
Root verifies postmerge outcome, native close, Bead and owned checkout/ref closure.

## Implementation status

The isolated Windows-only test target is in `tests/watch_cli_windows_ctrl_c.rs`,
with bounded job/process ownership in the closed child file
`tests/watch_cli_windows_ctrl_c/owned_process.rs`; the focused exact-test and
receipt checks are in the Windows test-suite job in `.github/workflows/ci.yml`.
The target has not been built or run locally because the NU03 lane owns Cargo
validation. The retained source-bound Assura binary
passed the full current structure check (exit 0), and external positive/negative
fixtures confirmed exact mappings pass while an unlisted ignored test still
fails (exits 0 and 1). Raw captures are at
`/Users/nroth/workspace/.assura-review-20261017/nu17/structure-gate-policy-20261018`.
Standalone rustfmt and diff checks also pass. Actual Windows signal delivery
remains unproven until the focused required CI step runs.

The source-bound full structure check was recaptured after these corrections:
the latest `final-source-full-check-v*` capture in
`/Users/nroth/workspace/.assura-review-20261017/nu17/structure-gate-policy-20261018`
(exit 0, `success=true`; its receipt binds both Rust test files, workflow,
config and PRD).
The current positive and unlisted-negative policy fixtures also pass their
expected outcomes (0 and 1) under the same retained binary. An initial positive
fixture attempt used the repository root as cwd and is preserved as uncredited;
the corrected fixture-scoped capture is
`final-policy-positive-fixture-39ebfaca-1a7e-4710-aea8-c5fd7386f2e3`, paired
with `final-policy-unlisted-negative-fixture-6c123ac9-b05a-41de-bb88-37333ee885f5`.

The repository's high-severity `supported_surface_tests` policy has exact
ignored-test entries for these two deliberate Windows-only tests: the outer
test is selected by the focused required CI step, and the helper is invoked
only as that outer test's owned child. Their distinct reason categories were
added under a separate root policy decision; the performance-audit exception,
other relationships, and severity remain unchanged. The current root source
review additionally requires exact parent-side job membership proof before the
helper can send, successful initial-event fields in both receipts, and bounded
child/reader termination. Those corrections are in source; no Windows signal
execution is claimed until the focused required CI step completes.

## Evidence and phase

Preparation base a1d2f0aaf405a8d7e6359eb4c42aaaa4251a599e.
External method: /Users/nroth/workspace/.assura-review-20261017/nu17/ready-slice-proposal-v2.md
Independent review: /Users/nroth/workspace/.assura-review-20261017/nu17/design-v2-independent-review.md
Root corrections: /Users/nroth/workspace/.assura-review-20261017/nu17/root-design-v2-disposition.json
Current source review disposition: /Users/nroth/workspace/.assura-review-20261017/nu17/root-source-review-disposition-71e4af1e.json
Source-only implementation first. No local Cargo while NU03 owns build lane;
root releases validation separately. All unexpected gates preserve raw and stop
for three read-only triages. Historic Mac timeout remains unexplained.
