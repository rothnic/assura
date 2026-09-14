# Implement Assura delivery closure workflow

## Goal

Make accepted Assura work traceable from task ownership through review,
integration, verification, and cleanup. The process must expose unmerged work
instead of allowing it to disappear behind a completed task, while preserving
paused, foreign, unknown, rejected, superseded, and historical work.

The full implementation handoff is at
`/Users/nroth/Documents/Codex/2026-09-14-assura-delivery-process-plan/docs/superpowers/plans/2026-09-14-assura-delivery-closure.md`.
This PRD carries the executable requirements from that plan.

## What I already know

* The current integration ref is `origin/master`; refresh it before execution.
* The root checkout has preserved user-owned untracked files and must not be
  modified by this task.
* The existing topology script scans local `goal/*` refs but does not classify
  all refs or detached worktrees for integration.
* The existing ownership helper is exercised by tests but is not connected to
  the operational audit.
* `task.py finish` clears active-task state, and task archiving can mark work
  completed without proving that its change was integrated.
* Task progress treats missing children as completed, which hides unresolved
  work.
* CI preview artifacts already carry package version and commit SHA. Durable
  releases are tag-driven and currently require additional binary-version and
  retry verification.

## Requirements

### Candidate contract

* Add a versioned `meta.delivery` intent for new or explicitly migrated tasks.
* Support candidate kinds `integration`, `artifact`, `experiment`, `release`,
  and `aggregate`.
* Keep task intent in `task.json`/PRD and operational observations in a local
  receipt store resolved beneath the absolute Git common directory.
* Keep stable candidate identity separate from changing branch, worktree, and
  commit observations.
* Represent phases `registered`, `implementing`, `review`, `ready_to_integrate`,
  `integrated_pending_verification`, `verified`, and `held`.
* Represent outcomes `delivered`, `superseded`, `rejected`, `cancelled`, and
  nonterminal `unknown`.
* A delivered integration requires accepted criteria, resolved review,
  required checks, merged reachability or verified PR mapping, post-merge
  verification, and no owned uncommitted changes.
* Superseded, rejected, and cancelled outcomes require a reason, decision
  evidence, exact candidate tip, and a recovery or replacement reference.

### Resumable evidence

* Write receipts atomically with generation checks and a process-releasing
  per-candidate lock.
* Detect stale writers and corrupt/unsupported receipts without overwriting
  newer evidence.
* Keep `audit`, `inspect`, `next`, and `checkpoint` read-only. Explicit
  `register`, `record`, and `close` commands are the only receipt mutations.
* Record exact repository, branch/ref, candidate/head, PR, merge, workflow-run,
  tag, release, asset, checksum, and installation identities where applicable.
* Treat unavailable host/API/pagination evidence as unknown or unavailable,
  never as an empty successful inventory.
* Never store secrets or unbounded command output in receipts.

### Inventory and lifecycle enforcement

* Discover local refs, remote refs returned by an explicit refresh, linked
  worktrees, task records, archived task records, and relevant GitHub PR/check
  records.
* Correctly classify normal merges, squash/rebase merges, candidates with
  post-review commits, superseded candidates, archives, and unresolved items.
* Invoke the same ownership and terminal predicates from real commands that
  the tests exercise.
* Guard task close/archive before changing task files, moving directories,
  clearing session pointers, or auto-committing.
* Preserve `finish` as an honest pause/detach operation; it must not imply
  delivery or abandonment.
* Require an owner, live handle or explicit unavailable state, next action,
  and disposition for owned candidates.
* Treat missing children as unknown and distinguish delivered from rejected,
  superseded, cancelled, held, and not-needed states.
* Do not block unrelated independent work because a foreign or pre-existing
  worktree is dirty.
* Do not merge, publish, delete, reset, stash, or execute shell text from
  metadata as a side effect of inspection.

### Idempotent continuation

* Provide read-only `delivery audit`, `next`, and bounded `checkpoint` views.
* Repeated observations with identical inputs produce identical output and no
  task, receipt, Git, or worktree changes.
* Keep the rendered checkpoint to 3–6 bullets and at most 1,024 UTF-8 bytes.
* Replace hardcoded current-SHA routing with a refresh-and-audit route while
  retaining historical SHAs as explicitly historical evidence.
* Surface an owned unresolved candidate before starting replacement work.

### Release path

* Preserve version plus full-SHA CI preview artifact names and existing stable
  installer asset names.
* Assert `assura` and `assura-full` versions before and after packaging for all
  durable release targets, including Windows.
* Verify tag/package agreement, archive contents, checksums, required assets,
  and explicit-version installation.
* Make retries idempotent: matching release assets are accepted, conflicting
  assets stop for investigation, and no silent overwrite occurs.
* Document the version increment procedure without publishing a release as a
  side effect of this implementation.

### Existing-work recovery

* Inventory the current repository and explicitly in-scope hosts after the
  process is implemented.
* Give every in-scope candidate a truthful disposition, owner, evidence, and
  next action.
* Recover useful changes only through small, current-master, reviewed PRs.
* Preserve historical and foreign work; never bulk-merge or bulk-delete refs.
* Require zero newly created unowned candidates and zero unsupported
  completion claims at the end of recovery.

