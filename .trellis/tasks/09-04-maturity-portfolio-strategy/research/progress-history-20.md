# Preserved progress history

This file retains historical progress entries moved from the active index to
keep the configured line limit intact. Do not route live work from this file.

## Iteration 49 — 2026-09-07 — Complete diagnostic-consumer correction

- PR #185 head 1852b61 passed final local PR tier 82802 (exit 0), then failed
  a distinct hosted root-debug assumption. The root path is `[""]`, not `[]`;
  hosted report mode/count are unknown. No stale-predecessor explanation was
  invented. Linux/Windows fail-fast cancellations are not passing platform proof.
- R01 fb724fd audits all diagnostic readers and validates complete FIFO batches
  with separate full-root and incremental contracts. Actual reader RED/GREEN,
  26 watch tests, 20 watch units and 40 stop-on-failure focused runs passed.
  Exact support-module structure allowance retains limits and negative control.
  Independent review and exact-head gates are pending; PR remains held.
- Hosted many-scopes performance failed at 19.364834 ms versus LS-Lint
  17.6041085 ms, 7/8 accepted comparisons, 392 retained rows. Native/warm
  passes do not clear it. Wider-stack profiling produced 3,430 samples with
  zero lost but still unreliable callers; attribution must use a discriminating
  probe.
- No green rerun, release, deployment or scope reduction was used for closure.

## Iteration 48 — 2026-09-07 — Hook diagnostics and context health

- PR #184 `d42094c` failed Ubuntu executable-hook launch with OS 26
  (`Text file busy`); 23/24 ownership and 10/10 lifecycle tests passed.
  macOS/Windows were cancelled, not passed. Five adoption smoke lanes and
  performance passed but cannot replace the failed suite/platform proof.
- Passing Linux traces show close-before-rename and direct wrapper/sidecar
  execution, excluding a persistent leaked writer in those observations, not
  a transient race in the failing hosted schedule. A writable-inode negative
  control reproduced errno 26. No production source cause is established.
- Approved bounded failure-only Linux diagnostics: invoke the installed
  wrapper once, capture executable identity and same-inode writer evidence,
  then return failure. Exclude command lines/environments, bound enumeration
  and label missing/racy data. No retries, sleeps or shell-launch bypass.
- R03 quiet probe on `vps-9cb01956`, session 10542 exit 0, found continuing
  Node/Temporal/background activity. No benchmark was started, no services
  stopped, and no exclusive-runner or historical-noise claim was made.
- Context level: not exposed. Master `3d9a255`; reviewed R01 `1852b61` has
  final PR-tier bootstrap running; A04 owns diagnostics; A05 is preserved;
  inventory remains 48 worktrees. Repeated failures require exact stream
  association and discriminating execution evidence, not green reruns.
  Executable helpers and existing goal/local-build/performance skills are the
  appropriate homes; no new skill or AGENTS expansion is warranted.
- Next: R01 integration, A04 diagnostic review, current-master rebase and
  fresh hosted proof without waivers.

## Preserved historical tail

Iterations 44–47 and the historical index are preserved in
[`progress-history-14.md`](progress-history-14.md). Do not route live work
from that file.
