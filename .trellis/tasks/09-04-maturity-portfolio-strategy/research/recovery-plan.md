# Execution recovery plan

Status: process recovery in progress, 2026-09-09. This is a plan and audit,
not evidence that product cards passed. Coordinator: this thread's process
owner, branch `goal/execution-recovery-plan`. Product changes stay in their
separately owned card slices.

## Outcome and success gates

The PRD's outcome remains dependable repository conventions and agent-assisted
setup, with credible portfolio evidence. Work is successful when the selected
slice's observable contract passes, independent findings are resolved, required
local/hosted proof covers current source, integration is verified, and owned
branches close cleanly. A card is done only when its full packet outcome exists.

For A07 this retains screening, untouched holdout and final acceptance as
separate allocations; final acceptance is 10 runs per stack, at least 9/10 per
stack, zero destructive overwrites and zero critical misses. Follow-up feature
proof and full evaluator dimensions remain mandatory. CI, generated files and
partial checks cannot substitute. R06/publication/pilots retain their separate
authority and observed-outcome requirements.

## Corrected baseline

Refreshed integration source: `da773bce8315cbb9e69941f0fe89d0a91e42829d`.
Master Rust CI `34310994162`, Documentation `34310994191` and Security
`34310994120` report success at that SHA. PR #227 merged; its scoped init
guidance contract is not proof of A07 success. PR #196 is already merged and
is an A04 diagnostic, while R03 completion is recorded through PR #197.
The old heartbeat's PR identity and failure narrative are unsuitable for
candidate selection. Preserve failed historical evidence without reviving it.

The previous thread review read `docs/maturity-portfolio-strategy` at
`f3a8f66`, not the current ledger. Its claim that only B00 was recorded done
was therefore stale. Current master records A07 active and R03/A04/A05/A06
done. R01, W02 and F01 retain specific held outcomes; W03 is verified with
publication outstanding. Reconcile packet-level contradictions before promotion
(including A06's R01 dependency), using source and reviewed evidence.

Unknown root dirt remains untouched. `goal/a07-composed-init` is clean at
`da773bc` with no product patch yet. Other clean historical goal branches need
ownership and reachability classification, not blanket deletion. The global
topology report exits 128 on a missing worktree; record incomplete coverage
and use the read-only fallback before proposing a targeted audit repair.

## Ordered recovery slices

| Priority / owner | Action | Exit proof / next action |
| --- | --- | --- |
| 1 / process coordinator | Integrate this recovery plan, lean AGENTS router, execution/validation references and corrected scheduler routing | Independent process review and decision scenarios; structure/evidence and applicable hosted checks; verify merge and close this branch |
| 2 / train coordinator | Reconcile active A07 evidence with retained private results and confirm live owner of composed-init worktree | One current checkpoint; no hidden evaluator material in public artifacts; select smallest public behavior contract |
| 3 / A07 implementation owner | Test explicit composed init proposal against public semantics; plain init remains config-only; preserve conflicts and honest runtime permission reporting | Focused negative/positive controls, review, required final-source local/hosted gates; merge slice only if its own goals pass; rerun blinded protocol before A07 completion |
| 4 / independent R01 investigator | Read latest R01 packet/evidence and rejected diagnostics; determine missing causal observable before spending another platform run | Concrete hypothesis, distinguishing observation and reviewed method amendment; do not repeat prior unchanged diagnostic or invent native readiness |
| 5 / topology owner | Inventory existing goal branches, live owners and base reachability; repair audit parser as a separately tested slice if needed | Complete report; merged clean branches removed by exact identity or verified archive; unknown work preserved |
| 6 / release/portfolio coordinator | Prepare R06/W02/W03 and later cards against actual dependencies and observed trigger coupling | Complete authorized local preparation; specific authority remains attached only to publication/deployment/invitation action |

Priorities express dependency and urgency, not compulsory single-lane execution.
R01 investigation/topology preparation can run while A07 review or CI waits,
with explicit owners and resource capacity. Do not spawn a worker just to poll.
This process slice does not implement A07, performance changes, releases or CI
policy changes. Those require their own recorded acceptance and review.

## Instruction audit and corrections

| Observed defect | Durable correction |
| --- | --- |
| Old checkout mistaken for current canonical ledger | Goal skill requires revision comparison and current-base task reads |
| Long AGENTS includes stale Rust 1.70, generic examples and setup advice | Shared router points to manifest, skills and actual specs; retain universal ownership/review/authority rules and managed Trellis block |
| Goal skill only routes `docs/goals`, while train lives in Trellis | Add Trellis routing and checkpoint/merge contract references |
| Pending-only queue selection misses unfinished candidates | Inspect active/implemented/verified candidates first, then ready pending rows |
| Strong prose but repeated idle heartbeat replies | Resume exact checkpoint; after two unchanged wakes inspect liveness/state and choose a diagnostic; notifications stay quiet when appropriate |
| Environment recipe repeats unrelated suites and chained cwd errors | Resume original applicable command, use root-relative package setup and validation matrix |
| VPS diff helper destroys/reuses labeled directories and lacks clean-commit identity | Use isolated exact-commit bundle procedure; no automatic helper promotion |
| Evidence-policy check mistaken for outcome proof | Require card consumer/negative evidence separately from `cargo xtask evidence` |

Read layers: AGENTS → goal skill → selected card/current checkpoint → required
phase reference → exact source/log. Do not load all historical packets or
evaluator internals into implementers. Current facts live here and in card
evidence; reusable rules live in skills. No second scheduler/ledger is added.

## Validation efficiency and VPS decision

Read-only SSH audit: `vps-dev` did not resolve; configured `vps` reached a
16-CPU Linux host, 62,787 MiB total / 43,688 MiB available memory, load below
1 at observation. Disk: 94% used, 23 GiB free. Default Rust is nightly
1.95; Node 22.22.1 and pnpm 10.29.3 were present. This shows available CPU/RAM,
not measured speedup or validated build capacity. No heavy remote job ran.

Use the goal skill's validation matrix and local-build VPS procedure. Start
with one isolated job after checking target-size headroom and explicit matching
toolchain. Keep platform-specific CI and idle-host benchmark controls. Never
clean others' artifacts to obtain disk space. Capture three prepared candidate
measurements before proposing cache/concurrency/runner changes.

`cargo xtask pr` includes `fast`; avoid unchanged back-to-back runs. Existing
hosted scope reuse verifies successful prior jobs and falls back to full checks;
preserve it. `cargo xtask evidence` checks repository evidence policy, not card
acceptance. Do not change workflow coverage or performance thresholds here.

## Verification and continuation

Process review must exercise: stale checkout vs current ledger; local green
with failed required hosted performance; live CI timeout; process-only scope;
VPS nightly/low disk; clean branches with unknown dirt elsewhere; passing slice
while A07 acceptance fails. Expected decisions follow the execution contract.
Record actual reviewer decisions and limitations in `recovery-evidence.md`.

Next action: validate and independently review this committed process slice,
resolve findings, run applicable hosted checks, integrate and close its owned
branch. Refresh runtime goal state through the supported product tools; the
latest read returned no goal, so no automatic goal continuation is claimed.
Do not replace or complete an unfinished goal to repair a status mismatch.
