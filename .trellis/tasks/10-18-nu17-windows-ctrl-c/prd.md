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
  outer PID absent, live child and IsProcessInJob on actual watch handle.
- One shared ten-second event deadline covers helper ACK and successful actual
  child exit. Job query success with ActiveProcesses zero, direct child reaping,
  fallback terminate/wait on every path, bounded streams/writer joins and no
  watch artifact are mandatory; retain fixture until cleanup established.
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

## Evidence and phase

Preparation base a1d2f0aaf405a8d7e6359eb4c42aaaa4251a599e.
External method: /Users/nroth/workspace/.assura-review-20261017/nu17/ready-slice-proposal-v2.md
Independent review: /Users/nroth/workspace/.assura-review-20261017/nu17/design-v2-independent-review.md
Root corrections: /Users/nroth/workspace/.assura-review-20261017/nu17/root-design-v2-disposition.json
Source-only implementation first. No local Cargo while NU03 owns build lane;
root releases validation separately. All unexpected gates preserve raw and stop
for three read-only triages. Historic Mac timeout remains unexplained.