## Acceptance Criteria

* [ ] A real temporary repository test finds a unique `codex/*` branch, a
  detached worktree candidate, and a remote-only ref outside `goal/*`.
* [ ] A squash-merged PR is recognized through verified PR/merge evidence, and
  a later commit on the candidate remains outstanding.
* [ ] `task.py` cannot archive an unmerged integration task and leaves task,
  session, index, refs, and worktree state byte-for-byte unchanged on failure.
* [ ] Pause/resume retains the same candidate and does not create a duplicate
  branch or PR.
* [ ] Rejected, superseded, cancelled, held, and missing-child fixtures do not
  advance delivered product progress.
* [ ] Concurrent receipt updates reject stale generations, survive process
  interruption, and preserve the newest valid receipt.
* [ ] Repeated audit and checkpoint commands produce identical output and no
  side effects; checkpoint output is within the byte/line bound.
* [ ] Focused process tests execute with a nonzero test count through the real
  CLI entry points.
* [ ] Durable release workflow tests reject wrong binary versions, wrong tag
  versions, missing/corrupt assets, checksum mismatches, and conflicting
  retries.
* [ ] Every inventoried in-scope existing candidate has a reviewed disposition;
  any remaining unknown or authority-held item has an owner and next action.
* [ ] The implementation candidate has required independent review, passes
  applicable local/hosted checks, is merged into current `master`, and its
  post-merge workflows pass on the exact merge SHA.

## Definition of Done

* P01–P04 are implemented as one connected workflow capability and are merged
  through one reviewed PR.
* P05 release hardening is merged through a second reviewed PR, unless the
  reviewer demonstrates that the existing release implementation already
  satisfies every criterion and records that judgment.
* P06 recovery is complete or has explicit unresolved coverage/authority
  records; no unresolved item is silently counted as done.
* P07 lifecycle fixtures pass, including pause/resume, merge, squash merge,
  rejection, held release, concurrency, and cleanup cases.
* No goal-created worktree is dirty or left without an owner, next action, and
  terminal disposition.
* No tag, public release, deployment, or external message is created by this
  task without separate authorization.

## Out of Scope

* Renaming `master` to `main`.
* A new task queue, scheduler, database, service, or global daemon.
* Automatic merging, branch deletion, broad cleanup, or modification of the
  preserved root/foreign worktrees.
* Product feedback behavior, semantic productivity scoring, or evaluator
  allocation/credit.
* Publishing `v0.4.0` or changing installer URLs before publication authority.
* Adding a global installed skill or wrapper as part of the repository PR.

## Technical Approach

Use Python standard-library dataclasses and subprocess wrappers around Git and
the GitHub CLI. Keep provider access behind an injected read-only interface so
tests use recorded responses and real temporary Git repositories. Use a
small, schema-versioned receipt store beneath the absolute Git common
directory, with atomic replacement, kernel-held locking, and compare-and-swap
generations. Make inspection pure; guard mutating Trellis lifecycle commands
before their existing mutations.

Implement the following shared interfaces and keep their names stable:

```python
load_delivery_intent(task_json: Path) -> DeliveryIntent
collect_inventory(repo: Path, github: GithubReader) -> Inventory
classify_candidate(intent: DeliveryIntent, inventory: Inventory) -> CandidateStatus
validate_transition(intent: DeliveryIntent, status: CandidateStatus,
                    action: str) -> list[DeliveryIssue]
render_checkpoint(statuses: list[CandidateStatus], max_bytes: int = 1024) -> str
```

Proposed CLI surface:

```text
task.py delivery register <task> --candidate <id> --owner <owner>
task.py delivery inspect <task> --format json|text
task.py delivery record <task> --evidence-file <file> --expected-generation <integer>
task.py delivery close <task> --outcome delivered|superseded|rejected|cancelled
task.py delivery audit --format json|text [--strict --owner <owner>]
task.py delivery next --owner <owner> --format json|text
task.py delivery checkpoint --owner <owner> --format json|text
```

Use task metadata for stable intent and receipt files for changing observations.
Do not persist the candidate's current commit in the source commit that records
it. Compute reachability against a refreshed `origin/master`; use verified PR
records for squash/rebase cases. Keep product terminal state separate from
candidate closure and physical worktree cleanup.

## Verification Commands

```sh
python3 tests/agent_delivery_contract_tests.py
python3 tests/agent_process_contract_tests.py
cargo test -p xtask versioned_ci_artifact_contract_matches_ci_workflow
cargo xtask release-readiness --format json
```

Run the repository's applicable `cargo xtask pr`, evidence, docs, structure,
and hosted workflow gates after the final candidate settles. Do not count a
scope-skipped job as a pass.

## Current Constraints and References

* Current root checkout: `/Users/nroth/workspace/assura` on
  `docs/maturity-portfolio-strategy`, with preserved untracked files.
* Implementation worktree: `/tmp/assura-delivery-closure.X2NThZ` on
  `goal/assura-delivery-closure`.
* Integration ref at task creation: `origin/master` →
  `69e6ca109d27360f90d7cbd886586324f79003c2`.
* Read `AGENTS.md`, `.trellis/workflow.md`, the Assura orchestration skill,
  and the full referenced implementation plan before changing code.
