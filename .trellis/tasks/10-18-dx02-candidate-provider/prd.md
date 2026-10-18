# DX02: Bounded current candidate provider reads

## Problem

Bead ae-i4d. Delivery record currently fetches identity, reviews and checks for every repository PR. W02 original checks query exited 1 with invalid JSON; v2 checks query exited 1 with HTTP 504. Both failed record commands exited 2 and left generation 0 intact. A separate successful full checks read took 30.731s and 2,061,535 bytes; a separate targeted PR368 probe took 0.776s and 7,164 bytes. These observations establish cost and failure exposure, not a general speed guarantee or original provider root cause.

## Authority

Nick authorized the active Beads execution goal, independent GPT-6 Luna review and corrections for each implementation, and reviewed current-base merges including disclosed website deployment coupling. Root owns Beads, pushes, PRs, merge and native production receipt writes. dx02-candidate-provider owns this clean candidate only. No release/tag/publication beyond already authorized merge coupling, private evaluator, foreign work or human adoption authority is implied.

## Scope and ownership

Root prepares and approves this contract, dispatches one Luna implementer, judges independent review and integrates. Source ownership is existing .trellis/scripts/common/delivery.py, delivery_cli.py and delivery_status.py, tests/agent_delivery_contract_tests.py, the scoped delivery-lifecycle spec and this task. Add another existing delivery module only with an evidenced necessity; no new generic layers, dependencies or public flags. Other agents work in separate checkouts; do not revert their work.

Implement an explicit candidate read path for record and close. Resolve a uniquely mapped PR from a complete current identity cohort using existing typed task/ref/tip mapping, then fetch current details for that exact PR. Do not select identity from submitted evidence or silently fall back after a failed query. Keep audit/next/checkpoint/inspect/archive on their existing fleet inventory contract unless an independently reviewed contract amendment explicitly admits a narrower consumer. Full-cohort audit semantics and its failure behavior remain unchanged.

## Acceptance

- Successful candidate path makes one complete identity query plus one exact-number current details query, without requesting other PR reviews or checks. Requested number/head/base/ref/state/merge identities must correlate with discovery, local current tip and existing evidence predicates. Race, malformed or incomplete data fail closed.
- Candidate coverage is represented explicitly and cannot be mistaken for complete repository review/check coverage or satisfy another candidate. Preserve complete cohort bound, ambiguity refusal and exact merged historical-base rules. Missing or duplicate identities, mismatched number/head/base, invalid detail field shapes, provider failure and selected pending/failed/skipped jobs never advance a receipt or fabricate successful coverage.
- Provider failures report bounded safe context identifying query stage and unavailability; preserve failure status and prevent secret/control-character leakage. Identity mismatch remains distinct from provider unavailability. Task/receipt bytes and generation remain unchanged on rejected record/close attempts.
- Preserve existing compare-and-swap, authority/binding checks, terminal immutability, local evidence semantics, release/aggregate/non-delivered outcomes, remote advertised-base checks, review independence and exact selected run/job/name/URL checks. Existing strict fleet reads remain no-fallback and fail closed.
- Meaningful RED/GREEN candidate-path regression plus positive and negative query/identity/coverage/no-write controls. Run the complete delivery and process Python suites and cheap workflow/context/diff/structure/evidence gates after candidate settles. Hosted process contracts must execute on Ubuntu/macOS/Windows; scope skips are uncredited. No full Rust suite is required solely for Python/spec changes; inspect current CI selection.
- Independent Luna source/proof review, accepted corrections with affected checks and scoped rereview, then root merge. One exact real candidate operation with unchanged input must prove supported provider behavior after implementation is reviewed; retain both original failures and do not call new success their diagnosis.

## Evidence and method

External immutable evidence root: /Users/nroth/workspace/.assura-review-20261017/dx02/. Existing source/environment design reports are under ../w02-current/dx02/; adversarial report location is linked when available. Preserve original W02 command receipts under ../w02-current/integration-976481b7/ and exact provider captures under ../native-gh-observation/. Never edit production receipts or substitute cached/manual provider data. New observer/verifier support for the explicitly changed query shape must independently validate real capture argv, exits, identity and coverage; old three-projection verifier is not applicable without a reviewed change.

