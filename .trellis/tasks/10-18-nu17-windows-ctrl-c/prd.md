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


## Hosted compile correction (2026-10-20)

PR374's first Windows job failed before running tests: the root integration
target's bare module declaration searched tests/owned_process.rs instead of
the approved nested helper. Three independent read-only reviews identified
that path error and the associated type-inference diagnostics. The declaration
now names the exact private helper path. Unused imports left after extraction
were removed. The separate hosted formatting failure is corrected through
workspace cargo fmt, with the ordinary import/wrapping changes retained.
Neither change alters console membership, deadlines, receipt validation or
cleanup assertions. The original failed logs remain external under
nu17/hosted-failure-d15697a0-20261020.

The earlier local all-targets gate passed1484 tests, zero failed, one manual
performance audit ignored. That macOS run did not execute the Windows-only
protocol. Corrected-head independent review and actual Windows CI remain
required; no Windows behavioral pass is claimed.


## Hosted runtime correction (2026-10-20)

Head5ee185cc compiled and its ordinary Windows suite passed, but the focused
positive helper protocol timed out before READY. The retained failure is at
external nu17/hosted-failure-5ee185cc-20261020. Three read-only diagnoses
identified missing child diagnostics. A subsequent exact-argv Rust libtest
canary reproduced the same-line test-name prefix before READY, which the
strict parser rejected while the helper waited for GO. Root inserts a leading
newline to frame the marker independently, preserving exact matching, all
deadlines, membership checks and receipt requirements. Bounded stdout/stderr
and actual helper/Job/count diagnostics now accompany the original error.
No Windows success or Ctrl-C acceptance is claimed before corrected hosted
execution; the prior failed run remains unfavorable.


## Configured watcher fixture correction (2026-10-20)

Head4bf0356e now reached READY, but the watch stream disconnected before its
initial report (helper exit101, owned Job empty). The bounded diagnostics
exposed that stage; Ctrl-C was never sent. Three read-only diagnoses identified
the missing project configuration, and the retained source-bound Mac canary
reproduced missing-config exit4. Adding a finite Markdown naming policy gave
a successful initial watch report, while a planted BadName.md failed check1.
The helper now uses that configured nonempty fixture and preserves bounded
watcher stderr and observed cleanup status on protocol failure. Existing
deadlines, exact console/Job ownership and positive/no-send receipt gates stay
unchanged. Mac fixture proof does not establish Windows Ctrl-C acceptance.


## Final receipt framing correction

The f450 Windows run passed the focused Rust test and emitted an actual
positive Ctrl-C receipt (ACK, event0, watcher exit0 in5ms) and passing no-send
control with both owned Jobs empty. The outer PowerShell gate still failed:
libtest prefixed the final receipt on the test-name line. Three independent
diagnoses support the same leading-newline framing at the final output,
preserving strict StartsWith extraction and all receipt assertions. Earlier
failure stays failed. Separate Ubuntu adoption smoke passed, then artifact
finalization failed ECONNRESET; this requires no source correction and is not
a successful hosted job. Corrected-source hosted gates remain required.
