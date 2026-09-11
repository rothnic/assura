# Preserved recovery history

This file retains the 5b03c7f A07 protocol and candidate-bound preparation
entries moved from the active recovery index after PR #280 advanced
`origin/master` to `961dced`. They remain valid historical no-credit evidence;
do not route live work from this file.

## Historical A07 protocol review — 2026-09-11 UTC (`origin/master=5b03c7f`)

- Owner/phase: `/root/a07_protocol_review` / `review`; the private review
  artifact records `PASS` with no mandatory findings for the 5b03c7f packet.
  The reviewer inspected exactly the requested metadata records and returned
  the reviewed SHA-256 values for the identity control, canary, manifest,
  mapping, receipts, public evaluator summaries, six-handle construction,
  binding and second-readonly confirmation.
- The review confirms current source/tree/binary/shim/prompt/public-contract/
  toolchain agreement; exactly two conditions with one differing input value;
  both receipts and full evaluator aggregates; six unique holdouts with
  immutable creation times/evidence and second-readonly `pass`; a complete
  `3 × 2 × 5 = 30` reserved matrix; and private/no-credit boundaries. Raw
  evaluator output, fixtures and child transcripts were excluded as required.
- The manifest, binding, construction and confirmation now carry protocol
  `PASS`, but screening authority and all allocation/acceptance flags remain
  false. Residual limits are explicit: this is metadata-only and the reviewer
  did not independently rehash public-contract bytes. No screening, holdout,
  follow-up-feature, release, deployment, publication or invitation action is
  authorized by this review.
- Exact next action: refresh `origin/master`, release/tag, PR/CI, topology and
  the revision-pinned ledger, then prove the revision still matches this
  packet; on mismatch record contract/location/failure/smallest verification,
  freeze a new candidate, rerun the no-credit canary, rebind holdout/manifest
  and obtain isolated protocol `PASS` before separately authorized screening.
  Preserve the private packet and do not allocate a cell from this protocol
  `PASS` alone.

## Historical A07 candidate-bound preparation — 2026-09-11 UTC (`origin/master=5b03c7f`)

- Owner/phase: process coordinator `/root` / `candidate-bound-canary`; clean
  owned worktree `/private/tmp/assura-a07-current-5b03c7f` was built from the
  refreshed `origin/master=5b03c7f41df1fe8ecf9ae5168eaa11f550b8721f`. The
  root's unknown A04 note and all foreign or historical topology exceptions
  were preserved. Local capacity and the exact Rust/Cargo `1.94.1` toolchain
  were sufficient, so no VPS handoff was needed.
- Candidate freeze and login-shell identity control match source SHA, Git tree
  SHA, absolute binary, version and shim. The wrong-path identity control
  intentionally returned exit `97`; the result is retained as a negative
  protocol control. A malformed evaluator invocation using an unsupported
  dimension token was retained as failed no-credit evidence and corrected by
  consulting the evaluator's declared dimensions.
- Two separate source-only fixtures beneath sibling-free parents completed the
  composed initialization route. Both full evaluator results are
  `verification_scope=full`, `acceptance_eligible=true`, `acceptance_pass=true`,
  zero critical failures, all seven dimensions `pass`, expected negative policy
  rejection and collected native test. These are candidate-bound no-credit
  canaries only; no 30-cell screening or 18-run holdout credit is implied.
- The private manifest is exactly two conditions with one input variable,
  separate mapping, 30 reserved cells and private receipts. Six opaque
  holdouts are rebound with immutable creation records, exact toolchain,
  current candidate identity, second read-only confirmation and raw-hook
  exclusion. The manifest, binding and confirmation deliberately remain
  `PENDING` for independent protocol review; screening authority is false.
- Exact next action: independent isolated review of the current packet using
  the review brief. If it returns concrete findings, record the contract,
  location, failure scenario and smallest fix, repair privately and rereview.
  If it returns `PASS`, refresh current source/ledger once more before asking
  for the separately authorized screening decision. Do not allocate, publish,
  release, deploy, invite, or close A07 from this preparation evidence.

## Historical reconciliation — 2026-09-10 (`origin/master=8cabc536`)

- A fresh read-only reset fetched `origin/master=8cabc53658530a239c00a0c55cbae9b050ad74ca`.
  This is the merge of PR #237 from reviewed head
  `19c5941a8717f05f305032be56080f626fdd06b5`; the prior PR #236 merge
  `35cce811` and its `5329abd` follow-on baseline are historical.
- The reconciled slice changed only live process pointers and task execution-
  branch metadata. It did not alter product source, A07 scoring, private
  conditions, holdout allocation, performance thresholds or authority
  boundaries. A07 remained active with no screening, holdout or acceptance
  credit.
- The owned checkout was `/private/tmp/assura-current-master-pointer-refresh`
  on `docs/current-master-pointer-refresh`, based on `8cabc536`; the root
  unknown path and existing detached/stale registrations were preserved.
- Local process gates, the evidence-only CI classifier and independent review
  passed for the reviewed pointer slice; the first docs gate failed only because
  the disposable checkout lacked `website/node_modules`, then passed after the
  locked install. Hosted/product/private-manifest/canary proof was not claimed.
- Required completion was scoped rereview, applicable hosted checks on the
  exact final head, merge, post-merge reachability and clean owned closure;
  strict topology exceptions from preserved unknown dirt or stale registrations
  were retained as limitations rather than represented as green.