Use disk-backed private 0700 TMPDIR for test fixtures and bounded Python child processes. Preserve real outer tool handles, raw output, exits, source/tree/tool identities and teardown. No broad recursive process/disk probes or global setup changes.

## Current state

Planning at base 74d9917dd39e599bbd43c3a5760b71644b2cf91b. Two failed W02 records remain generation 0. No production implementation or receipt mutation has been admitted yet. One task-create invocation stopped before writes because the new worktree lacked a developer identity; the documented --assignee option supplied the candidate owner on successful creation. No test or validation credit is claimed for setup.

## Approved design

Root accepted the three independent source/environment/adversarial designs. Decision and exact evidence links: /Users/nroth/workspace/.assura-review-20261017/dx02/design-adjudication-v1.md. Candidate-scoped record/close only; strict fleet/inspect/archive paths preserved. Implementation may proceed after supported task start; no production receipt write by the implementer.

The approved structure boundary places GitHub protocol, query construction,
provider shape checks, and provider errors in leaf
`.trellis/scripts/common/delivery_github.py`, which imports no delivery domain,
status, audit, or archive module. The delivery collector passes normalized
short branch names and owns candidate mapping and error translation. The
candidate GitHub coverage predicate belongs beside other evidence predicates
in `delivery_evidence.py`. Keep the task CLI and full-fleet audit contract
unchanged.

## Implementation evidence checkpoint

Post-split `python3 tests/agent_delivery_contract_tests.py` output records 110
tests passing in 80.321 seconds, with child exit 0. The durable output and
exit record are under
`/Users/nroth/workspace/.assura-review-20261017/dx02/implementer-20261018/`
as `delivery-final.log` and `delivery-final.exit`. This run includes the
fresh-process import-order controls. Its recorder then raised
`ProcessLookupError` because it queried the child's process group after
`communicate()` had reaped it; this was a recorder error after the child test
run, not a test failure. The recorded start hash covered only `delivery.py`,
not the whole candidate; root must run the final clean-SHA delivery and process
validation before approval.

The earlier full raw log for 107 tests with two failures was overwritten by
that post-split run and is lost. Three contemporaneous read-only triages
attested the earlier outcome, but its raw output is unavailable and is not
claimed as retained. The correction was scoped to explicit valid CheckRun
fixtures and typed CheckRun/StatusContext rollup validation; malformed union
controls remain rejecting. This is the only full delivery-suite result after
the split; root's final whole-input delivery and process validation is still
pending.

The post-split structure check `assura check .` exited 0 with 16 nonblocking
max-line findings; captured output and exit are in the same external evidence
directory under `structure-final-20261018T123351Z.*`. The workflow gate
reported `blocked_dirty_worktree` because these eight owned paths are
uncommitted; that was an ownership prerequisite, not a code/test failure.

## Root correction checkpoint

Independent review of d89dc899 accepted REV1 strict merge snapshot coherence and
REV2 unfinished CheckRun/conclusion coherence. Candidate-only provider checks
now enforce both. Root fixed two test-fixture details before validation: the
actual diagnostic regex and a real full tip OID. All seven exact focused tests
passed in 7.412s, including invalid-provider record/close no-write controls.
Evidence: `/Users/nroth/workspace/.assura-review-20261017/dx02/root-focused-corrections-20261018T125647Z-541f31af/`.
The external runner's preceding status-parser attempt aborted before launch
and is uncredited. Full clean-source corrected suites and scoped independent
rereview remain required; earlier raw-loss limitation above remains unchanged.
Root cheap gates (structure, task/context validation, diff and reused xtask
evidence) all exited zero at `../root-final-cheap-20261018T125728Z/dx02/`
under the same external evidence root. No native provider write has occurred.
