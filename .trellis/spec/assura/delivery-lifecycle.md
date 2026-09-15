# Delivery lifecycle

Assura delivery is a provenance contract layered on the existing Trellis task
record. A task is not a delivered change merely because it is marked
`completed`, moved to `archive/`, has a branch, or has a green process check.

## Records

New tasks contain `meta.delivery` with `schema_version: 1`, a stable candidate
ID, kind, owner, repository, base ref, and acceptance reference. The candidate
ID is safe for use as a receipt filename and remains stable across rebases and
session changes. Release intents additionally require a validated package
version and unique safe asset names. Historical tasks without this metadata are
`legacy_unclassified` until an owner explicitly classifies them; malformed
typed intents are invalid and remain held.

Mutable observations live outside the task worktree at the Git common
directory's `assura/delivery-v1/` store. Receipts are atomic, generation-aware,
and protected by a kernel-held per-candidate lock. A stale writer fails rather
than overwriting newer evidence. Audits never write receipts, task JSON, refs,
or worktrees.

An active receipt also records its `attached_branch_ref`, which is an observed
worktree attachment rather than a replacement for the stable task intent. When
available, the live checkout branch is authoritative for that observation;
task metadata is only a fallback for a detached checkout. A new or resumed
registration must use the same attached branch. Evidence recording and
terminal closure validate the same binding before mutation. They may run from
the canonical base checkout after integration, but that checkout cannot create
or repair a missing attachment. A non-terminal receipt with no attachment or
a different non-base branch is held as a binding conflict and requires
explicit recovery; it is never silently rebound by a second linked worktree.
This prevents two branches from advancing one candidate and sharing one
receipt. Pausing preserves the attachment and does not imply delivery.

For a legacy non-terminal receipt that predates `attached_branch_ref`, the
explicit recovery is to rerun `delivery register` from a live checkout of the
intent's declared candidate branch with the same candidate and owner. The
command validates the task and receipt identity, then attaches that branch with
a generation-checked update. Repeating the command is idempotent. A base,
detached, or different-branch checkout is rejected without changing the task
or receipt; callers must not edit the receipt store directly.

Each recorded evidence section is a typed `assura.delivery-evidence.v1`
object. It names its source, exact repository, full candidate `head_oid`,
bounded `evidence_ref`, and successful result. Review sections additionally
name a distinct reviewer, role, review identity, and finding dispositions;
check sections name a run and job/check identity; post-merge sections name the
full integration base and merge OID; acceptance sections bind the intent's
authority and a distinct approver to the named criterion and a digest or
measurement. GitHub-sourced review and check sections must also match the
current PR number/head and the live provider payload. A caller-supplied
boolean or stale URL is not terminal evidence.

## State and outcomes

`task.py finish` means pause: it records `session_state: paused` and then
clears the session pointer. It does not imply delivery or abandonment.

The computed candidate phase is based on current Git/GitHub facts and explicit
evidence. Terminal outcomes are `delivered`, `superseded`, `rejected`, and
`cancelled`; missing or conflicting evidence remains unresolved/held.

An integration candidate requires an exact tip, verified integration (including
squash/rebase PR mapping), resolved review, required checks, post-merge proof,
acceptance proof, complete required coverage, and no owned uncommitted changes.
Release candidates additionally require a matching `assura.release-receipt.v1`
record containing the exact source commit, tag object, workflow run, required
asset hashes, checksum verification, and installed version proof. The receipt
is retained as an Actions evidence artifact; retrying publication verifies
matching assets, uploads only missing assets, and stops on conflicts or missing
remote digests. Publication evidence remains distinct from explicit
owner-authorized acceptance, so a release receipt alone cannot close a release
candidate.
Superseded, rejected, and cancelled outcomes require a decision bound to the
exact tip; superseded additionally requires a replacement and remaining-diff
disposition. These outcomes remain distinct from delivered for parent
progress.

Physical archiving is optional storage maintenance after a verified outcome.
The archive command validates the outcome before changing status, session
state, or filesystem paths. Its auto-commit stages only the selected move and
related selected task paths; unrelated staged or dirty work remains untouched.

## Commands

```text
python3 .trellis/scripts/task.py delivery register <task> --candidate <id> --owner <owner>
python3 .trellis/scripts/task.py delivery inspect <task> --format json
python3 .trellis/scripts/task.py delivery record <task> --evidence-file <file> --expected-generation <n>
python3 .trellis/scripts/task.py delivery close <task> --outcome delivered|superseded|rejected|cancelled
python3 .trellis/scripts/task.py delivery audit --format json [--refresh] [--strict --owner <owner>]
python3 .trellis/scripts/task.py delivery next [--owner <owner>]
python3 .trellis/scripts/task.py delivery checkpoint [--owner <owner>]
python3 .agents/skills/assura-goal-execution/scripts/audit-delivery.py --format json
```

`audit`, `next`, and `checkpoint` are read-only. `audit --refresh` performs a
read-only advertised-head/tag query; it does not fetch or mutate local refs.
Without it, remote coverage is explicitly `local-tracking-only` and strict
fleet claims cannot pass. These commands report unavailable GitHub/host
coverage as unknown rather than as an empty successful inventory.
The checkpoint is deterministic for unchanged inputs and is bounded to 1,024
UTF-8 bytes. No delivery command merges, publishes, deploys, deletes branches,
or executes shell text from task metadata.
